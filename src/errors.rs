//! Defines application-specific error types with descriptive messages
//! and maps them to proper HTTP responses for API clients.

use actix_web::{HttpResponse, ResponseError};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Parsing error: {0}")]
    ParseError(ParseEnumError),
}

/// Error type for enum parsing failures with context.
#[derive(Debug, Clone)]
pub struct ParseEnumError {
    pub input: String,
    pub target_enum: &'static str,
}

impl ParseEnumError {
    pub fn new(input: &str, target_enum: &'static str) -> Self {
        Self {
            input: input.to_string(),
            target_enum,
        }
    }
}

impl fmt::Display for ParseEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Value '{}' is not a valid variant of enum '{}'",
            self.input, self.target_enum
        )
    }
}

impl std::error::Error for ParseEnumError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::DbError(e) => {
                HttpResponse::InternalServerError().body(format!("Internal database error: {}", e))
            }
            AppError::InvalidInput(msg) => HttpResponse::BadRequest().body(msg.clone()),
            AppError::ParseError(e) => {
                HttpResponse::BadRequest().body(format!("Parsing error: {}", e))
            }
        }
    }
}
