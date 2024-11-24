use super::ProfileError;
use crate::dynamics::{ControlType, Dynamics, Limits};
use crate::exit_trigger::{ExitComparison, ExitTrigger, ExitType};
use json::object::Object;
use json::JsonValue;
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Flow(pub f64);
impl From<f64> for Flow {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}

impl From<Flow> for f64 {
    fn from(val: Flow) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Pressure(pub f64);
impl From<f64> for Pressure {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}
impl From<Pressure> for f64 {
    fn from(val: Pressure) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Percent(pub u8);
impl From<f64> for Percent {
    fn from(value: f64) -> Self {
        Self(value as u8)
    }
}
impl From<Percent> for f64 {
    fn from(val: Percent) -> Self {
        <u8 as Into<f64>>::into(val.0)
    }
}
impl From<Percent> for u16 {
    fn from(val: Percent) -> Self {
        val.0 as u16
    }
}
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Temp(f64);
impl From<f64> for Temp {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}
impl From<Temp> for f64 {
    fn from(val: Temp) -> Self {
        val.0 / 10.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Weight(f64);
impl From<f64> for Weight {
    fn from(value: f64) -> Self {
        Self(value * 10.0)
    }
}
impl From<Weight> for f64 {
    fn from(val: Weight) -> Self {
        val.0 / 10.0
    }
}

// Profile defines the entire profile, with multiple stages.
/* need:
   - Stages : probably an iterator, or vector, depending on if exit_trigger is allowed to jump
   - stage log : Vector with capacity of the parsed stages, filled after/during each stage
   - some constants, including
       - start-time,
       - final-weight,
       - auto-purge,
       - initial-temperature, and
       - wait-after-heating
*/
#[derive(Debug)]
pub struct Profile {
    //start_time: SystemTime,
    starting_temp: Temp,
    target_weight: Weight,
    wait_after_heating: bool,
    auto_purge: bool,

    //stages: *const Stage,
    stages: HashMap<u8, Stage>,
    //stage_log: *const StageLog,
    stage_log: Vec<StageLog>,
}

impl Profile {
    fn new(
        //start_time: SystemTime,
        init_temperature: f64,
        target_weight: f64,
        wait_after_heating: bool,
        auto_purge: bool,
        stages: HashMap<u8, Stage>,
        stage_log: Vec<StageLog>,
    ) -> Self {
        Self {
            //    start_time,
            starting_temp: Temp(init_temperature),
            target_weight: Weight(target_weight),
            wait_after_heating,
            auto_purge,
            stage_log,
            stages,
        }
    }

    pub fn get_starting_temp(&self) -> Temp {
        self.starting_temp
    }

    pub fn get_target_weight(&self) -> Weight {
        self.target_weight
    }

    pub fn wait_after_heating(&self) -> bool {
        self.wait_after_heating
    }

    pub fn get_stages(&self) -> &HashMap<u8, Stage> {
        &self.stages
    }

    //pub fn get_stages_mut(&mut self) -> &mut HashMap<u8, Stage> {
    //    &mut self.stages
    //}

    pub fn get_stage_logs(&self) -> &[StageLog] {
        &self.stage_log
    }
    pub fn get_stage_logs_mut(&mut self) -> &mut Vec<StageLog> {
        &mut self.stage_log
    }

    pub fn auto_purge(&self) -> bool {
        self.auto_purge
    }
}

impl TryFrom<&JsonValue> for Profile {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &JsonValue) -> Result<Self, Self::Error> {
        match e {
            JsonValue::Object(o) => Self::try_from(o),
            _ => Err(ProfileError::Type("Expected object, got other".to_string())),
        }
    }
}

impl TryFrom<&Object> for Profile {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &Object) -> Result<Self, Self::Error> {
        //let temperature: Temp,
        let target_weight = e
            .get("final_weight")
            .map(|v| v.as_f64())
            .ok_or(ProfileError::no_name("final_weight"))?
            .ok_or(ProfileError::unexpected_type("f64"))?;

        let wait_after_heating = e
            .get("wait_after_heating")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let auto_purge = e
            .get("auto_purge")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let temperature = e
            .get("temperature")
            .map(|v| v.as_f64())
            .ok_or(ProfileError::no_name("temperature"))?
            .ok_or(ProfileError::unexpected_type("f64"))?;

        let stages = {
            let stage_json = match e.get("stages").ok_or(ProfileError::no_name("stages"))? {
                JsonValue::Array(arr) => arr,
                _ => return Err(ProfileError::Type("Expected array, got other".to_string())),
            };
            parse_stage(stage_json)?
        };

        let stage_log: Vec<StageLog> = Vec::with_capacity(stages.capacity());
        //let start_time: SystemTime = SystemTime::now();

        Ok(Self {
            target_weight: Weight(target_weight),
            wait_after_heating,
            auto_purge,
            starting_temp: Temp(temperature),
            stages,
            stage_log,
            //start_time,
        })
    }
}

fn parse_stage(value: &[JsonValue]) -> Result<HashMap<u8, Stage>, ProfileError> {
    let names: HashMap<&str, u8> = value
        .iter()
        .zip(1u8..)
        .map(|(v, i)| match v {
            JsonValue::Object(o) => o
                .get("name")
                .ok_or(ProfileError::no_name("name"))?
                .as_str()
                .ok_or(ProfileError::unexpected_type("string"))
                .map(|ob| (ob, i)),
            _ => Err(ProfileError::unexpected_type("object")),
        })
        .collect::<Result<HashMap<&str, u8>, ProfileError>>()?;
    let mut out = HashMap::with_capacity(names.capacity());

    for v in value {
        let v = match v {
            JsonValue::Object(o) => o,
            _ => return Err(ProfileError::unexpected_type("object")),
        };
        let name = v.get("name").unwrap().as_str().unwrap();
        let control_type = match v
            .get("type")
            .ok_or(ProfileError::no_name("type"))?
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))?
        {
            "flow" => ControlType::Flow,
            "pressure" => ControlType::Pressure,
            "power" => ControlType::Power,
            "piston_position" => ControlType::PistonPosition,
            x => {
                return Err(ProfileError::Name(format!(
                    "No valid value for type, got `{x}`"
                )))
            }
        };
        let dynamics = {
            let dynamics_json = v.get("dynamics").ok_or(ProfileError::no_name("dynamics"))?;
            Dynamics::try_from(dynamics_json)?
        };

        let limits = {
            if let Some(limit_json) = v.get("limits") {
                match limit_json {
                    JsonValue::Array(arr) => {
                        arr.iter()
                            .map(Limits::try_from)
                            .collect::<Result<Vec<_>, ProfileError>>()?
                    }
                    _ => return Err(ProfileError::unexpected_type("array")),
                }
            } else {
                vec![]
            }
        };
        let exit_triggers = {
            let triggers = match v
                .get("exit_triggers")
                .ok_or(ProfileError::no_name("exit_triggers"))?
            {
                JsonValue::Array(arr) => arr
                    .iter()
                    .map(|v| match v {
                        JsonValue::Object(o) => Ok(o),
                        _ => Err(ProfileError::unexpected_type("object")),
                    })
                    .collect::<Result<Vec<&Object>, ProfileError>>()?,
                _ => return Err(ProfileError::unexpected_type("array")),
            };
            triggers
                .into_iter()
                .map(|et| {
                    let exit_type = ExitType::try_from(et)?;
                    let exit_comp = if let Some(v) = et.get("comparison") {
                        ExitComparison::try_from(v)?
                    } else {
                        ExitComparison::default()
                    };
                    let target_stage = et
                        .get("target_stage")
                        .and_then(|v| v.as_str())
                        .and_then(|v| names.get(v).copied());
                    let value = et
                        .get("value")
                        .ok_or(ProfileError::no_name("value"))?
                        .as_u32()
                        .ok_or(ProfileError::unexpected_type("int"))?;
                    Ok(ExitTrigger::new(exit_type, exit_comp, target_stage, value))
                })
                .collect::<Result<Vec<ExitTrigger>, ProfileError>>()
        }?;

        out.insert(
            *(names.get(name).unwrap()),
            Stage::new(control_type, dynamics, exit_triggers, limits),
        );
    }

    Ok(out)
}

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
    fn new(
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

#[derive(Debug)]
pub struct StageVariables {
    flow: Flow,
    pressure: Pressure,
    piston_pos: Percent,
    timestamp: SystemTime,
}
impl StageVariables {
    pub fn get_timestamp(&self) -> &SystemTime {
        &self.timestamp
    }
    pub fn test() -> Self {
        Self {
            flow: Flow(0.0),
            pressure: Pressure(0.0),
            piston_pos: Percent(0),
            timestamp: SystemTime::now(),
        }
    }
}

#[derive(Debug)]
pub struct StageLog {
    start: StageVariables,
    end: StageVariables,
    valid: bool,
}

impl StageLog {
    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn get_start(&self) -> &StageVariables {
        &self.start
    }

    pub fn test() -> Self {
        StageLog {
            start: StageVariables::test(),
            end: StageVariables::test(),
            valid: true,
        }
    }
}
