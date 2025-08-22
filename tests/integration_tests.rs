use actix_web::{App, test, web};
use backend::db::DbPool;
use backend::handlers::goats::get_goats;
use tracing::{error, info};
use tracing_subscriber;

#[actix_rt::test]
async fn test_db_connection() {
    // Use the test database
    let db_path = "sample_livestock.db";

    // Create the pool
    let pool = DbPool::new(db_path).expect("Failed to create database pool");

    // Attempt to lock the SQLite connection mutex
    {
        let conn = pool
            .conn
            .lock()
            .expect("Failed to lock DB connection mutex");

        // Execute a simple query to verify the DB is accessible and schema exists
        let result = conn.execute_batch("PRAGMA journal_mode;");

        assert!(result.is_ok(), "Database PRAGMA command failed");
    }
}

#[actix_rt::test]
async fn test_get_goats_endpoint() {
    // Initialize tracing logger (does nothing if already initialized)
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
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

    let resp_body = actix_web::test::read_body(resp).await;
    panic!("Response body: {:?}", String::from_utf8_lossy(&resp_body));
    //info!(status = ?resp.status(), "Received response");
    //assert!(resp.status().is_success(), "GET /goats did not succeed");

    //let content_type = resp
    //    .headers()
    //    .get("content-type")
    //    .expect("Missing content-type header");
    //let ct_str = content_type
    //    .to_str()
    //    .expect("Failed to get content-type string");
    //assert!(
    //    ct_str.starts_with("application/json"),
    //    "Content-Type not JSON, got {}",
    //    ct_str
    //);
}
