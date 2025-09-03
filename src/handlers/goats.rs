//! This module handles creation, retrieval, updating, and deletion of goats,
//! including managing their associated vaccines and diseases as normalized data.
//!
//! It applies verbose multi-level logging at trace, debug, info, warn, and error levels,
//! assisting with detailed runtime diagnostics and operational transparency.
//!
//! All operations return structured errors using the `AppError` type to communicate
//! clear feedback to API clients while logging internal errors for troubleshooting.

use crate::db::{DbPool, get_or_insert_disease, get_or_insert_vaccine, row_to_goat};
use crate::db_helpers::{breed_to_str, gender_to_str};
use crate::errors::AppError;
use crate::models::NamePayload;
use actix_web::{HttpResponse, Responder, web};
use rusqlite::params;
use shared::{Breed, Gender, GoatParams};
use tracing::{debug, info, trace, warn};

/// Handler for retrieving the full list of goats with complete details.
///
/// # HTTP Method
/// - `GET /goats`
///
/// # Success
/// - Returns HTTP 200 with JSON array containing all goats including their vaccines and diseases.
///
/// # Errors
/// - Returns appropriate error responses if database access or mapping fails.
///
/// # Logs
/// - Info: Entry point of request.
/// - Trace: Loading each goat by ID.
/// - Error: On any failure loading individual goats.
pub async fn get_goats(db: web::Data<DbPool>) -> Result<impl Responder, AppError> {
    debug!("GET /goats called");
    let conn = db.get_conn()?;
    debug!("Acquired connection in get_goats");
    let mut stmt = conn
        .prepare("SELECT * FROM goats")
        .map_err(AppError::DbError)?;
    let goats: Result<Vec<GoatParams>, rusqlite::Error> = stmt
        .query_map([], |row| {
            row_to_goat(row).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?
        .collect();

    let goats = goats?; // propagate or handle your error here

    info!("Returning {} goats", goats.len());
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(goats))
}

/// Handler for adding a new goat along with vaccinations and diseases.
///
/// # HTTP Method
/// - `POST /goats`
///
/// # Request
/// - JSON payload conforming to `Goat` struct.
///
/// # Success
/// - Returns HTTP 201 on successful insertion.
///
/// # Errors
/// - Returns error responses if input validation or database operations fail.
///
/// # Logs
/// - Info: Receipt of add request.
/// - Debug: After inserting base goat entry.
/// - Trace: Adding each vaccine and disease link.
/// - Info: Upon successful commit.
pub async fn add_goat(
    db: web::Data<DbPool>,
    new_goat: web::Json<GoatParams>,
) -> Result<impl Responder, AppError> {
    debug!(name = %new_goat.name, "POST /goats called");
    let mut conn = db.get_conn()?;
    info!("Connection recieved in add_goat instance");

    let tx = conn.transaction()?;

    tx.execute(
        "INSERT INTO goats (breed, name, gender, offspring, cost, weight, current_price, diet, last_bred, health_status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            Breed::to_str(&new_goat.breed),
            &new_goat.name,
            Gender::to_str(&new_goat.gender),
            &new_goat.offspring,
            &new_goat.cost,
            &new_goat.weight,
            &new_goat.current_price,
            &new_goat.diet,
            &new_goat.last_bred,
            &new_goat.health_status,
        ]
    )?;

    let goat_id = tx.last_insert_rowid();
    debug!(goat_id, "Inserted goat base record");

    for vaccine in &new_goat.vaccinations {
        let vaccine_id = get_or_insert_vaccine(&tx, vaccine)?;
        tx.execute(
            "INSERT INTO goat_vaccines (goat_id, vaccine_id) VALUES (?, ?)",
            &[&goat_id, &vaccine_id],
        )?;
        info!(goat_id, vaccine_id, "Linked vaccine");
    }

    for disease in &new_goat.diseases {
        let disease_id = get_or_insert_disease(&tx, disease)?;
        tx.execute(
            "INSERT INTO goat_diseases (goat_id, disease_id) VALUES (?, ?)",
            &[&goat_id, &disease_id],
        )?;
        trace!(goat_id, disease_id, "Linked disease");
    }

    tx.commit()?;
    info!(goat_id, "Successfully added new goat with associations");
    Ok(HttpResponse::Created().body("Goat added"))
}

/// Handler for updating an existing goat and its relations by ID.
///
/// # HTTP Method
/// - `PUT /goats`
///
/// # Request
/// - JSON payload conforming to `Goat` struct, with `id` field.
///
/// # Success
/// - Returns HTTP 200 on successful update.
///
/// # Errors
/// - Returns HTTP 400 for missing `id` or if goat does not exist.
/// - Returns other errors on database failure.
///
/// # Logs
/// - Info: Receipt of update, including `id`.
/// - Debug: After base update, and clearing old relations.
/// - Trace: Adding vaccine and disease links.
/// - Warn/Error: For missing record or update failures.
pub async fn update_goat(
    db: web::Data<DbPool>,
    goat: web::Json<GoatParams>,
) -> Result<impl Responder, AppError> {
    let name = &goat.name;

    info!(goat_name = name, "PUT /goats called");

    let mut conn = db.get_conn()?;
    let tx = conn.transaction()?;

    debug!("Params loaded in update_goat");

    let affected = tx.execute(
        "UPDATE goats 
         SET breed = ?, gender = ?, offspring = ?, cost = ?, weight = ?, current_price = ?, diet = ?, last_bred = ?, health_status = ? 
         WHERE name = ?",
        params![
            Breed::to_str(&goat.breed),
            Gender::to_str(&goat.gender),
            &goat.offspring,
            &goat.cost,
            &goat.weight,
            &goat.current_price,
            &goat.diet,
            &goat.last_bred,
            &goat.health_status,
            &goat.name,
        ],
    )?;

    if affected == 0 {
        warn!(goat_name = name, "No goat found for update");
        return Err(AppError::InvalidInput(format!(
            "No goat found with name {}",
            name
        )));
    } else {
        // Delete existing links for the goat
        tx.execute(
            "DELETE FROM goat_vaccines WHERE goat_id IN (SELECT id FROM goats WHERE name = ?1 LIMIT 1)",
            [&name],
        )?;
        tx.execute(
            "DELETE FROM goat_diseases WHERE goat_id IN (SELECT id FROM goats WHERE name = ?1 LIMIT 1)",
            [&name],
        )?;
        debug!(goat_name = name, "Cleared old vaccine and disease links");

        // Fetch goat id
        let goat_id: i64 = tx.query_row(
            "SELECT id FROM goats WHERE name = ?1 LIMIT 1",
            [&name],
            |row| row.get(0),
        )?;

        // Insert updated vaccine links
        for vaccine in &goat.vaccinations {
            let vaccine_id = get_or_insert_vaccine(&tx, vaccine)?;
            tx.execute(
                "INSERT OR IGNORE INTO goat_vaccines (goat_id, vaccine_id) VALUES (?, ?)",
                &[&goat_id, &vaccine_id],
            )?;
        }
        // Insert updated disease links
        for disease in &goat.diseases {
            let disease_id = get_or_insert_disease(&tx, disease)?;
            tx.execute(
                "INSERT OR IGNORE INTO goat_diseases (goat_id, disease_id) VALUES (?, ?)",
                &[&goat_id, &disease_id],
            )?;
        }
    }

    tx.commit()?;
    info!(
        goat_name = name,
        "Updated goat and associations successfully"
    );
    Ok(HttpResponse::Ok().body("Goat updated"))
}

/// Handler for deleting a goat by ID.
///
/// # HTTP Method
/// - `DELETE /goats`
///
/// # Request
/// - JSON payload containing the goat's `id`.
///
/// # Success
/// - Returns HTTP 200 when deletion is successful.
///
/// # Errors
/// - Returns HTTP 400 if no goat matches the provided ID.
///
/// # Logs
/// - Info: Receipt of delete request.
/// - Warn: If goat not found.
/// - Info: Successful deletion.
pub async fn delete_goat(
    db: web::Data<DbPool>,
    name: web::Json<NamePayload>,
) -> Result<impl Responder, AppError> {
    info!(goat_id = name.name, "DELETE /goats called");

    let conn = db.get_conn()?;
    let affected = conn.execute("DELETE FROM goats WHERE name = ?", &[&name.name])?;

    if affected == 0 {
        warn!(goat_id = name.name, "Goat not found for deletion");
        return Err(AppError::InvalidInput(format!(
            "No goat found with name {}",
            name.name
        )));
    }

    info!(goat_id = name.name, "Goat deleted successfully");
    Ok(HttpResponse::Ok().body("Goat deleted"))
}
