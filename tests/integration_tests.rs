use actix_web::{App, test, web};
use backend::db::DbPool;
use backend::handlers::goats::{add_goat, delete_goat, get_goats, update_goat};
use backend::models::Goat;
use serde_json::json;
use tracing::{debug, info};
use tracing_subscriber;

#[actix_rt::test]
async fn test_db_connection() {
    // Use the test database
    let db_path = "sample_livestock.db";

    // Create the pool
    let pool = DbPool::new(db_path).expect("Failed to create database pool");

    // Attempt to lock the SQLite connection mutex
    {
        let conn = pool.get_conn().expect("Failed to get connection");

        // Execute a simple query to verify the DB is accessible and schema exists
        let result = conn.execute_batch("PRAGMA journal_mode;");

        assert!(result.is_ok(), "Database PRAGMA command failed");
    }
}

#[actix_rt::test]
async fn test_get_goats_endpoint() {
    // Initialize tracing logger (does nothing if already initialized)
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    info!("Initializing test DB pool");
    let db_pool = DbPool::new("sample_livestock.db").expect("Failed to create DbPool");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db_pool))
            .service(web::scope("/goats").route("", web::get().to(get_goats))),
    )
    .await;

    info!("Sending GET /goats test request");
    let req = test::TestRequest::get().uri("/goats").to_request();
    let resp = test::call_service(&app, req).await;

    info!(status = ?resp.status(), "Received response");
    assert!(resp.status().is_success(), "GET /goats did not succeed");

    let content_type = resp
        .headers()
        .get("content-type")
        .expect("Missing content-type header");
    let ct_str = content_type
        .to_str()
        .expect("Failed to get content-type string");
    assert!(
        ct_str.starts_with("application/json"),
        "Content-Type not JSON, got {}",
        ct_str
    );
}

#[actix_rt::test]
async fn test_add_goat_endpoint() {
    // Initialize tracing (only once per test run)
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    // Setup DB pool pointing to test database
    let db_pool = DbPool::new("sample_livestock.db").expect("Failed to create DbPool");

    // Initialize Actix app with POST /goats route
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db_pool))
            .service(web::scope("/goats").route("", web::post().to(add_goat))),
    )
    .await;

    // Prepare JSON payload for new goat
    let new_goat = json!({
        "breed": "Beetal",
        "name": "TestGoat",
        "gender": "Male",
        "offspring": 1,
        "cost": 100.0,
        "weight": 50.0,
        "current_price": 120.0,
        "diet": "hay",
        "last_bred": null,
        "health_status": "healthy",
        "vaccinations": [],
        "diseases": []
    });

    // Create POST request
    let req = test::TestRequest::post()
        .uri("/goats")
        .set_json(&new_goat)
        .to_request();

    // Call the service and get response
    let resp = test::call_service(&app, req).await;

    // Assert response status is 201 Created
    assert_eq!(resp.status(), 201);

    // Optionally, read and print response body for debug
    let body_bytes = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap_or("<invalid utf8>");
    debug!("Response body: {}", body_str);
}

#[actix_rt::test]
async fn test_update_goat_endpoint() {
    // Init tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();

    let db_pool = DbPool::new("sample_livestock.db").expect("Failed to create DbPool");
    debug!("Pool generated");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db_pool))
            .service(web::scope("/goats").route("", web::put().to(update_goat))),
    )
    .await;
    debug!("App created in test_update_goats");

    // Example of goat data with an existing id (adjust id according to your test DB)
    let updated_goat = json!({
        "id": 3,
        "breed": "Beetal",
        "name": "UpdatedName",
        "gender": "Female",
        "offspring": 2,
        "cost": 110.0,
        "weight": 55.0,
        "current_price": 130.0,
        "diet": "grass",
        "last_bred": null,
        "health_status": "good",
        "vaccinations": [],
        "diseases": []
    });
    debug!("Updated Goat created");

    let req = test::TestRequest::put()
        .uri("/goats")
        .set_json(&updated_goat)
        .to_request();
    debug!("Request ran");

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 200);

    // Optionally, print body for debug
    let body_bytes = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap_or("<invalid utf8>");
    debug!("Response body: {}", body_str);
}

//ToDo: Delete goat has to take a hardcoded id
#[actix_rt::test]
async fn test_delete_goat_endpoint() {
    // Init tracing
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_test_writer()
        .try_init();

    let db_pool = DbPool::new("sample_livestock.db").expect("Failed to create DbPool");

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db_pool))
            .service(web::scope("/goats").route("", web::delete().to(delete_goat))),
    )
    .await;

    // Provide the ID of the goat to delete (adjust based on your test DB content)
    let id_payload = json!({ "id": 2});

    let req = test::TestRequest::delete()
        .uri("/goats")
        .set_json(&id_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success(), "DELETE /goats failed");

    let body_bytes = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap_or("<invalid utf8>");
    debug!("Response body: {}", body_str);
}
