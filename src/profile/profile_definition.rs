use std::time::SystemTime;

use super::ProfileError;
use crate::exit_trigger::ExitTrigger;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Flow(pub u8);
impl From<f64> for Flow {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}

impl Into<f64> for Flow {
    fn into(self) -> f64 {
        <u8 as Into<f64>>::into(self.0) / 10.0
    }
}
impl Into<u16> for Flow {
    fn into(self) -> u16 {
        (self.0 / 10) as u16
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Pressure(pub u8);
impl From<f64> for Pressure {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}
impl Into<f64> for Pressure {
    fn into(self) -> f64 {
        <u8 as Into<f64>>::into(self.0) / 10.0
    }
}
impl Into<u16> for Pressure {
    fn into(self) -> u16 {
        (self.0 / 10) as u16
    }
}
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Percent(pub u8);
impl From<f64> for Percent {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u8)
    }
}
impl Into<f64> for Percent {
    fn into(self) -> f64 {
        <u8 as Into<f64>>::into(self.0)
    }
}
impl Into<u16> for Percent {
    fn into(self) -> u16 {
        self.0 as u16
    }
}
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Temp(u16);
impl From<f64> for Temp {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}
impl Into<f64> for Temp {
    fn into(self) -> f64 {
        <u16 as Into<f64>>::into(self.0) / 10.0
    }
}
impl Into<u16> for Temp {
    fn into(self) -> u16 {
        (self.0 / 10) as u16
    }
}
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Weight(u16);
impl From<f64> for Weight {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}
impl Into<f64> for Weight {
    fn into(self) -> f64 {
        <u16 as Into<f64>>::into(self.0) / 10.0
    }
}
impl Into<u16> for Weight {
    fn into(self) -> u16 {
        (self.0 / 10) as u16
    }
}
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TimeStamp(u16);
impl From<f64> for TimeStamp {
    fn from(value: f64) -> Self {
        Self((value * 10.0) as u16)
    }
}
impl Into<f64> for TimeStamp {
    fn into(self) -> f64 {
        <u16 as Into<f64>>::into(self.0) / 10.0
    }
}
impl Into<u16> for TimeStamp {
    fn into(self) -> u16 {
        (self.0 / 10) as u16
    }
}
pub struct Profile {
    start_time: usize,
    stage_len: usize,
    temperature: Temp,
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
            temperature: temparature,
            final_weight,
            wait_after_heating,
            auto_purge,
            stage_log,
            stages,
        }
    }

    pub fn get_stages(&self) -> &[Stage] {
        &self.stages
    }
    pub fn get_stages_mut(&mut self) -> &mut Vec<Stage> {
        &mut self.stages
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

    pub fn get_final_weight(&self) -> Weight {
        self.final_weight
    }
    pub fn get_temp(&self) -> Temp {
        self.temperature
    }
    pub fn wait_after_heating(&self) -> bool {
        self.wait_after_heating
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlType {
    ControlPressure = 0u8,
    ControlFlow = 1u8,
    ControlPower = 2u8,
    ControlPistonPosition = 3u8,
}

impl ControlType {
    fn is_percentage(&self) -> bool {
        match self {
            ControlType::ControlPower | ControlType::ControlPistonPosition => true,
            _ => false,
        }
    }
}

impl TryFrom<&str> for ControlType {
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pressure" => Ok(Self::ControlPressure),
            "flow" => Ok(Self::ControlFlow),
            "power" => Ok(Self::ControlPower),
            "piston_positon" => Ok(Self::ControlPistonPosition),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputType {
    InputTime = 0u8,
    InputPistonPosition = 1u8,
    InputWeight = 2u8,
}

impl TryFrom<&str> for InputType {
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "time" => Ok(Self::InputTime),
            "piston_position" => Ok(Self::InputPistonPosition),
            "weight" => Ok(Self::InputWeight),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum InterpolationType {
    #[default]
    InterpolationLinear = 0u8,
    InterpolationCatmull = 1u8,
    InterpolationBezier = 2u8,
}

impl TryFrom<&str> for InterpolationType {
    //type Error = &'static str;
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "linear" => Ok(Self::InterpolationLinear),
            "catmull" => Ok(Self::InterpolationCatmull),
            "bezier" => Ok(Self::InterpolationBezier),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

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
            flow: Flow(0),
            pressure: Pressure(0),
            piston_pos: Percent(0),
            timestamp: SystemTime::now(),
        }
    }
}
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

#[derive(Debug, Clone, Copy)]
pub enum YVal {
    Flow(Flow),
    Pressure(Pressure),
    Power(Percent),
    PistonPosition(Percent),
    Val(u16),
}

impl YVal {
    pub fn value(&self) -> u16 {
        println!("yval: {self:?}");
        match *self {
            YVal::Flow(v) => v.into(),
            YVal::Pressure(v) => v.into(),
            YVal::Power(v) => v.into(),
            YVal::PistonPosition(v) => v.into(),
            YVal::Val(v) => v,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: u16,
    pub y: YVal,
}

impl Point {
    fn new(x: u16, y: YVal) -> Self {
        Self { x, y }
    }
}

// In the original coe, the use uninitialized Limis from calloc, effectively
// making the initial value 0, which is what I have done
#[derive(Default)]
pub struct Limits {
    pub pressure: Pressure,
    pub flow: Flow,
}

impl Limits {
    fn new(pressure: Pressure, flow: Flow) -> Self {
        Limits { pressure, flow }
    }
}

pub struct Dynamics {
    control_select: ControlType,
    input_select: InputType,
    interpolation: InterpolationType,
    //points_len: u8,
    //points: *const Point,
    points: Vec<Point>,
    limits: Limits,
}

impl Dynamics {
    fn new(
        control_select: ControlType,
        input_select: InputType,
        interpolation: InterpolationType,
        points: Vec<Point>,
        limits: Limits,
    ) -> Self {
        Dynamics {
            control_select,
            input_select,
            interpolation,
            points,
            limits,
        }
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    pub fn ctrl(&self) -> ControlType {
        self.control_select
    }
    pub fn interpolation(&self) -> InterpolationType {
        self.interpolation
    }
    pub fn input_type(&self) -> InputType {
        self.input_select
    }
    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

// TODO: Consider some marker trait for this blanket impl
/*
impl<T: for<'a> TryFrom<&'a json::JsonValue>> TryFrom<json::JsonValue> for T {
    type Error = Self::Error;

    fn try_from(value: json::JsonValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl<T: for<'a> TryFrom<&'a json::object::Object>> TryFrom<json::object::Object> for T {
    type Error = Self::Error;

    fn try_from(value: json::object::Object) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
*/

pub struct Stage {
    dynamics: Dynamics,
    //exitTrigger_len: u8,
    //exitTrigger: *const ExitTrigger,
    exit_trigger: Vec<ExitTrigger>,
}

impl Stage {
    pub fn dynamics(&self) -> &Dynamics {
        &self.dynamics
    }
    pub fn exit_triggers(&self) -> &[ExitTrigger] {
        &self.exit_trigger
    }
}

impl TryFrom<&json::JsonValue> for Stage {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(value: &json::JsonValue) -> Result<Self, Self::Error> {
        let json::JsonValue::Object(o) = value else {
            return Err(ProfileError::unexpected_type("object"));
        };
        o.try_into()
    }
}

impl TryFrom<json::JsonValue> for Stage {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(value: json::JsonValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
impl TryFrom<&json::object::Object> for Stage {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(value: &json::object::Object) -> Result<Self, Self::Error> {
        //parseStage(const JsonObject &stageJson, Stage &stage, int16_t default_stage_exit)
        //size_t bytes_allocated = 0;

        let ctrl: ControlType = value["type"]
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))
            .and_then(ControlType::try_from)?;
        //.try_into();
        //.map_err(|_| json::Error::FailedUtf8Parsing)?;
        let interpolation = value["dynamics"]["interpolation"]
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))
            .and_then(InterpolationType::try_from)?;
        //.try_into()?;
        //.map_err(json::Error::WrongType)?;
        let input_sel: InputType = value["dynamics"]["over"]
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))
            .and_then(InputType::try_from)?;
        //.try_into()?;
        //.map_err(|_| json::Error::FailedUtf8Parsing)?;
        let mut points: Vec<Point> = vec![];
        let mut exit_trigger = vec![];
        if let Some(exit_triggers_json) = value.get("exit_triggers") {
            let json::JsonValue::Array(ref exit_triggers_json) = exit_triggers_json else {
                return Err(ProfileError::unexpected_type("array"));
            };
            let json::JsonValue::Array(ref json_points) = value["dynamics"]["points"] else {
                return Err(ProfileError::unexpected_type("array"));
            };

            //let num_points = std::cmp::min(jsonPoints.len(), 100);
            //let num_exit_triggers = std::cmp::min(exit_triggers.len(), 100);
            //let points = Vec::with_capacity(num_points);
            points = json_points
                .into_iter()
                .map(|p| {
                    p[0].as_f64()
                        .ok_or(ProfileError::unexpected_type("float"))
                        .and_then(|x| -> Result<Point, ProfileError> {
                            let y = match ctrl {
                                ControlType::ControlPressure => YVal::Pressure(p[0].as_f64()
                                    .ok_or(ProfileError::unexpected_type("float"))?.into()),
                                ControlType::ControlFlow => YVal::Flow(p[0].as_f64()
                                    .ok_or(ProfileError::unexpected_type("float"))?.into()),
                                ControlType::ControlPower => YVal::Power(p[0].as_f64()
                                    .ok_or(ProfileError::unexpected_type("float"))?.into()),
                                ControlType::ControlPistonPosition => YVal::PistonPosition(p[0].as_f64()
                                    .ok_or(ProfileError::unexpected_type("float"))?.into()),
                            };
                            Ok(Point::new(
                                x as u16,
                                y,
                            ))
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            exit_trigger = exit_triggers_json
                .into_iter()
                .map(TryFrom::try_from)
                .collect::<Result<Vec<ExitTrigger>, _>>()?;
        }

        let limits = if let Some(limits_json) = value.get("limits") {
            let json::JsonValue::Array(limits_json) = limits_json else {
                return Err(ProfileError::unexpected_type("array"));
            };
            let mut iter = limits_json.into_iter().rev();
            let flow = iter
                .find_map(|o| {
                    let json::JsonValue::Object(o) = o else {
                        return None;
                    };
                    o.get("flow")
                        .map(|oo| Flow(oo["value"].as_f64().unwrap() as u8))
                })
                .unwrap_or_default();
            let pressure = iter
                .find_map(|o| {
                    let json::JsonValue::Object(o) = o else {
                        return None;
                    };
                    o.get("pressure")
                        .map(|oo| Pressure(oo["value"].as_f64().unwrap() as u8))
                })
                .unwrap_or_default();

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
