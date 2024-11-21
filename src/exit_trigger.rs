use crate::profile::ProfileError;

// NOTE: Optimize memory storage by collecting all enums into one byte, and using the remaining of
// an u32 for the value (cpp_rpoject has 20-bit limit on value, we can gain more)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ExitTrigger {
    //r#type: ExitType,
    //comparison: ExitComparison,
    //reference: ExitReferenceType,
    target_stage: u8,
    value: u32,
}

impl ExitTrigger {
    const TYPE_OFFSET: u32 = 0;
    const COMP_OFFSET: u32 = Self::TYPE_OFFSET + ExitType::BIT_SIZE;
    const REF_OFFSET: u32 = Self::COMP_OFFSET + ExitComparison::BIT_SIZE;

    const VALUE_OFFSET: u32 = Self::REF_OFFSET + ExitReferenceType::BIT_SIZE;
    pub const VALUE_MAX: u32 = u32::MAX >> Self::VALUE_OFFSET;

    pub fn new(
        r#type: ExitType,
        comparison: ExitComparison,
        reference: ExitReferenceType,
        target_stage: u8,
        value: u32,
    ) -> Self {
        //TODO: assert value is less than the remaining bits

        assert!(
            value <= Self::VALUE_MAX,
            "Value greater than the maximum allowed, {}",
            Self::VALUE_MAX
        );
        let value = value << Self::VALUE_OFFSET
            | ((r#type as u32) << Self::TYPE_OFFSET)
            | ((comparison as u32) << Self::COMP_OFFSET)
            | ((reference as u32) << Self::REF_OFFSET);

        //println!("{value}");

        Self {
            //r#type,
            //comparison,
            //reference,
            target_stage,
            value,
        }
    }

    pub fn exit_type(&self) -> ExitType {
        //self.r#type
        //(((self.value as ExitType::OutpuT) >> ExitType::OFFSET) & ExitType::SIZE)
        ((self.value >> Self::TYPE_OFFSET) & 2u32.pow(ExitType::BIT_SIZE) - 1)
            .try_into()
            .unwrap()
    }
    pub fn exit_comp(&self) -> ExitComparison {
        //self.comparison
        ((self.value >> Self::COMP_OFFSET) & 2u32.pow(ExitComparison::BIT_SIZE) - 1)
            .try_into()
            .unwrap()
    }
    pub fn exit_ref(&self) -> ExitReferenceType {
        //self.reference
        ((self.value >> Self::REF_OFFSET) & 2u32.pow(ExitReferenceType::BIT_SIZE) - 1)
            .try_into()
            .unwrap()
    }
    pub fn value(&self) -> u32 {
        //self.value
        self.value >> (Self::REF_OFFSET + ExitReferenceType::BIT_SIZE)
    }
    pub fn target_stage(&self) -> u8 {
        self.target_stage // TODO: BIT STUFF
    }

    //fn get_exit_input(&self, driver: &Driver)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
// 3 bits used
pub enum ExitType {
    ExitPressure = 0u8,
    ExitFlow = 1u8,
    ExitTime = 2u8,
    ExitWeight = 3u8,
    ExitPistonPosition = 4u8,
    ExitPower = 5u8,
    ExitTemperature = 6u8,
    ExitButton = 7u8,
}
impl ExitType {
    const MAX_VALUE: u32 = 7;
    //const BIT_SIZE: u32 = Self::MAX_VALUE.ilog2();
    const BIT_SIZE: u32 = if Self::MAX_VALUE.is_power_of_two() && Self::MAX_VALUE != 1 {
        Self::MAX_VALUE.ilog2()
    } else {
        Self::MAX_VALUE.ilog2() + 1
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(u8)]
// 2 bits used
pub enum ExitComparison {
    ExitCompSmaller = 0u8,
    #[default]
    ExitCompGreater = 1u8,
}

impl ExitComparison {
    const MAX_VALUE: u32 = 1;
    const BIT_SIZE: u32 = if Self::MAX_VALUE.is_power_of_two() && Self::MAX_VALUE != 1 {
        Self::MAX_VALUE.ilog2()
    } else {
        Self::MAX_VALUE.ilog2() + 1
    };
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
// 2 bits used
pub enum ExitReferenceType {
    ExitRefAbsolute = 0u8,
    #[default]
    ExitRefSelf = 1u8,
}

impl ExitReferenceType {
    const MAX_VALUE: u32 = 1;
    const BIT_SIZE: u32 = if Self::MAX_VALUE.is_power_of_two() && Self::MAX_VALUE != 1 {
        Self::MAX_VALUE.ilog2()
    } else {
        Self::MAX_VALUE.ilog2() + 1
    };

    pub fn is_absolute(&self) -> bool {
        *self == Self::ExitRefAbsolute
    }
}

impl From<ExitType> for u32 {
    fn from(value: ExitType) -> Self {
        value as u32
    }
}
impl TryFrom<u32> for ExitType {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > Self::MAX_VALUE {
            Err("Value out of bounds")
        } else {
            Ok(unsafe { std::mem::transmute(value as u8) })
        }
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
        if value > Self::MAX_VALUE {
            Err("Value out of bounds")
        } else {
            Ok(unsafe { std::mem::transmute(value as u8) })
        }
    }
}

impl From<ExitReferenceType> for u32 {
    fn from(value: ExitReferenceType) -> Self {
        value as u32
    }
}
impl TryFrom<u32> for ExitReferenceType {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > Self::MAX_VALUE {
            Err("Value out of bounds")
        } else {
            Ok(unsafe { std::mem::transmute(value as u8) })
        }
    }
}

impl From<bool> for ExitReferenceType {
    fn from(value: bool) -> Self {
        if value {
            Self::ExitRefSelf
        } else {
            Self::ExitRefAbsolute
        }
    }
}

// ------------------------------- TryFrom json stuff

impl TryFrom<&json::JsonValue> for ExitTrigger {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &json::JsonValue) -> Result<Self, Self::Error> {
        match e {
            json::JsonValue::Object(o) => Self::try_from(o),
            _ => Err(ProfileError::JsonTypeError(
                "Expected object, got other".to_string(),
            )),
        }
    }
}

impl TryFrom<&json::object::Object> for ExitTrigger {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &json::object::Object) -> Result<Self, Self::Error> {
        let relative = e
            .get("comparison")
            .map(|b| match b {
                json::JsonValue::Boolean(b) => (*b).into(),
                _ => ExitReferenceType::default(),
            })
            .unwrap_or_default();
        Ok(ExitTrigger::new(
            ExitType::try_from(&e["type"])?,
            //.as_str()
            //.ok_or(ProfileError::unexpected_type("string"))
            //.and_then(ExitType::try_from)?,
            //.ok_or(ProfileError::unexpected_type("string"))?,
            e.get("comparison")
                .map(ExitComparison::try_from)
                .unwrap_or(Ok(ExitComparison::default()))
                .unwrap_or_default(),
            //.ok_or(ProfileError::no_name("comparison"))?
            //.try_into()
            //.unwrap_or_default(),
            //.map_or(Ok(ExitComparison::default()), ExitComparison::try_from)?,
            //.and_then(ExitComparison::try_from)
            //.unwrap_or_default(),
            relative, //.ok_or(ProfileError::no_name("relative"))?
            ////.map(JsonValue::as_bool)
            //.as_bool()
            //.ok_or(ProfileError::unexpected_type("bool"))?
            //.into(),
            //e.get("target_stage").map().unwrap_or()
            10, // TODO: TARGET STAGE
            e["value"].as_u32().unwrap_or_default(),
        ))
    }
}

impl TryFrom<&str> for ExitType {
    //type Error = json::Error;
    type Error = ProfileError;

    fn try_from(e: &str) -> Result<Self, Self::Error> {
        match e {
            "pressure" => Ok(Self::ExitPressure),
            "flow" => Ok(Self::ExitFlow),
            "time" => Ok(Self::ExitTime),
            "weight" => Ok(Self::ExitWeight),
            "piston_positon" => Ok(Self::ExitPistonPosition),
            "power" => Ok(Self::ExitPower),
            "temparature" => Ok(Self::ExitTemperature),
            "button" => Ok(Self::ExitButton),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

impl TryFrom<&json::JsonValue> for ExitType {
    type Error = ProfileError;
    //type Error = String;

    fn try_from(e: &json::JsonValue) -> Result<Self, Self::Error> {
        e.as_str()
            .ok_or(ProfileError::unexpected_type("string"))
            .and_then(Self::try_from)
        //Self::try_from(e.as_str().ok_or(ProfileError::unexpected_type("string"))?)
        //.map_err(|_| json::Error::FailedUtf8Parsing)
    }
}

impl TryFrom<json::JsonValue> for ExitType {
    //type Error = json::Error; //<Self as TryFrom<&'a json::JsonValue>>::Error;
    type Error = ProfileError;

    fn try_from(value: json::JsonValue) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&json::JsonValue> for ExitComparison {
    //type Error = json::Error;
    type Error = String;

    fn try_from(e: &json::JsonValue) -> Result<Self, Self::Error> {
        e.as_str()
            .ok_or(ProfileError::unexpected_type("string").to_string())
            .and_then(|s| match s {
                "smaller" => Ok(ExitComparison::ExitCompSmaller),
                "greater" => Ok(ExitComparison::ExitCompGreater),
                x => Err(format!("No correct value provided, got {x}")),
            })
    }
}
