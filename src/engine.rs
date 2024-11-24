use crate::dynamics::{ControlType, InputType, Limits};
use crate::engine::EngineStepResult::{Finished, Next};
use crate::profile::{Flow, Pressure, Profile, StageVariables};
use crate::sensor::{Driver, SensorState};
use std::time::SystemTime;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileState {
    #[default]
    //Idle,
    Start,
    Heating,
    Ready,
    Retracting,
    Brewing,
    Done,
    Purging,
    //End,

    //Error,
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
    //start_time: Option<SystemTime>,
    //state: ProfileState,
}

impl<'a, T: SensorState> ProfileEngineIdle<'a, T> {
    pub fn try_new(
        profile: &'a mut Profile,
        driver: Driver<T>,
        //sampler: Sampler,
        //start_time_stamp: usize,
        //current_stage_id: usize,
    ) -> Result<Self, &'static str> {
        if profile.get_stages().is_empty() {
            return Err("Profile with no states is not allowed");
        }
        //let sampler = Sampler::load_from_stage(&profile.get_stages()[0], 0);

        Ok(Self {
            profile,
            driver,
            //sampler,
            //start_time: None,
            //current_stage_id: 0,
            //state: ProfileState::Idle,
            //_marker: PhantomData::<Idle>,
        })
    }

    pub fn start(self) -> ProfileEngineRunning<'a, T> {
        ProfileEngineRunning {
            driver: self.driver,
            profile: self.profile,
            profile_start_time: SystemTime::now(),
            stage_start_time: SystemTime::now(),
            state: ProfileState::Heating,
            current_stage_id: 1,
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
                // After starting from idle-state, it heats the module
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
                self.current_stage_id = 1; // keys in hashmap
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
                    //self.save_stage_log(None);
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
            } //PS::End => {
              //   // self.state = PS::Idle;
              //}
              //PS::Error => (),
              // NOTE: What I think should happen:
              // PS::Error => return Err("Trying to step from Error state, fix flaw in machine"),
        }

        Next(self)
    }

    // If it's an exiting stage, give Some with the exit timestamp,
    // otherwise it's assumed to be entry, and uses the one set on the value
    fn save_stage_log(&mut self, _exit_time: Option<SystemTime>) {
        //println!("Saving {} log for stage {}. Timestamp = {}", if _exit_time.is_some() { "EXIT"} else {"START"}, this->currentStageId, timestamp);
        let log = self
            .profile
            .get_stage_logs_mut()
            .get_mut(&self.current_stage_id)
            .unwrap();

        if let Some(time) = _exit_time {
            // is exiting stage
            println!(
                "Saving EXIT log for stage {}. Timestamp = {:?}",
                self.current_stage_id, time
            );
            let vars = StageVariables::new(
                self.driver.sensor_data().water_flow().into(),
                self.driver.sensor_data().piston_position().into(),
                self.driver.sensor_data().water_pressure().into(),
                time,
            );
            log.put_entry_log(vars);
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
            log.put_exit_log(vars);
        };

        //log = &this->profile->stage_log[this->currentStageId];

        //vars->flow = writeProfileFlow(this->driver->get_sensor_data().water_flow);
        //vars->piston_position = writeProfilePercent(this->driver->get_sensor_data().piston_position);
        //vars->pressure = writeProfilePressure(this->driver->get_sensor_data().water_pressure);
        //vars->timestamp = timestamp;
        //
        //log->valid = true;
        //self.profile.get_stage_logs_mut().push(StageLog::test())
    }

    fn process_stage_step(&mut self) -> Result<ProfileState, &'static str> {
        use ProfileState as PS;
        //let mut _self = self as *mut Self;

        //if self.current_stage_id as usize >= self.profile.get_stages().len() {
        //    return Err("StageID unreachable");
        //}
        if self.driver.has_reached_final_weight() {
            println!("Profile End reached via final weight hit");
            return Ok(ProfileState::Done);
        }

        println!("executing stage={}\n", self.current_stage_id);

        let elapsed = self.profile_start_time.elapsed().unwrap();

        {
            let stage_log = self
                .profile
                .get_stage_logs()
                .get(&self.current_stage_id)
                .unwrap();
            if !stage_log.is_valid() {
                self.save_stage_log(None);
            }
        }

        let stage = &self.profile.get_stages()[&self.current_stage_id];

        //TODO: What does the sampler actually do???!
        // Ensure the sampler is fed with the right stage
        //if self.sampler.stage_id != self.current_stage_id {
        //    self.sampler.load_new_stage(stage, self.current_stage_id);
        //}
        //
        //let stage_time_stamp = self
        //    .start_time_stamp
        //    .unwrap()
        //    .duration_since(*stage_log.get_start().get_timestamp())
        //    .or_else(|e| Ok::<Duration, ()>(e.duration()))
        //    .unwrap()
        //    .as_millis() as f64;
        //if !stage_log.is_valid() {
        //    unsafe {
        //        //let mut _self = self as *mut Self;
        //        (*_self).save_stage_log(None);
        //    }
        //    //self.save_stage_log(None);
        //}
        let exit_triggers = stage.exit_triggers();

        for trigger in exit_triggers {
            if trigger.check_cond(&self.driver, self.stage_start_time, self.profile_start_time) {
                println!("EXIT COND");

                // If at least one of them is reached
                return if let Some(jump_stage) = trigger.target_stage() {
                    Ok(self.transition_stage(jump_stage))
                } else {
                    // Default is increment
                    Ok(self.transition_stage(self.current_stage_id + 1))
                };
            }
        }

        let stage_dyn = stage.dynamics();
        let input_ref_val = match stage_dyn.input_type() {
            InputType::Time => elapsed.as_secs_f64(),
            InputType::PistonPosition => self.driver.sensor_data().piston_position(),
            InputType::Weight => self.driver.sensor_data().weight(),
        };

        //        let sampled_output = self.sampler.get(input_ref_val);

        let sampled_output = stage_dyn.run_interpolation(input_ref_val);
        println!("sampled ({},{})", input_ref_val, sampled_output);
        println!(
            "Setting output at {:?} ms to {}",
            elapsed.as_millis(),
            sampled_output
        );

        // Don't use the parsed value for limiter checks here as the
        // float might not be perfectly encoding zero
        for l in stage.limits() {
            match l {
                Limits::Pressure(p) => self.driver.set_limited_pressure(Pressure(*p)), // NOTE: In C++ it is limited flow?
                Limits::Flow(f) => self.driver.set_limited_flow(Flow(*f)),
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

        if self.profile.get_stages().get(&target_stage).is_some() {
            self.stage_start_time = SystemTime::now();
            PS::Brewing
        } else {
            println!("Profile End reached via stage end");
            PS::Done
        }

        //self.save_stage_log(Some(SystemTime::now()));
        //
        //if self.current_stage_id == target_stage {
        //    println!("Profile End reached via stage end");
        //    return ProfileState::Done;
        //}
        //
        ////        self.current_stage_id = target_stage.into();
        //self.current_stage_id += 1; // target_stage.into();
        //if self.current_stage_id as usize >= self.profile.get_stages().len() {
        //    println!("Next StageID unreachable");
        //    return ProfileState::Done;
        //}
        //self.save_stage_log(None);
    }
}
