pub mod profile;
pub mod profile_generator;

pub fn type_err(v: &str) -> json::Error {
    json::Error::WrongType(format!("Expected {v}, got something else"))
}

pub use profile::*;
pub use profile_generator::*;
