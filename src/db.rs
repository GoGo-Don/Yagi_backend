//! Database module managing connection pooling, and core entity queries.
//!
//! # Overview
//!
//! This module provides a thread-safe database pool abstraction (`DbPool`) using SQLite and the
//! Rusqlite crate and implements helpers for loading, inserting, and updating complex
//! domain entities like `Goat`,
//! including their many-to-many relations with vaccines and diseases.
//!
//! Detailed multi-level logging is applied throughout for diagnostics and troubleshooting.
//! Errors are carefully mapped to the app’s unified `AppError` type.

use crate::db_helpers::{str_to_breed, str_to_gender};
use crate::errors::AppError;
use crate::models::{DiseaseRef, Goat, VaccineRef};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
//use refinery::embed_migrations;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Row, Transaction};
use std::sync::Arc;
use tracing::{debug, info, trace};

// Embed refinery migrations located inside the `migrations` directory under `src`.
//embed_migrations!("migrations");

/// Thread-safe database pool using r2d2 and rusqlite with connection multiplexing.
#[derive(Clone)]
pub struct DbPool {
    pool: Arc<Pool<SqliteConnectionManager>>,
}

impl DbPool {
    /// Opens or creates the SQLite database at the provided path,
    ///
    /// # Arguments
    /// * `db_path` - The file path to the SQLite database.
    ///
    /// # Errors
    /// Fails if opening the DB fails, wrapped in `AppError::DbError`.
    ///
    /// # Logging
    /// Emits info-level logs on DB open, error-level logs on failure.
    pub fn new(db_path: &str) -> Result<Self, AppError> {
        info!(
            db_path,
            "Opening SQLite database and creating connection pool"
        );

        // Create connection manager with flags
        let manager = SqliteConnectionManager::file(db_path)
            .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE);
        let pool = Pool::new(manager).map_err(AppError::PoolError)?;

        // Get a connection from the pool and enable WAL mode
        {
            let conn = pool.get().map_err(AppError::PoolError)?;
            conn.pragma_update(None, "journal_mode", &"WAL")
                .map_err(AppError::DbError)?;
        }

        // Run migrations here if desired
        //{
        //    let conn = pool.get().map_err(AppError::DbError)?;
        //    // run_migrations(&mut conn).map_err(AppError::DbError)?;
        //}

        info!("Database WAL enabled and ready for use with connection pool");

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Acquires a pooled SQLite connection for use in queries.
    pub fn get_conn(&self) -> Result<PooledConnection<SqliteConnectionManager>, AppError> {
        self.pool.get().map_err(AppError::PoolError)
    }

    /// Maps a SQLite row from the `goats` table to a fully validated and parsed `Goat` struct.
    ///
    /// This method converts string fields into Rust enums and returns application-level parse errors as necessary.
    /// It does not load related vaccinations or diseases; use `load_goat_details` for full loading.
    ///
    /// # Errors
    /// Returns `AppError::ParseError` if enum parsing fails or `DbError` if any DB row field retrieval fails.
    ///
    /// # Logging
    /// Emits trace-level logs indicating mapping operations.
    pub fn row_to_goat(row: &Row) -> Result<Goat, AppError> {
        trace!("Mapping DB row to Goat struct");
        let breed_str: String = row.get(1)?;
        let gender_str: String = row.get(3)?;

        let breed = str_to_breed(&breed_str)?;
        let gender = str_to_gender(&gender_str)?;

        Ok(Goat {
            id: row.get(0)?,
            breed,
            name: row.get(2)?,
            gender,
            offspring: row.get(4)?,
            cost: row.get(5)?,
            weight: row.get(6)?,
            current_price: row.get(7)?,
            diet: row.get(8)?,
            last_bred: row.get(9).ok(),
            health_status: row.get(10)?,
            vaccinations: Vec::new(),
            diseases: Vec::new(),
        })
    }

    /// Loads full details of a goat, including related vaccines and diseases by goat ID.
    ///
    /// Performs multiple queries under the same DB lock to guarantee consistent view of data.
    ///
    /// # Arguments
    /// * `goat_id` - The unique identifier of the goat to load.
    ///
    /// # Errors
    /// Propagates database access or parsing errors as application errors.
    ///
    /// # Logging
    /// Records debug-level messages for loading steps and loaded relation counts.
    pub fn load_goat_details(
        &self,
        conn: &PooledConnection<SqliteConnectionManager>,
        goat_id: i64,
    ) -> Result<Goat, AppError> {
        debug!(goat_id, "Loading full goat details from database");

        let statement = String::from("SELECT * FROM goats WHERE id = ?1");
        let mut stmt = conn.prepare(&statement).map_err(AppError::DbError)?;
        trace!("Prepared {} for loading goat details", &statement);

        let mut goat = stmt.query_row([goat_id], |row| {
            Self::row_to_goat(row).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;

        goat.vaccinations = self.fetch_vaccines(conn, goat_id)?;
        goat.diseases = self.fetch_diseases(conn, goat_id)?;

        debug!(
            %goat_id,
            vaccines_count = goat.vaccinations.len(),
            diseases_count = goat.diseases.len(),
            "Successfully loaded related vaccines and diseases"
        );

        Ok(goat)
    }

    /// Fetches the list of vaccine references associated with a goat.
    ///
    /// # Errors
    /// Returns database errors that occur during querying.
    ///
    /// # Logging
    /// Traces the fetch initiation and debugs the result count.
    pub fn fetch_vaccines(
        &self,
        conn: &Connection,
        goat_id: i64,
    ) -> Result<Vec<VaccineRef>, AppError> {
        trace!(goat_id, "Fetching vaccine list");

        let mut stmt = conn.prepare(
            "SELECT v.id, v.name FROM vaccines v INNER JOIN goat_vaccines gv ON v.id = gv.vaccine_id WHERE gv.goat_id = ?1"
        ).map_err(AppError::DbError)?;

        let vaccines: Vec<VaccineRef> = stmt
            .query_map([goat_id], |row| {
                {
                    Ok(VaccineRef {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                }
            })?
            .filter_map(Result::ok)
            .collect();

        trace!(goat_id, count = vaccines.len(), "Retrieved vaccines");
        Ok(vaccines)
    }

    /// Fetches the list of disease references associated with a goat.
    ///
    /// # Errors
    /// Returns database errors that occur during querying.
    ///
    /// # Logging
    /// Tracks the fetch process with detailed trace and debug logs.
    pub fn fetch_diseases(
        &self,
        conn: &Connection,
        goat_id: i64,
    ) -> Result<Vec<DiseaseRef>, AppError> {
        trace!(goat_id, "Fetching disease list");

        let mut stmt = conn.prepare(
            "SELECT d.id, d.name FROM diseases d INNER JOIN goat_diseases gd ON d.id = gd.disease_id WHERE gd.goat_id = ?1"
        )?;

        let diseases: Vec<DiseaseRef> = stmt
            .query_map([goat_id], |row| {
                {
                    Ok(DiseaseRef {
                        id: row.get(0)?,
                        name: row.get(1)?,
                    })
                }
            })?
            .filter_map(Result::ok)
            .collect();

        trace!(goat_id, count = diseases.len(), "Retrieved diseases");
        Ok(diseases)
    }
}

/// Runs all embedded refinery migrations on the provided connection,
/// ensuring the database schema is current.
///
/// # Errors
/// Returns an application database error if migration fails.
///
/// # Logging
/// Logs migration success and applied migration info at info level,
/// or failure at error level.
pub fn run_migrations(conn: &mut Connection) -> Result<(), AppError> {
    info!("Migrations disabled currently!");
    //info!("Running database migrations...");
    //match embedded_migrations::run(conn) {
    //    Ok(report) => {
    //        info!(affected = ?report.applied_migrations(), "Migrations applied");
    //        Ok(())
    //    }
    //    Err(e) => {
    //        error!("Migration failure: {:?}", e);
    //        Err(AppError::DbError(rusqlite::Error::ExecuteReturnedResults))
    //    }
    //}
    Ok(())
}

/// Attempts to fetch the ID of the vaccine by name in the given transaction.
/// Inserts the vaccine if missing, ensuring referential integrity.
///
/// # Errors
/// Returns a database error if queries or inserts fail.
///
/// # Logging
/// Forwards errors and logs keys steps and outcomes.
pub fn get_or_insert_vaccine(tx: &Transaction, vaccine: &VaccineRef) -> Result<i64, AppError> {
    if let Some(id) = vaccine.id {
        return Ok(id);
    }
    let mut stmt = tx.prepare("SELECT id FROM vaccines WHERE name = ?1")?;
    if let Some(id) = stmt.query_row([&vaccine.name], |r| r.get(0)).optional()? {
        return Ok(id);
    }
    tx.execute("INSERT INTO vaccines (name) VALUES (?1)", [&vaccine.name])?;
    Ok(tx.last_insert_rowid())
}

/// Like `get_or_insert_vaccine`, but for diseases.
pub fn get_or_insert_disease(tx: &Transaction, disease: &DiseaseRef) -> Result<i64, AppError> {
    if let Some(id) = disease.id {
        return Ok(id);
    }
    let mut stmt = tx.prepare("SELECT id FROM diseases WHERE name = ?1")?;
    if let Some(id) = stmt.query_row([&disease.name], |r| r.get(0)).optional()? {
        return Ok(id);
    }
    tx.execute("INSERT INTO diseases (name) VALUES (?1)", [&disease.name])?;
    Ok(tx.last_insert_rowid())
}
