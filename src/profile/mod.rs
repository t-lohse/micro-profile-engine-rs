pub mod profile_definition;
//pub mod profile_generator;

use json::object::Object;
use json::Result as JsonResult;
use json::{Error as JsonError, JsonValue};
pub use profile_definition::*;
//pub use profile_generator::*;

#[derive(Debug, Clone)]
pub enum ProfileError {
    JsonNameError(String),
    JsonTypeError(String), //JsonNoCorrect
    JsonParseError(String),
}

pub trait FromJson: Sized {
    fn parse_value(value: &json::JsonValue) -> Result<Self, ProfileError>;
    fn parse_object(value: &json::object::Object) -> Result<Self, ProfileError>;
}

impl<T: for<'a> TryFrom<&'a JsonValue, Error = ProfileError> + for<'a> TryFrom<&'a Object, Error = ProfileError>> FromJson
    for T
{
    fn parse_value(value: &JsonValue) -> Result<Self, ProfileError> {
        Self::try_from(value)
    }

    fn parse_object(value: &Object) -> Result<Self, ProfileError> {
        Self::try_from(value)
    }
}

impl ProfileError {
    pub fn unexpected_type<T: AsRef<str>>(t: T) -> Self {
        ProfileError::JsonTypeError(format!(
            "Expected type `{}`, got something else",
            t.as_ref()
        ))
    }

    pub fn no_name<T: AsRef<str>>(t: T) -> Self {
        Self::JsonNameError(format!(
            "Expected entry named `{}`, could not find",
            t.as_ref()
        ))
    }
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let o = match self {
            ProfileError::JsonNameError(e) => format!("JsonNameError: {e}"),
            ProfileError::JsonTypeError(e) => format!("JsonTypeError: {e}"),
            ProfileError::JsonParseError(e) => format!("JsonParseError: {e}"),
        };
        write!(f, "{o}")
    }
}

impl std::error::Error for ProfileError {}

impl From<JsonError> for ProfileError {
    fn from(value: JsonError) -> Self {
        match value {
            JsonError::UnexpectedCharacter {
                ch: v,
                line: l,
                column: c,
            } => Self::JsonParseError(format!("Parsing error on {l}:{c}, character: {v}")),
            JsonError::UnexpectedEndOfJson => todo!(),
            JsonError::ExceededDepthLimit => todo!(),
            JsonError::FailedUtf8Parsing => todo!(),
            JsonError::WrongType(x) => Self::JsonTypeError(x),
        }
    }
}
