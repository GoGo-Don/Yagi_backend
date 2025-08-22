//! HTTP handlers managing CRUD operations for the Goat entity in the livestock backend.
//!
//! This module handles creation, retrieval, updating, and deletion of goats,
//! including managing their associated vaccines and diseases as normalized data.
//!
//! It applies verbose multi-level logging at trace, debug, info, warn, and error levels,
//! assisting with detailed runtime diagnostics and operational transparency.
//!
//! All operations return structured errors using the `AppError` type to communicate
//! clear feedback to API clients while logging internal errors for troubleshooting.

use crate::db::{DbPool, get_or_insert_disease, get_or_insert_vaccine};
use crate::db_helpers::GoatParams;
use crate::errors::AppError;
use crate::models::{Goat, IdPayload};
use actix_web::{HttpResponse, Responder, web};
use tracing::{debug, error, info, trace, warn};

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
    info!("GET /goats called");
    let conn = db.get_conn()?;
    trace!("Acquired connection in get_goats");
    let mut stmt = conn.prepare("SELECT id FROM goats")?;
    let goat_ids = stmt.query_map([], |row| row.get(0))?;

    let mut goats = Vec::new();
    for id_res in goat_ids {
        let id = id_res?;
        trace!(goat_id = id, "Loading goat details");
        match db.load_goat_details(&conn, id) {
            Ok(goat) => goats.push(goat),
            Err(e) => {
                error!(goat_id = id, error=?e, "Failed to load goat details");
                return Err(e);
            }
        }
    }
    info!("Returning {} goats", goats.len());
    Ok(HttpResponse::Ok().json(goats))
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
    new_goat: web::Json<Goat>,
) -> Result<impl Responder, AppError> {
    info!(name = %new_goat.name, "POST /goats called");
    let mut conn = db.get_conn()?;
    trace!("Got connection");

    let tx = conn.transaction()?;
    let params = GoatParams::new(&new_goat)?;

    tx.execute(
        "INSERT INTO goats (breed, name, gender, offspring, cost, weight, current_price, diet, last_bred, health_status) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        &params.as_params(),
    )?;

    let goat_id = tx.last_insert_rowid();
    debug!(goat_id, "Inserted goat base record");

    for vaccine in &new_goat.vaccinations {
        let vaccine_id = get_or_insert_vaccine(&tx, vaccine)?;
        tx.execute(
            "INSERT INTO goat_vaccines (goat_id, vaccine_id) VALUES (?, ?)",
            &[&goat_id, &vaccine_id],
        )?;
        trace!(goat_id, vaccine_id, "Linked vaccine");
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
    goat: web::Json<Goat>,
) -> Result<impl Responder, AppError> {
    let id = goat
        .id
        .ok_or_else(|| AppError::InvalidInput("Goat ID required for update".to_string()))?;

    info!(goat_id = id, "PUT /goats called");

    let mut conn = db.get_conn()?;
    trace!("Got connection");
    let tx = conn.transaction()?;

    let goat_params = GoatParams::new(&goat)?;
    let params = goat_params.as_update_params(&id);

    let affected = tx.execute(
        "UPDATE goats SET breed = ?, name = ?, gender = ?, offspring = ?, cost = ?, weight = ?, current_price = ?, diet = ?, last_bred = ?, health_status = ? WHERE id = ?",
        params,
    )?;

    if affected == 0 {
        warn!(goat_id = id, "No goat found for update");
        return Err(AppError::InvalidInput(format!(
            "No goat found with id {}",
            id
        )));
    }

    tx.execute("DELETE FROM goat_vaccines WHERE goat_id = ?", &[&id])?;
    tx.execute("DELETE FROM goat_diseases WHERE goat_id = ?", &[&id])?;
    debug!(goat_id = id, "Cleared old vaccine and disease links");

    for vaccine in &goat.vaccinations {
        let vaccine_id = get_or_insert_vaccine(&tx, vaccine)?;
        tx.execute(
            "INSERT INTO goat_vaccines (goat_id, vaccine_id) VALUES (?, ?)",
            &[&id, &vaccine_id],
        )?;
        trace!(goat_id = id, vaccine_id, "Linked vaccine");
    }

    for disease in &goat.diseases {
        let disease_id = get_or_insert_disease(&tx, disease)?;
        tx.execute(
            "INSERT INTO goat_diseases (goat_id, disease_id) VALUES (?, ?)",
            &[&id, &disease_id],
        )?;
        trace!(goat_id = id, disease_id, "Linked disease");
    }

    tx.commit()?;
    info!(goat_id = id, "Updated goat and associations successfully");
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
    id_payload: web::Json<IdPayload>,
) -> Result<impl Responder, AppError> {
    info!(goat_id = id_payload.id, "DELETE /goats called");

    let conn = db.get_conn()?;
    let affected = conn.execute("DELETE FROM goats WHERE id = ?", &[&id_payload.id])?;

    if affected == 0 {
        warn!(goat_id = id_payload.id, "Goat not found for deletion");
        return Err(AppError::InvalidInput(format!(
            "No goat found with id {}",
            id_payload.id
        )));
    }

    info!(goat_id = id_payload.id, "Goat deleted successfully");
    Ok(HttpResponse::Ok().body("Goat deleted"))
}
