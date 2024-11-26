pub mod dynamics;
mod exit_trigger;
pub mod profile_definition;
mod stage;
mod types;
//pub mod profile_generator;

use json::{Error as JsonError, JsonValue};
pub use profile_definition::*;
pub use stage::*;
pub use types::*;
//pub use profile_generator::*;

#[derive(Debug, Clone)]
pub enum ProfileError {
    Name(String),
    Type(String),
    JsonParsing(String),
}

pub trait FromJson: Sized {
    fn parse_value(value: &JsonValue) -> Result<Self, ProfileError>;
    //fn parse_object(value: &json::object::Object) -> Result<Self, ProfileError>;
}

impl<
        T: for<'a> TryFrom<&'a JsonValue, Error = ProfileError>, //+ for<'a> TryFrom<&'a Object, Error = ProfileError>,
    > FromJson for T
{
    fn parse_value(value: &JsonValue) -> Result<Self, ProfileError> {
        Self::try_from(value)
    }

    //fn parse_object(value: &Object) -> Result<Self, ProfileError> {
    //    Self::try_from(value)
    //}
}

impl ProfileError {
    pub fn unexpected_type<T: AsRef<str>>(t: T) -> Self {
        ProfileError::Type(format!(
            "Expected type `{}`, got something else",
            t.as_ref()
        ))
    }

    pub fn no_name<T: AsRef<str>>(t: T) -> Self {
        Self::Name(format!(
            "Expected entry named `{}`, could not find",
            t.as_ref()
        ))
    }
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let o = match self {
            ProfileError::Name(e) => format!("JsonNameError: {e}"),
            ProfileError::Type(e) => format!("JsonTypeError: {e}"),
            ProfileError::JsonParsing(e) => format!("JsonParseError: {e}"),
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
            } => Self::JsonParsing(format!("Parsing error on {l}:{c}, character: {v}")),
            JsonError::UnexpectedEndOfJson => {
                Self::JsonParsing("Unexpected end of json".to_string())
            }
            JsonError::ExceededDepthLimit => {
                Self::JsonParsing("Depth limit of json reached".to_string())
            }
            JsonError::FailedUtf8Parsing => Self::JsonParsing("Failed UTF8 parsing".to_string()),
            JsonError::WrongType(x) => Self::Type(x),
        }
    }
}
