use crate::profile::ProfileError;
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

impl TryFrom<&str> for ControlType {
    type Error = ProfileError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "pressure" => Ok(Self::Pressure),
            "flow" => Ok(Self::Flow),
            "power" => Ok(Self::Power),
            "piston_positon" => Ok(Self::PistonPosition),
            x => Err(ProfileError::Name(format!("Unexpected name: {x}"))),
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
            x => Err(ProfileError::Name(format!("Unexpected name: {x}"))),
        }
    }
}

// How to calculate the slope??
trait InterpolationAlgorithm: std::fmt::Debug {
    fn get_value(&self, points: &[Point], input: f64, current_index: usize) -> f64; // Returns a rate `r` (y = r*x)
}

#[derive(Debug)]
struct LinearInterpolation;

impl InterpolationAlgorithm for LinearInterpolation {
    fn get_value(&self, points: &[Point], input: f64, current_index: usize) -> f64 {
        println!("input: {input}, index: {current_index}");
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
        match value {
            JsonValue::Array(arr) => match <Vec<JsonValue> as AsRef<[JsonValue]>>::as_ref(arr) {
                [x, y] => Ok(Point {
                    x: x.as_f64().ok_or(ProfileError::unexpected_type("f64"))?,
                    y: y.as_f64().ok_or(ProfileError::unexpected_type("f64"))?,
                }),
                _ => Err(ProfileError::JsonParsing(
                    "Expected array with two elements".to_string(),
                )),
            },
            _ => Err(ProfileError::unexpected_type("array")),
        }
    }
}

// In the original coe, the use uninitialized Limis from calloc, effectively
// making the initial value 0, which is what I have done
#[derive(Debug)]
pub enum Limit {
    Pressure(f64), // bar
    Flow(f64),     // ml/s
}

impl TryFrom<&JsonValue> for Limit {
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        json_val_to_obj_tryfrom!(value)
    }
}
impl TryFrom<&Object> for Limit {
    type Error = ProfileError;

    fn try_from(value: &Object) -> Result<Self, Self::Error> {
        let val = value
            .get("value")
            .ok_or(ProfileError::no_name("value"))?
            .as_f64()
            .ok_or(ProfileError::unexpected_type("float"))?;
        match value
            .get("type")
            .ok_or(ProfileError::no_name("type"))?
            .as_str()
            .ok_or(ProfileError::unexpected_type("string"))?
        {
            "flow" => Ok(Limit::Flow(val)),
            "pressure" => Ok(Limit::Pressure(val)),
            x => Err(ProfileError::Name(format!(
                "Limits does not recognize the type value `{x}`"
            ))),
        }
    }
}

#[derive(Debug)]
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
    pub fn input_type(&self) -> InputType {
        self.input_select
    }
    //pub fn points(&self) -> &[Point] {
    //    &self.points
    //}

    pub fn run_interpolation(&self, input: f64) -> f64 {
        match self.find_current_segment(input) {
            SegmentIndexOrValue::Index(i) => self.interpolation.get_value(&self.points, input, i),
            SegmentIndexOrValue::Value(v) => v,
        }
    }

    fn find_current_segment(&self, input: f64) -> SegmentIndexOrValue {
        match <Vec<_> as AsRef<[_]>>::as_ref(&self.points) {
            [first] => SegmentIndexOrValue::Value(first.y),
            [first, ..] if first.x >= input => SegmentIndexOrValue::Value(first.y),
            [.., last] if last.x <= input => SegmentIndexOrValue::Value(last.y),
            arr => SegmentIndexOrValue::Index(
                arr.iter()
                    .enumerate()
                    .find_map(|(i, p)| if p.x >= input { Some(i) } else { None })
                    .unwrap(),
            ),
        }
    }
}

enum SegmentIndexOrValue {
    Index(usize),
    Value(f64),
}

impl TryFrom<&JsonValue> for Dynamics {
    type Error = ProfileError;

    fn try_from(value: &JsonValue) -> Result<Self, Self::Error> {
        json_val_to_obj_tryfrom!(value)
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
                return Err(ProfileError::Name(format!(
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
                JsonValue::Array(arr) => arr
                    .iter()
                    .map(Point::try_from)
                    .collect::<Result<Vec<Point>, ProfileError>>(),
                _ => Err(ProfileError::Type("Expected object, got other".to_string())),
            })??;
        if points.is_empty() {
            return Err(ProfileError::JsonParsing(
                "Note enough points provided, minimum 1".to_string(),
            ));
        }

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
        x => Err(ProfileError::Name(format!(
            "No valid value for type, got `{x}`"
        ))),
    }
}
