use crate::profile::ProfileError;
use crate::profile::StageLog;
use hardware_connection::has_reached_final_weight;
use hardware_connection::set_limited_pressure;
use hardware_connection::set_target_flow;
use hardware_connection::set_target_piston_position;
use hardware_connection::set_target_power;
use hardware_connection::set_target_pressure;

use crate::exit_trigger;
use crate::profile::Profile;
use crate::profile::Temp;
use crate::sensor::Driver;
//use std::marker::PhantomData;
use std::{
    os::linux::raw::stat,
    time::{Duration, SystemTime},
};

use crate::sampler::Sampler;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileState {
    #[default]
    Idle,

    Start,
    Heating,
    Ready,
    Retracting,
    Brewing,
    Done,
    Purging,
    End,

    Error,
}

impl ProfileState {
    pub fn transition(&mut self) {
        use ProfileState as PS;
        *self = match self {
            PS::Idle => todo!(),
            PS::Start => todo!(),
            PS::Heating => todo!(),
            PS::Ready => todo!(),
            PS::Retracting => todo!(),
            PS::Brewing => todo!(),
            PS::Done => todo!(),
            PS::Purging => todo!(),
            PS::End => todo!(),
            PS::Error => PS::Error,
        }
    }
}

//#[derive(Debug)]
pub struct SimplifiedProfileEngine<'a> {
    profile: &'a mut Profile,
    driver: &'a Driver,
    sampler: Sampler,
    start_time_stamp: Option<SystemTime>,
    current_stage_id: u16,
    //_marker: PhantomData<ST>,
    state: ProfileState,
}

impl<'a> SimplifiedProfileEngine<'a> {
    pub fn try_new(
        profile: &'a mut Profile,
        driver: &'a Driver,
        //sampler: Sampler,
        //start_time_stamp: usize,
        //current_stage_id: usize,
    ) -> Result<Self, &'static str> {
        if profile.get_stages().is_empty() {
            return Err("Profile with no states is not allowed");
        }
        let sampler = Sampler::load_from_stage(&profile.get_stages()[0], 0);

        Ok(Self {
            profile,
            driver,
            sampler,
            start_time_stamp: None,
            current_stage_id: 0,
            state: ProfileState::Idle,
            //_marker: PhantomData::<Idle>,
        })
    }

    pub fn start(&mut self) {
        assert!(self.state == ProfileState::Idle, "State is not idle");
        self.state = ProfileState::Heating;
        self.start_time_stamp = Some(SystemTime::now());
    }

    pub fn step(&mut self) -> Result<(), &'static str> {
        use ProfileState as PS;
        if self.state != PS::Idle {}

        match self.state {
            PS::Idle => {
                return Err("Stepping while idle is not allowed");
            }
            PS::Start => {
                self.state = PS::Heating;
            }
            PS::Heating => {
                hardware_connection::set_target_temperature(self.profile.get_temp());
                hardware_connection::set_target_weight(self.profile.get_final_weight());
                // NOTE: Should be a while or async to wait for the
                // hardware to get to temp
                if (hardware_connection::heating_finished()) {
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
                hardware_connection::set_target_piston_position(0.0);
                if self.driver.sensor_data().piston_position <= 1.0 {
                    self.state = PS::Brewing;
                    self.start_time_stamp = Some(SystemTime::now());
                    self.save_stage_log(None);
                }
            }
            PS::Brewing => {
                self.state = self.process_stage_step();
            }
            PS::Done => {
                if self.profile.auto_purge() {
                    self.state = PS::Retracting;
                }
            }
            PS::Purging => {
                hardware_connection::set_target_piston_position(100.0);
                if self.driver.sensor_data().piston_position >= 99.0 {
                    self.state = PS::End;
                }
            }
            PS::End => {
                self.state = PS::Idle;
            }
            PS::Error => (),
            // NOTE: What I think should happen:
            // PS::Error => return Err("Trying to step from Error state, fix flaw in machine"),
        }

        Ok(())
    }

    // If it's an exiting stage, give Some with the exit timestamp, otherwise it's assumed to be
    // entry, and uses the one set on the value
    fn save_stage_log(&mut self, exit_time: Option<SystemTime>) {
        self.profile.get_stage_logs_mut().push(StageLog::test())
    }

    fn process_stage_step(&mut self) -> ProfileState {
        use ProfileState as PS;

        if (self.current_stage_id as usize >= self.profile.get_stages().len()) {
            println!("StageID unreachable");
            return PS::Error;
        }
        if has_reached_final_weight() {
            println!("Profile End reached via final weight hit");
            return ProfileState::Done;
        }

        println!("executing stage={}\n", self.current_stage_id);

        let elapsed = self.start_time_stamp.unwrap().elapsed().unwrap();

        let stage = &self.profile.get_stages()[self.current_stage_id as usize];
        let stage_log: &StageLog = &self.profile.get_stage_logs()[self.current_stage_id as usize];

        // Ensure the sampler is fed with the right stage
        if self.sampler.stage_id != self.current_stage_id {
            self.sampler.load_new_stage(stage, self.current_stage_id);
        }

        let stage_time_stamp = self
            .start_time_stamp
            .unwrap()
            .duration_since(*stage_log.get_start().get_timestamp())
            .unwrap()
            .as_millis() as f64;
        if !stage_log.is_valid() {
            // TODO: fuck this
            self.save_stage_log(None);
        }
        //let stage_time_stamp = (SystemTime.now()
        //    - (self.start_time_stamp.unwrap()
        //        + stage_log.get_start().get_timestamp().elapsed().unwrap()))
        //    / Duration::from_millis(1);
        let exit_triggers = stage.exit_triggers();

        for trigger in exit_triggers {
            if self
                .driver
                .check_exit_cond(trigger, stage_time_stamp, elapsed.as_secs_f64())
            {
                println!("Exit trigger activated!");
                return self.transition_stage(trigger.target_stage());
            }
        }
        let mut input_ref_val = match stage.dynamics().input_type() {
            crate::profile::InputType::InputTime => stage_time_stamp,
            crate::profile::InputType::InputPistonPosition => {
                self.driver.sensor_data().piston_position
            }
            crate::profile::InputType::InputWeight => self.driver.sensor_data().weight,
        };

        let sampled_output = self.sampler.get(input_ref_val);

        println!("sampled ({},{})", input_ref_val, sampled_output);
        println!("Setting output at {:?} ms to {}", elapsed, sampled_output);

        // Dont use the parsed value for limiter checks here as the
        // float might not be perfectly encoding zero
        if (stage.dynamics().limits().flow > crate::profile::Flow(0)) {
            //let flow_limit = parseProfileFlow(stage.dynamics.limits.flow);
            let flow_limit = stage.dynamics().limits().flow;
            hardware_connection::set_limited_flow(flow_limit);
        }
        if (stage.dynamics().limits().pressure > crate::profile::Pressure(0)) {
            let pressure_limit = stage.dynamics().limits().pressure;
            hardware_connection::set_limited_pressure(pressure_limit); // NOTE: In C++ it is limited flow?
        }

        match stage.dynamics().ctrl() {
            crate::profile::ControlType::ControlPressure => {
                set_target_pressure(sampled_output.into())
            }
            crate::profile::ControlType::ControlFlow => set_target_flow(sampled_output.into()),
            crate::profile::ControlType::ControlPower => set_target_power(sampled_output.into()),
            crate::profile::ControlType::ControlPistonPosition => {
                set_target_piston_position(sampled_output.into())
            }
        }

        PS::Brewing
    }

    pub fn get_state(&self) -> ProfileState {
        self.state
    }

    fn transition_stage(&mut self, target_stage: u8) -> ProfileState {
        use ProfileState as PS;

        self.save_stage_log(Some(SystemTime::now()));

        if self.current_stage_id == <u8 as Into<u16>>::into(target_stage) {
            println!("Profile End reached via stage end");
            return ProfileState::Done;
        }

        self.current_stage_id = target_stage.into();
        if self.current_stage_id as usize >= self.profile.get_stages().len() {
            println!("Next StageID unreachable");
            return ProfileState::Done;
        }
        self.save_stage_log(None);
        PS::Brewing
    }
}

mod hardware_connection {
    use crate::profile::{Flow, Pressure, Temp, Weight};

    pub fn heating_finished() -> bool {
        true
    }

    pub fn has_reached_final_weight() -> bool {
        false
    }

    pub fn set_target_weight(set_point: Weight) {
        println!("Setting target weight to {}", Into::<f64>::into(set_point));
    }

    pub fn set_target_temperature(set_point: Temp) {
        println!(
            "Setting target temperature to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_target_pressure(set_point: Pressure) {
        println!(
            "Setting target pressure to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_limited_pressure(set_point: Pressure) {
        println!(
            "Setting target pressure limit to {}",
            Into::<f64>::into(set_point)
        );
    }

    pub fn set_target_flow(set_point: Flow) {
        println!("Setting target flow to {}", Into::<f64>::into(set_point));
    }

    pub fn set_limited_flow(set_point: Flow) {
        println!("Setting flow limit to {}", Into::<f64>::into(set_point));
    }

    pub fn set_target_power(set_point: f64) {
        println!("Setting target power to {}", set_point);
    }

    pub fn set_target_piston_position(set_point: f64) {
        println!("Setting target piston position to {}", set_point);
    }
}
