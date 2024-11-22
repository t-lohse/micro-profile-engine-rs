use crate::exit_trigger::ExitTrigger;
use crate::profile::{Flow, Percent, Pressure, ProfileError, Stage};
use json::object::Object;
use json::JsonValue;

// Specifies the type of limit, and/or the interpretation of y-axis values??
// (Can be represented in the YVal of a point)
// (unused in example, hard to decode meaning)
// THE TYPE OF STAGE, DAMN
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlType {
    Pressure = 0u8,
    Flow = 1u8,
    Power = 2u8,
    PistonPosition = 3u8,
}

impl ControlType {
    fn is_percentage(&self) -> bool {
        match self {
            ControlType::Power | ControlType::PistonPosition => true,
            _ => false,
        }
    }
}

impl TryFrom<&str> for ControlType {
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pressure" => Ok(Self::Pressure),
            "flow" => Ok(Self::Flow),
            "power" => Ok(Self::Power),
            "piston_positon" => Ok(Self::PistonPosition),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

// Represents the variable to be controlled in a stage (`over`) - The x-axis?????
// The input to the x-axis to calculate the y value
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputType {
    Time = 0u8,
    PistonPosition = 1u8,
    Weight = 2u8,
}

impl TryFrom<&str> for InputType {
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "time" => Ok(Self::Time),
            "piston_position" => Ok(Self::PistonPosition),
            "weight" => Ok(Self::Weight),
            x => Err(ProfileError::JsonNameError(format!("Unexpected name: {x}"))),
        }
    }
}

// How to calculate the slope??
trait InterpolationAlgorithm {
    fn get_value(&self, points: &[Point], input: f64, current_index: usize) -> f64; // Returns a rate `r` (y = r*x)
}

struct LinearInterpolation;

impl InterpolationAlgorithm for LinearInterpolation {
    fn get_value(&self, points: &[Point], input: f64, current_index: usize) -> f64 {
        let slope: f64 = (points[current_index].y - points[current_index - 1].y)
            / ((points[current_index].x) - (points[current_index - 1].x));

        let intercept = points[current_index].y - slope * (points[current_index].x);
        (slope * input) + intercept
    }
}

// Point on graph, x is time, y is value from control-type
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl TryFrom<&JsonValue> for Point {
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        todo!()
    }
}

// In the original coe, the use uninitialized Limis from calloc, effectively
// making the initial value 0, which is what I have done
pub enum Limits {
    Pressure(f64), // bar
    Flow(f64),     // ml/s
}


impl TryFrom<&JsonValue> for Limits {
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Object(o) => Self::try_from(o),
            _ => Err(ProfileError::JsonTypeError(
                "Expected object, got other".to_string(),
            )),
        }
    }
}
impl TryFrom<&Object> for Limits {
    type Error = ProfileError;

    fn try_from(value: &Object) -> Result<Self, Self::Error> {
        todo!()
    }
}

pub struct Dynamics {
    //control_select: Option<ControlType>, On stage
    input_select: InputType, //`over`
    interpolation: Box<dyn InterpolationAlgorithm>,
    //points_len: u8,
    //points: *const Point,
    points: Vec<Point>,
    //limits: Limits,
}

impl Dynamics {
    fn new(
        //control_select: ControlType,
        input_select: InputType,
        interpolation: Box<dyn InterpolationAlgorithm>,
        points: Vec<Point>,
    ) -> Self {
        Dynamics {
            //control_select,
            input_select,
            interpolation,
            points,
            //limits,
        }
    }

    pub fn input_type(&self) -> InputType {
        self.input_select
    }
    pub fn points(&self) -> &[Point] {
        &self.points
    }
}

impl TryFrom<&JsonValue> for Dynamics {
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        match value {
            JsonValue::Object(o) => Self::try_from(o),
            _ => Err(ProfileError::JsonTypeError(
                "Expected object, got other".to_string(),
            )),
        }
    }
}
impl TryFrom<&Object> for Dynamics {
    type Error = ProfileError;

    fn try_from(value: &Object) -> Result<Self, Self::Error> {
        let input_select = match value
            .get("over")
            .ok_or(ProfileError::no_name("over"))?
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))?
        {
            "time" => InputType::Time,
            "piston_position" => InputType::PistonPosition,
            "weight" => InputType::Weight,
            x => {
                return Err(ProfileError::JsonNameError(format!(
                    "No valid value for type, got `{x}`"
                )))
            }
        };
        let interpolation_name = value
            .get("interpolation")
            .ok_or(ProfileError::no_name("interpolation"))?
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))?;
        let interpolation: Box<dyn InterpolationAlgorithm> =
            get_interpolation_from_name(interpolation_name)?;
        let points: Vec<Point> = value
            .get("points")
            .ok_or(ProfileError::no_name("points"))
            .map(|v| match v {
                JsonValue::Array(arr) => arr.into_iter().map(Point::try_from).collect::<Result<Vec<Point>, ProfileError>>(),
                _ => Err(ProfileError::JsonTypeError(
                    "Expected object, got other".to_string(),
                )),
            })??;

        Ok(Self {
            points,
            interpolation,
            input_select,
        })
    }
}

fn get_interpolation_from_name(
    name: &str,
) -> Result<Box<dyn InterpolationAlgorithm>, ProfileError> {
    match name {
        "linear" => Ok(Box::new(LinearInterpolation)),
        x => Err(ProfileError::JsonNameError(format!(
            "No valid value for type, got `{x}`"
        ))),
    }
}
