use crate::profile::ProfileError;
use crate::sensor::SensorState;
use json::object::Object;
use json::JsonValue;
use std::fmt::Formatter;
use std::time::{Duration, SystemTime};

// ExitTrigger: Condition to exit stage and enter next.
// The variants are the ExitType that must be compared to a value
// If time, value must be specified to be either absolute (for entire profile) or relative (stage)
// What's needed:
// - check_exit - Should take a Driver (trait?) to get the values to compare with, along with the times
// - ref-type should be on the time object
// -
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ExitTrigger {
    //r#type: ExitType,
    //comparison: ExitComparison,
    //reference: ExitReferenceType,
    //target_stage: u8,
    value: u32,
}

impl std::fmt::Debug for ExitTrigger {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ExitTrigger {{ type: {:?}, comparison: {:?}, target_stage {:?}, value: {:?} }}",
            self.exit_type(),
            self.exit_comp(),
            self.target_stage(),
            self.value()
        )
    }
}

// LAYOUT:
//------------------------------------
// VALUE | TARGET_OFFSET | COMP | TYPE
//------------------------------------
impl ExitTrigger {
    const TYPE_OFFSET: u32 = 0;
    const COMP_OFFSET: u32 = Self::TYPE_OFFSET + ExitType::BITS;

    const TARGET_OFFSET: u32 = Self::COMP_OFFSET + ExitComparison::BITS;

    const VALUE_OFFSET: u32 = Self::TARGET_OFFSET + u8::BITS;
    pub const VALUE_MAX: u32 = u32::MAX >> Self::VALUE_OFFSET;

    pub fn new(
        r#type: ExitType,
        comparison: ExitComparison,
        //reference: ExitReferenceType,
        target_stage: Option<u8>,
        value: u32,
    ) -> Self {
        assert!(
            value <= Self::VALUE_MAX,
            "Value greater than the maximum allowed, {}",
            Self::VALUE_MAX
        );

        let value = value << Self::VALUE_OFFSET
            | ((r#type as u32) << Self::TYPE_OFFSET)
            | ((comparison as u32) << Self::COMP_OFFSET)
            | ((target_stage.unwrap_or(0) as u32) << Self::TARGET_OFFSET);

        Self {
            //r#type,
            //comparison,
            //reference,
            //target_stage,
            value,
        }
    }

    pub fn exit_type(&self) -> ExitType {
        //self.r#type
        //(((self.value as ExitType::Output) >> ExitType::OFFSET) & ExitType::SIZE)
        ((self.value >> Self::TYPE_OFFSET) & (2u32.pow(ExitType::BITS) - 1))
            .try_into()
            .unwrap()
    }
    pub fn exit_comp(&self) -> ExitComparison {
        //self.comparison
        ((self.value >> Self::COMP_OFFSET) & (2u32.pow(ExitComparison::BITS) - 1))
            .try_into()
            .unwrap()
    }
    pub fn value(&self) -> u32 {
        //self.value
        self.value >> (Self::VALUE_OFFSET)
    }

    pub fn target_stage(&self) -> Option<u8> {
        let t = (self.value >> (Self::TARGET_OFFSET)) & (2u32.pow(u8::BITS) - 1);
        if t == 0 {
            None
        } else {
            Some(t as u8)
        }
    }

    pub fn check_cond<T: SensorState>(
        &self,
        input: &crate::sensor::Driver<T>,
        stage_timestamp: SystemTime,
        profile_timestamp: SystemTime,
    ) -> bool {
        // this value = lhs, extern input = rhs
        let lhs = match self.exit_type() {
            ExitType::Pressure => input.sensor_data().water_pressure(),
            ExitType::Flow => input.sensor_data().water_flow(),
            ExitType::TimeRelative => SystemTime::now()
                .duration_since(stage_timestamp)
                .or_else(|e| Ok::<Duration, ()>(e.duration()))
                .unwrap()
                .as_secs_f64(),
            ExitType::TimeAbsolute => SystemTime::now()
                .duration_since(profile_timestamp)
                .or_else(|e| Ok::<Duration, ()>(e.duration()))
                .unwrap()
                .as_secs_f64(),
            ExitType::Weight => input.sensor_data().weight(),
            ExitType::PistonPosition => input.sensor_data().piston_position(),
            ExitType::Button => {
                if input.get_button_gesture("Encoder Button", "Single Tap") {
                    f64::MAX
                } else {
                    f64::MIN
                }
            }
            ExitType::Temperature => input.sensor_data().stable_temperature(),
            ExitType::Power => unimplemented!("Exit button not done yet"),
        };
        let rhs = f64::from(self.value());
        println!(
            "{lhs} {:?} {rhs} with type {:?}",
            self.exit_comp(),
            self.exit_type()
        );
        self.exit_comp().comp(lhs, rhs)
    }
}

trait BitSize {
    const MAX_VALUE: u32;
    const BITS: u32 = Self::MAX_VALUE.ilog2() + 1;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
// 3 bits used
pub enum ExitType {
    Pressure = 0u8,
    Flow = 1u8,
    TimeRelative = 2u8,
    TimeAbsolute = 3u8,
    Weight = 4u8,
    PistonPosition = 5u8,
    Power = 6u8,
    Temperature = 7u8,
    Button = 8u8,
}
impl BitSize for ExitType {
    const MAX_VALUE: u32 = 8;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(u8)]
// 2 bits used
pub enum ExitComparison {
    Smaller = 0u8,       // ExitValue <= input
    SmallerStrict = 1u8, // ExitValue < input
    #[default]
    Greater = 2u8, // ExitValue >= input
    GreaterStrict = 3u8, // ExitValue > input
}

impl BitSize for ExitComparison {
    const MAX_VALUE: u32 = 3;
}

impl ExitComparison {
    fn comp<T: PartialOrd>(&self, lhs: T, rhs: T) -> bool {
        match self {
            ExitComparison::Smaller => lhs <= rhs,
            ExitComparison::SmallerStrict => lhs < rhs,
            ExitComparison::Greater => lhs >= rhs,
            ExitComparison::GreaterStrict => lhs > rhs,
        }
    }
}

impl From<ExitType> for u32 {
    fn from(value: ExitType) -> Self {
        value as u32
    }
}

macro_rules! tryfrom_json_exit {
    ($x:expr) => {
        if $x > Self::MAX_VALUE {
            Err("Value out of bounds")
        } else {
            Ok(unsafe { std::mem::transmute::<u8, Self>($x as u8) })
        }
    };
}

impl TryFrom<u32> for ExitType {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        tryfrom_json_exit!(value)
    }
}

impl From<ExitComparison> for u32 {
    fn from(value: ExitComparison) -> Self {
        value as u32
    }
}
impl TryFrom<u32> for ExitComparison {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        tryfrom_json_exit!(value)
    }
}

// ------------------------------- TryFrom json stuff

impl TryFrom<&Object> for ExitType {
    type Error = ProfileError;
    //type Error = String;

    fn try_from(e: &Object) -> Result<Self, Self::Error> {
        let t = e.get("type").ok_or(ProfileError::no_name("type"))?;
        let t = t.as_str().ok_or(ProfileError::unexpected_type("string"))?;

        Ok(match t {
            "pressure" => Self::Pressure,
            "flow" => Self::Flow,
            "weight" => Self::Weight,
            "piston_position" => Self::PistonPosition,
            "power" => Self::Power,
            "temperature" => Self::Temperature,
            "button" => Self::Button,
            "time" => {
                if e.get("relative").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Self::TimeRelative
                } else {
                    Self::TimeAbsolute
                }
            }
            x => {
                return Err(ProfileError::Name(format!(
                    "No valid argument for exit_trigger type, got `{x}`"
                )))
            }
        })
        //Self::try_from(e.as_str().ok_or(ProfileError::unexpected_type("string"))?)
        //.map_err(|_| json::Error::FailedUtf8Parsing)
    }
}

impl TryFrom<&JsonValue> for ExitComparison {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &JsonValue) -> Result<Self, Self::Error> {
        e.as_str()
            .ok_or(ProfileError::unexpected_type("string"))
            .and_then(|s| match s {
                "smaller" | "<=" => Ok(ExitComparison::Smaller),
                "smaller-strict" | "<" => Ok(ExitComparison::SmallerStrict),
                "greater" | ">=" => Ok(ExitComparison::Greater),
                "greater-strict" | ">" => Ok(ExitComparison::GreaterStrict),
                x => Err(ProfileError::Name(format!(
                    "No correct value provided, got {x}"
                ))),
            })
    }
}
