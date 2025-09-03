//! Helper functions for enum conversions and constructing query parameters.
//!
//! Provides safe, leak-free construction of SQLite params from `Goat` instances,
//! along with detailed logging and error handling.

use crate::errors::{AppError, ParseEnumError};
use shared::{Breed, Gender};
use tracing::{debug, trace};

/// Converts a database string to `Gender` enum with detailed error reporting.
pub fn str_to_gender(s: &str) -> Result<Gender, AppError> {
    trace!("Parsing Gender from '{}'", s);
    match s {
        "Male" => Ok(Gender::Male),
        "Female" => Ok(Gender::Female),
        other => {
            debug!("Failed to parse Gender enum from '{}'", other);
            Err(AppError::ParseError(ParseEnumError::new(other, "Gender")))
        }
    }
}

/// Converts a `Gender` enum to a database string.
pub fn gender_to_str(gender: &Gender) -> &str {
    match gender {
        Gender::Male => "Male",
        Gender::Female => "Female",
    }
}

/// Converts a database string to `Breed` enum, treating unknown values as `Other`.
pub fn str_to_breed(s: &str) -> Result<Breed, AppError> {
    trace!("Parsing Breed from '{}'", s);
    match s {
        "Beetal" => Ok(Breed::Beetal),
        "Jamunapari" => Ok(Breed::Jamunapari),
        "Barbari" => Ok(Breed::Barbari),
        "Sirohi" => Ok(Breed::Sirohi),
        "Osmanabadi" => Ok(Breed::Osmanabadi),
        "BlackBengal" => Ok(Breed::BlackBengal),
        "Kutchi" => Ok(Breed::Kutchi),
        "Kaghani" => Ok(Breed::Kaghani),
        "Chegu" => Ok(Breed::Chegu),
        "Jakhrana" => Ok(Breed::Jakhrana),
        other => {
            debug!("Unknown Breed '{}', mapping to Other", other);
            Ok(Breed::Other(other.to_string()))
        }
    }
}

/// Converts a `Breed` enum to a database string.
pub fn breed_to_str(breed: &Breed) -> &str {
    match breed {
        Breed::Beetal => "Beetal",
        Breed::Jamunapari => "Jamunapari",
        Breed::Barbari => "Barbari",
        Breed::Sirohi => "Sirohi",
        Breed::Osmanabadi => "Osmanabadi",
        Breed::BlackBengal => "BlackBengal",
        Breed::Kutchi => "Kutchi",
        Breed::Kaghani => "Kaghani",
        Breed::Chegu => "Chegu",
        Breed::Jakhrana => "Jakhrana",
        Breed::Other(name) => name,
    }
}
