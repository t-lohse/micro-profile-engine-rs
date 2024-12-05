use super::{ProfileError, StageLog, Temp, Weight};
use crate::profile::dynamics::{ControlType, Dynamics, Limit};
use crate::profile::exit_trigger::{ExitComparison, ExitTrigger, ExitType};
use crate::profile::stage::Stage;
use json::object::Object;
use json::JsonValue;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct Profile {
    //start_time: SystemTime,
    starting_temp: Temp,
    target_weight: Weight,
    wait_after_heating: bool,
    auto_purge: bool,

    //stages: *const Stage,
    stages: Vec<Stage>,
    //stage_log: *const StageLog,
    stage_log: Vec<StageLog>,
}

impl Profile {
    pub fn get_starting_temp(&self) -> Temp {
        self.starting_temp
    }

    pub fn get_target_weight(&self) -> Weight {
        self.target_weight
    }

    pub fn wait_after_heating(&self) -> bool {
        self.wait_after_heating
    }

    pub fn get_stages(&self) -> &[Stage] {
        &self.stages
    }

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
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        json_val_to_obj_tryfrom!(value)
    }
}

impl TryFrom<&Object> for Profile {
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

        let stages: BTreeMap<u8, Stage> = {
            let stage_json = match e.get("stages").ok_or(ProfileError::no_name("stages"))? {
                JsonValue::Array(arr) => arr,
                _ => return Err(ProfileError::Type("Expected array, got other".to_string())),
            };
            parse_stage(stage_json)?
        };

        let stage_log: Vec<StageLog> = stages
            .iter()
            .map(|_| StageLog::default())
            .collect();

        Ok(Self {
            target_weight: Weight::from(target_weight),
            wait_after_heating,
            auto_purge,
            starting_temp: Temp::from(temperature),
            stages: stages.into_values().collect(),
            stage_log,//: stage_log.into_values().collect(),
        })
    }
}

fn parse_stage(value: &[JsonValue]) -> Result<BTreeMap<u8, Stage>, ProfileError> {
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

    let mut out = BTreeMap::new();

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
                            .map(Limit::try_from)
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
