use crate::profile::dynamics::{ControlType, Dynamics, Limits};
use crate::profile::exit_trigger::ExitTrigger;
use crate::profile::{Flow, Percent, Pressure};
use std::time::SystemTime;

#[derive(Debug)]
pub struct Stage {
    control_type: ControlType,
    dynamics: Dynamics,
    //exitTrigger_len: u8,
    //exitTrigger: *const ExitTrigger,
    exit_trigger: Vec<ExitTrigger>,
    limits: Vec<Limits>,
}

impl Stage {
    pub(super) fn new(
        control_type: ControlType,
        dynamics: Dynamics,
        //exitTrigger_len: u8,
        //exitTrigger: *const ExitTrigger,
        exit_trigger: Vec<ExitTrigger>,
        limits: Vec<Limits>,
    ) -> Self {
        Self {
            control_type,
            dynamics,
            exit_trigger,
            limits,
        }
    }
    pub fn dynamics(&self) -> &Dynamics {
        &self.dynamics
    }
    pub fn exit_triggers(&self) -> &[ExitTrigger] {
        &self.exit_trigger
    }

    pub fn limits(&self) -> &[Limits] {
        &self.limits
    }

    pub fn ctrl(&self) -> ControlType {
        self.control_type
    }
}

#[derive(Debug, Clone)]
pub struct StageVariables {
    flow: Flow,
    pressure: Pressure,
    piston_pos: Percent,
    timestamp: SystemTime,
}
impl StageVariables {
    pub fn new(flow: Flow, pressure: Pressure, piston_pos: Percent, timestamp: SystemTime) -> Self {
        Self {
            flow,
            pressure,
            piston_pos,
            timestamp,
        }
    }
    pub fn get_timestamp(&self) -> &SystemTime {
        &self.timestamp
    }
}

#[derive(Debug, Default)]
pub struct StageLog {
    entry: Option<StageVariables>,
    exit: Option<StageVariables>,
}

impl StageLog {
    pub fn is_valid(&self) -> bool {
        self.entry.is_some()
    }

    pub fn get_entry(&self) -> Option<&StageVariables> {
        self.entry.as_ref()
    }
    pub fn get_entry_mut(&mut self) -> Option<&mut StageVariables> {
        self.entry.as_mut()
    }
    pub fn put_entry_log(&mut self, vars: StageVariables) -> Option<StageVariables> {
        Self::put_log(&mut self.entry, vars)
    }
    pub fn get_exit(&self) -> Option<&StageVariables> {
        self.exit.as_ref()
    }
    pub fn get_exit_mut(&mut self) -> Option<&mut StageVariables> {
        self.exit.as_mut()
    }
    pub fn put_exit_log(&mut self, vars: StageVariables) -> Option<StageVariables> {
        Self::put_log(&mut self.exit, vars)
    }

    fn put_log(old: &mut Option<StageVariables>, new: StageVariables) -> Option<StageVariables> {
        if let Some(o) = old {
            let out = o.clone();
            *o = new;
            Some(out)
        } else {
            *old = Some(new);
            None
        }
    }
}
