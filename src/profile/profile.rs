use std::ptr;

use super::type_err;

#[repr(transparent)]
pub struct Flow(u8);

impl From<f64> for Flow {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}

#[repr(transparent)]
pub struct Pressure(u8);
impl From<f64> for Pressure {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}
#[repr(transparent)]
pub struct Percent(u8);
impl From<f64> for Percent {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}
#[repr(transparent)]
pub struct Temp(u16);
impl From<f64> for Temp {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}
#[repr(transparent)]
pub struct Weight(u16);
impl From<f64> for Weight {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}
#[repr(transparent)]
pub struct TimeStamp(u16);
impl From<f64> for TimeStamp {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}

pub struct Profile {
    start_time: usize,
    stage_len: usize,
    temparature: Temp,
    final_weight: Weight,
    wait_after_heating: bool,
    auto_purge: bool,

    //stages: *const Stage,
    stages: Vec<Stage>,
    //stage_log: *const StageLog,
    stage_log: Vec<StageLog>,
}

impl Profile {
    pub fn new(
        start_time: usize,
        stage_len: usize,
        temparature: Temp,
        final_weight: Weight,
        wait_after_heating: bool,
        auto_purge: bool,
        stages: Vec<Stage>,
        stage_log: Vec<StageLog>,
    ) -> Self {
        Self {
            start_time,
            stage_len,
            temparature,
            final_weight,
            wait_after_heating,
            auto_purge,
            stage_log,
            stages,
        }
    }
}
#[repr(u8)]
pub enum ControlType {
    CONTROL_PRESSURE = 0u8,
    CONTROL_FLOW = 1u8,
    CONTROL_POWER = 2u8,
    CONTROL_PISTON_POSITION = 3u8,
}
#[repr(u8)]
pub enum InputType {
    INPUT_TIME = 0u8,
    INPUT_PISTON_POSITION = 1u8,
    INPUT_WEIGHT = 2u8,
}
#[repr(u8)]
pub enum InterpolationType {
    INTERPOLATION_LINEAR = 0u8,
    INTERPOLATION_CATMULL = 1u8,
    INTERPOLATION_BEZIER = 2u8,
}
#[repr(u8)]
pub enum ExitType {
    EXIT_PRESSURE = 0u8,
    EXIT_FLOW = 1u8,
    EXIT_TIME = 2u8,
    EXIT_WEIGHT = 3u8,
    EXIT_PISTON_POSITION = 4u8,
    EXIT_POWER = 5u8,
    EXIT_TEMPERATURE = 6u8,
    EXIT_BUTTON = 7u8,
}
#[repr(u8)]
#[derive(Default)]
pub enum ExitComparison {
    EXIT_COMP_SMALLER = 0u8,
    #[default]
    EXIT_COMP_GREATER = 1u8,
}
#[repr(u8)]
#[derive(Default)]
pub enum ExitReferenceType {
    EXIT_REF_ABSOLUTE = 0u8,
    #[default]
    EXIT_REF_SELF = 1u8,
}
pub struct StageVariables {
    flow: Flow,
    pressure: Pressure,
    piston_pos: Percent,
    timestamp: u32,
}
pub struct StageLog {
    start: StageVariables,
    end: StageVariables,
    valid: u8,
}

pub enum YVal {
    Flow(Flow),
    Pressure(Pressure),
    Power(Percent),
    PistonPosition(Percent),
    Val(u8),
}
pub struct Point {
    x: u16,
    y: YVal,
}

impl Point {
    fn new(x: u16, y: YVal) -> Self {
        Self { x, y }
    }
}

#[derive(Default)]
pub struct Limits {
    pressure: Option<Pressure>,
    flow: Option<Flow>,
}

impl Limits {
    fn new(pressure: Option<Pressure>, flow: Option<Flow>) -> Self {
        Limits { pressure, flow }
    }
}

pub struct Dynamics {
    controlSelect: ControlType,
    inputSelect: InputType,
    interpolation: InterpolationType,
    //points_len: u8,
    //points: *const Point,
    points: Vec<Point>,
    limits: Limits,
}

impl Dynamics {
    fn new(
        controlSelect: ControlType,
        inputSelect: InputType,
        interpolation: InterpolationType,
        points: Vec<Point>,
        limits: Limits,
    ) -> Self {
        Dynamics {
            controlSelect,
            inputSelect,
            interpolation,
            points,
            limits,
        }
    }
}

pub struct ExitTrigger {
    r#type: ExitType,
    comparison: ExitComparison,
    reference: ExitReferenceType,
    //target_stage: u8,
    value: u32,
}

impl ExitTrigger {
    fn new(
        r#type: ExitType,
        comparison: ExitComparison,
        reference: ExitReferenceType,
        //target_stage: u8,
        value: u32,
    ) -> Self {
        Self {
            r#type,
            comparison,
            reference,
            //target_stage,
            value,
        }
    }
}

impl TryFrom<json::object::Object> for ExitTrigger {
    type Error = json::Error;

    fn try_from(e: json::object::Object) -> Result<Self, Self::Error> {
        Ok(ExitTrigger::new(
            e["type"]
                .as_str()
                .map(TryInto::try_into)
                .ok_or(type_err("string"))?,
            e.get("comparison")
                .map(TryInto::try_into)
                .unwrap_or_default(),
            e.get("relative").map(TryInto::try_into).unwrap_or_default(),
            //e.get("target_stage").map().unwrap_or()
            e["value"],
        ))
    }
}

pub struct Stage {
    dynamics: Dynamics,
    //exitTrigger_len: u8,
    //exitTrigger: *const ExitTrigger,
    exit_trigger: Vec<ExitTrigger>,
}

impl TryFrom<json::JsonValue> for Stage {
    type Error = json::Error;

    fn try_from(value: json::JsonValue) -> Result<Self, Self::Error> {
        let json::JsonValue::Object(o) = value else {
            return Err(type_err("object"));
        };
        o.try_into()
    }
}
impl TryFrom<json::object::Object> for Stage {
    type Error = json::Error;

    fn try_from(value: json::object::Object) -> Result<Self, Self::Error> {
        //parseStage(const JsonObject &stageJson, Stage &stage, int16_t default_stage_exit)
        //size_t bytes_allocated = 0;

        let ctrl: ControlType = value["type"].as_str()?.try_into()?;
        let interpolation = value["dynamics"]["interpolation"]
            .as_str()
            .ok_or(type_err("string"))?
            .try_into()?;
        let input_sel = stageJson["dynamics"]["over"]
            .as_str()
            .ok_or(type_err("string"))?;
        let mut points = vec![];
        let mut exit_trigger = vec![];
        if let Some(exit_triggers_json) = value.get("exit_triggers") {
            let json::JsonValue::Array(exit_triggers_json) = exit_triggers_json else {
                return Err(type_err("array"));
            };
            let json::JsonValue::Array(jsonPoints) = stageJson["dynamics"]["points"] else {
                return Err(type_err("array"));
            };

            //let num_points = std::cmp::min(jsonPoints.len(), 100);
            //let num_exit_triggers = std::cmp::min(exit_triggers.len(), 100);
            //let points = Vec::with_capacity(num_points);
            points = jsonPoints
                .into_iter()
                .map(|p| {
                    let x = p[0].as_f64().ok_or(type_err("float"))? as u16;
                    if ctrl.is_percentage() {
                        Point::new(x, YVal::Val(p[0].as_f64().ok_or(type_err("float"))? as u8))
                    } else {
                        Point::new(
                            x,
                            YVal::Val(p[0].as_f64().ok_or(type_err("float"))? as u8 * 10),
                        )
                    }
                })
                .collect();

            exit_trigger = exit_triggers_json
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<ExitTrigger>, json::Error>>()?;
        }

        let limits = if let Some(limits_json) = value.get("limits") {
            let json::JsonValue::Array(limits_json) = limits_json else {
                return Err(type_err("array"));
            };
            let iter = limits_json.into_iter().rev();
            let flow = iter.find_map(|o| {
                let json::JsonValue::Object(o) = o else {
                    return None;
                };
                o.get("flow")
                    .map(|oo| Flow(oo["value"].as_f64().unwrap() as u8))
            });
            let pressure = iter.find_map(|o| {
                let json::JsonValue::Object(o) = o else {
                    return None;
                };
                o.get("pressure")
                    .map(|oo| Pressure(oo["value"].as_f64().unwrap() as u8))
            });

            Limits::new(pressure, flow)
        } else {
            Limits::default()
        };

        let dynamics = Dynamics::new(ctrl, input_sel, interpolation, points, limits);

        Ok(Stage {
            dynamics,
            exit_trigger,
        })
    }
}
