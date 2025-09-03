//!**Main entry point for the Livestock Management Backend server**
//!
//! This module is responsible for initializing the logging infrastructure,
//! managing database connection,
//! and launching the Actix web server with all registered routes.
//!
//! It ensures that the server only starts after a successful migration,
//! preventing runtime errors related to schema mismatch.

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use backend::db::DbPool;
use backend::handlers::goats;
use tracing::info;
use tracing_subscriber;

/// Main asynchronous function to configure and start the backend server.
///
/// # Steps performed:
/// 1. Initialize structured logging with `tracing_subscriber`, respecting the RUST_LOG env var.
/// 2. Open SQLite database connection (or create if missing).
/// 3. Run any pending database schema migrations; exit if migration fails.
/// 4. Wrap the DB connection in a thread-safe pool (`DbPool`).
/// 5. Configure the Actix web server with middleware and route handlers.
/// 6. Bind the server to `127.0.0.1:8000` and run.
///
/// # Panics
/// This function will terminate the process if the database cannot be opened or if migrations fail.
///
/// # Logging
/// - Emits info-level logs during startup phases.
/// - Logs database errors and migration failures at error-level with details.
/// - Default request logs provided by Actix's Logger middleware.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging: use environment variable `RUST_LOG` to set verbosity.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting Livestock Management Backend Server");

    let db_pool = DbPool::new("livestock.db").expect("Failed to create DB pool");

    // Build and run Actix web server.
    // Register logging middleware and route definitions.
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080/")
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .wrap(middleware::Logger::default()) // Logs every request at info level.
            .app_data(web::Data::new(db_pool.clone()))
            .service(
                web::scope("/goats")
                    .route("", web::get().to(goats::get_goats))
                    .route("", web::post().to(goats::add_goat))
                    .route("", web::put().to(goats::update_goat))
                    .route("", web::delete().to(goats::delete_goat)),
            )
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
