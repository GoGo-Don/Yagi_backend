//! Helper functions for enum conversions and constructing query parameters.
//!
//! Provides safe, leak-free construction of SQLite params from `Goat` instances,
//! along with detailed logging and error handling.

use crate::errors::{AppError, ParseEnumError};
use crate::models::{Breed, Gender, Goat};
use rusqlite::ToSql;
use tracing::{debug, trace};

/// Converts a database string to `Gender` enum with detailed error reporting.
pub fn str_to_gender(s: &str) -> Result<Gender, AppError> {
    trace!("Parsing Gender from '{}'", s);
    match s.to_lowercase().as_str() {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        other => {
            debug!("Failed to parse Gender enum from '{}'", other);
            Err(AppError::ParseError(ParseEnumError::new(other, "Gender")))
        }
    }
}

/// Converts a `Gender` enum to a database string.
pub fn gender_to_str(gender: &Gender) -> &str {
    match gender {
        Gender::Male => "male",
        Gender::Female => "female",
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

/// Structure owning strings and values required for goat database queries.
///
/// Safely stores owned strings and numeric fields, exposing parameters for query bindings.
/// This design avoids lifetime issues by encapsulating all necessary data.
///
/// Usage:
/// ```
/// let params = GoatParams::new(&goat)?;
/// tx.execute(sql, &params.as_params())?;
/// ```
pub struct GoatParams {
    breed: String,
    name: String,
    gender: String,
    diet: String,
    last_bred: String,
    health_status: String,
    offspring: i32,
    cost: f64,
    weight: f64,
    current_price: f64,
}

impl GoatParams {
    /// Creates a new `GoatParams` from a `Goat` by cloning and converting relevant fields.
    ///
    /// # Errors
    ///
    /// Currently does not error but uses a Result wrapper for future-proofing and consistency.
    ///
    /// # Logging
    ///
    /// Logs the formation of query parameters at the debug level.
    pub fn new(goat: &Goat) -> Result<Self, AppError> {
        let breed = breed_to_str(&goat.breed).to_string();
        let name = goat.name.clone();
        let gender = gender_to_str(&goat.gender).to_string();
        let diet = goat.diet.clone();
        let last_bred = goat.last_bred.clone().unwrap_or_default();
        let health_status = goat.health_status.clone();

        debug!("GoatParams created for goat '{}'", &name);

        Ok(Self {
            breed,
            name,
            gender,
            diet,
            last_bred,
            health_status,
            offspring: goat.offspring,
            cost: goat.cost,
            weight: goat.weight,
            current_price: goat.current_price,
        })
    }

    /// Returns a fixed-size array of references to parameters that implement `ToSql` for rusqlite.
    ///
    /// These values are tied to the lifetime of the `GoatParams` instance.
    pub fn as_params(&self) -> [&(dyn ToSql + Sync); 10] {
        [
            &self.breed,
            &self.name,
            &self.gender,
            &self.offspring,
            &self.cost,
            &self.weight,
            &self.current_price,
            &self.diet,
            &self.last_bred,
            &self.health_status,
        ]
    }
    pub fn as_update_params<'a>(&'a self, id: &'a i64) -> impl rusqlite::Params + 'a {
        (
            &self.breed,
            &self.name,
            &self.gender,
            &self.offspring,
            &self.cost,
            &self.weight,
            &self.current_price,
            &self.diet,
            &self.last_bred,
            &self.health_status,
            id,
        )
    }
}
