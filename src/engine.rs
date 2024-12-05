use crate::engine::EngineStepResult::{Finished, Next};
use crate::profile::dynamics::{ControlType, InputType, Limit};
use crate::profile::{Flow, Pressure, Profile, StageVariables};
use crate::sensor::{Driver, SensorState};
use std::time::SystemTime;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileState {
    #[default]
    Start,
    Heating,
    Ready,
    Retracting,
    Brewing,
    Done,
    #[allow(unused)]
    Purging,
}

pub enum EngineStepResult<'a, T: SensorState> {
    Next(ProfileEngineRunning<'a, T>),
    Finished(ProfileEngineIdle<'a, T>),
    Error(&'static str),
}

pub struct ProfileEngineRunning<'a, T: SensorState> {
    driver: Driver<T>,
    profile: &'a mut Profile,

    profile_start_time: SystemTime,
    stage_start_time: SystemTime,
    state: ProfileState,
    current_stage_id: u8,
}

pub struct ProfileEngineIdle<'a, T: SensorState> {
    driver: Driver<T>,
    profile: &'a mut Profile,
}

impl<'a, T: SensorState> ProfileEngineIdle<'a, T> {
    pub fn try_new(profile: &'a mut Profile, driver: Driver<T>) -> Result<Self, &'static str> {
        if profile.get_stages().is_empty() {
            return Err("Profile with no states is not allowed");
        }

        Ok(Self { profile, driver })
    }

    pub fn start(self) -> ProfileEngineRunning<'a, T> {
        ProfileEngineRunning {
            driver: self.driver,
            profile: self.profile,
            profile_start_time: SystemTime::now(),
            stage_start_time: SystemTime::now(),
            state: ProfileState::Heating,
            current_stage_id: 0,
        }
    }
}

impl<'a, T: SensorState> ProfileEngineRunning<'a, T> {
    pub fn step(mut self) -> EngineStepResult<'a, T> {
        use ProfileState as PS;

        match self.state {
            PS::Start => {
                self.state = PS::Heating;
            }
            PS::Heating => {
                self.driver
                    .set_target_temperature(self.profile.get_starting_temp());
                self.driver
                    .set_target_weight(self.profile.get_target_weight());
                // NOTE: Maybe it should be a while or async to wait for the
                // hardware to get to temp
                if self.driver.heating_finished() {
                    self.state = PS::Ready;
                }
            }
            PS::Ready => {
                self.current_stage_id = 0;
                if !self.profile.wait_after_heating() {
                    self.state = PS::Retracting;
                }
            }
            PS::Retracting => {
                self.driver.set_target_piston_position(0.0);
                if self.driver.sensor_data().piston_position() <= 1.0 {
                    self.state = PS::Brewing;
                    self.profile_start_time = SystemTime::now();
                    self.stage_start_time = SystemTime::now();
                }
            }
            PS::Brewing => {
                self.state = match self.process_stage_step() {
                    Ok(s) => s,
                    Err(e) => return EngineStepResult::Error(e),
                };
            }
            PS::Done => {
                if self.profile.auto_purge() {
                    self.state = PS::Retracting;
                }
            }
            PS::Purging => {
                self.driver.set_target_piston_position(100.0);
                if self.driver.sensor_data().piston_position() >= 99.0 {
                    return Finished(ProfileEngineIdle {
                        profile: self.profile,
                        driver: self.driver,
                    });
                }
            }
        }

        Next(self)
    }

    fn save_stage_log(&mut self, _exit_time: Option<SystemTime>) {
        let log = self
            .profile
            .get_stage_logs_mut()
            .get_mut(self.current_stage_id as usize)
            .unwrap();
        let vars = StageVariables::new(
            self.driver.sensor_data().water_flow().into(),
            self.driver.sensor_data().piston_position().into(),
            self.driver.sensor_data().water_pressure().into(),
            _exit_time.unwrap_or(SystemTime::now()),
        );

        if let Some(time) = _exit_time {
            // is exiting stage
            println!(
                "Saving EXIT log for stage {}. Timestamp = {:?}",
                self.current_stage_id, time
            );

            log.put_exit_log(vars);
        } else {
            // is entry stage
            let now = SystemTime::now();
            println!(
                "Saving ENTRY log for stage {}. Timestamp = {:?}",
                self.current_stage_id, now
            );
            let vars = StageVariables::new(
                self.driver.sensor_data().water_flow().into(),
                self.driver.sensor_data().piston_position().into(),
                self.driver.sensor_data().water_pressure().into(),
                now,
            );
            log.put_entry_log(vars);
        };
    }

    fn process_stage_step(&mut self) -> Result<ProfileState, &'static str> {
        use ProfileState as PS;
        if self.driver.has_reached_final_weight() {
            println!("Profile End reached via final weight hit");
            return Ok(ProfileState::Done);
        }

        println!("executing stage={}", self.current_stage_id);

        let elapsed = self.profile_start_time.elapsed().unwrap();

        {
            let stage_log = &self
                .profile
                .get_stage_logs()[self.current_stage_id as usize];
            if !stage_log.is_valid() {
                self.save_stage_log(None);
            }
        }

        let stage = &self.profile.get_stages()[self.current_stage_id as usize];

        let exit_triggers = stage.exit_triggers();

        for trigger in exit_triggers {
            if trigger.check_cond(&self.driver, self.stage_start_time, self.profile_start_time) {
                println!("EXIT COND");

                return Ok(self.transition_stage(
                    trigger.target_stage().unwrap_or(self.current_stage_id + 1),
                ));
            }
        }

        let stage_dyn = stage.dynamics();
        let input_ref_val = match stage_dyn.input_type() {
            InputType::Time => elapsed.as_secs_f64(),
            InputType::PistonPosition => self.driver.sensor_data().piston_position(),
            InputType::Weight => self.driver.sensor_data().weight(),
        };

        let sampled_output = stage_dyn.run_interpolation(input_ref_val);
        println!("sampled ({},{})", input_ref_val, sampled_output);
        println!(
            "Setting output at {:?} ms to {}",
            elapsed.as_millis(),
            sampled_output
        );

        for l in stage.limits() {
            match l {
                Limit::Pressure(p) => self.driver.set_pressure_limit(Pressure::from(*p)), // NOTE: In C++ it is limited flow?
                Limit::Flow(f) => self.driver.set_flow_limit(Flow::from(*f)),
            }
        }

        match stage.ctrl() {
            ControlType::Pressure => self.driver.set_target_pressure(sampled_output.into()),
            ControlType::Flow => self.driver.set_target_flow(sampled_output.into()),
            ControlType::Power => self.driver.set_target_power(sampled_output),
            ControlType::PistonPosition => self.driver.set_target_piston_position(sampled_output),
        }

        Ok(PS::Brewing)
    }

    pub fn get_state(&self) -> ProfileState {
        self.state
    }

    fn transition_stage(&mut self, target_stage: u8) -> ProfileState {
        use ProfileState as PS;

        self.save_stage_log(Some(self.stage_start_time));

        self.current_stage_id = target_stage;

        if self
            .profile
            .get_stages().len() > self.current_stage_id as usize
        {
            self.stage_start_time = SystemTime::now();
            PS::Brewing
        } else {
            println!("Profile End reached via stage end");
            PS::Done
        }
    }
}
