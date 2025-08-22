//! Database utility functions and state

use crate::errors::AppError;
use crate::models::Goat;
use rusqlite::{Connection, Row, params};
use std::sync::Mutex;

/// Holds the shared SQLite connection pool/state
pub struct DbPool {
    pub conn: Mutex<Connection>,
}

impl DbPool {
    /// Create new DB connection pool given DB file path
    pub fn new(db_path: &str) -> Result<Self, AppError> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Maps a SQLite row to Goat struct
    pub fn row_to_goat(row: &Row) -> Result<Goat, rusqlite::Error> {
        Ok(Goat {
            id: row.get(0)?,
            breed: row.get(1)?,
            name: row.get(2)?,
            gender: row.get(3)?,
            vaccinations: row.get(4)?,
            diseases: row.get(5)?,
            offspring: row.get(6)?,
            cost: row.get(7)?,
            weight: row.get(8)?,
            current_price: row.get(9)?,
            diet: row.get(10)?,
            last_bred: row.get(11).ok(),
            health_status: row.get(12)?,
        })
    }
}
