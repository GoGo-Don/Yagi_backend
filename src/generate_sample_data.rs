//! Generates sample livestock data with vaccines, diseases, and relationships

use chrono::NaiveDate;
use rand::{Rng, seq::SliceRandom};
use rusqlite::{Connection, Result, params};

fn main() -> Result<()> {
    let conn = Connection::open("sample_livestock.db")?;

    // Load schema from file or ensure created manually before running this

    let mut rng = rand::thread_rng();

    // Insert vaccines
    let vaccines = vec!["Rabies", "CDT", "Clostridium", "FootAndMouth"];
    for vaccine in &vaccines {
        conn.execute(
            "INSERT OR IGNORE INTO vaccines (name) VALUES (?1)",
            params![vaccine],
        )?;
    }

    // Insert diseases
    let diseases = vec!["FootRot", "Mastitis", "Parasites", "Pneumonia"];
    for disease in &diseases {
        conn.execute(
            "INSERT OR IGNORE INTO diseases (name) VALUES (?1)",
            params![disease],
        )?;
    }

    // Helper random date generator
    fn random_date(start: &str, end: &str) -> NaiveDate {
        let start = NaiveDate::parse_from_str(start, "%Y-%m-%d").unwrap();
        let end = NaiveDate::parse_from_str(end, "%Y-%m-%d").unwrap();
        let days = (end - start).num_days();
        let offset = rand::thread_rng().gen_range(0..=days);
        start + chrono::Duration::days(offset)
    }

    // Breeds relevant to India
    let breeds = [
        "Beetal",
        "Jamunapari",
        "Barbari",
        "Sirohi",
        "Osmanabadi",
        "BlackBengal",
        "Kutchi",
        "Kaghani",
        "Chegu",
        "Jakhrana",
    ];
    let genders = ["Male", "Female"];
    let diets = ["Hay", "Pasture", "Mixed"];

    // Cache vaccine IDs
    let vaccine_ids: Vec<(i64, String)> = conn
        .prepare("SELECT id, name FROM vaccines")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(Result::ok)
        .collect();

    // Cache disease IDs
    let disease_ids: Vec<(i64, String)> = conn
        .prepare("SELECT id, name FROM diseases")?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .filter_map(Result::ok)
        .collect();

    // Insert ~20 goats with random associations
    for i in 1..=20 {
        let breed = breeds[rng.gen_range(0..breeds.len())];
        let name = format!("Goat{}", i);
        let gender = genders[rng.gen_range(0..genders.len())];
        let offspring = rng.gen_range(0..5);
        let cost = rng.gen_range(100.0..250.0);
        let weight = rng.gen_range(40.0..90.0);
        let current_price = cost * rng.gen_range(1.1..1.5);
        let diet = diets[rng.gen_range(0..diets.len())];
        let last_bred = random_date("2024-01-01", "2025-08-01").to_string();
        let health_status = if i % 15 == 0 { "recovering" } else { "healthy" };

        conn.execute(
            "INSERT INTO goats (breed, name, gender, offspring, cost, weight, current_price, diet, last_bred, health_status)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![breed, name, gender, offspring, cost, weight, current_price, diet, last_bred, health_status],
        )?;

        let goat_id = conn.last_insert_rowid();

        // Assign random vaccines to goat
        let count = rng.gen_range(1..=3);
        let assigned_vaccine_ids = vaccine_ids
            .choose_multiple(&mut rng, count)
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for &v_id in &assigned_vaccine_ids {
            conn.execute(
                "INSERT INTO goat_vaccines (goat_id, vaccine_id) VALUES (?1, ?2)",
                params![goat_id, v_id],
            )?;
        }

        // Assign random diseases to goat (mostly none or few)
        let count = rng.gen_range(1..=2);
        let assigned_disease_ids = disease_ids
            .choose_multiple(&mut rng, if i % 10 == 0 { count } else { 0 })
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();

        for &d_id in &assigned_disease_ids {
            conn.execute(
                "INSERT INTO goat_diseases (goat_id, disease_id) VALUES (?1, ?2)",
                params![goat_id, d_id],
            )?;
        }
    }

    // Insert ~10 workers
    for i in 1..=10 {
        let name = format!("Worker{}", i);
        let hours_worked = rng.gen_range(120..200);
        let leaves = rng.gen_range(0..10);
        let role = if i % 2 == 0 {
            "Feeder"
        } else {
            "Health Monitor"
        };
        let contact = format!("worker{}@farm.com", i);
        conn.execute(
            "INSERT INTO workers (name, hours_worked, leaves, role, contact) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, hours_worked, leaves, role, contact],
        )?;
    }

    // Insert ~5 equipment
    let equipments = vec![
        (
            "Feeder",
            "Automatic feed dispenser",
            "2023-05-10",
            "Good",
            "2025-01-15",
        ),
        (
            "Pesticide Sprayer",
            "Field pesticide sprayer",
            "2022-07-20",
            "Fair",
            "2024-11-01",
        ),
        (
            "Water Pump",
            "Irrigation water pump",
            "2021-09-05",
            "Excellent",
            "2025-07-12",
        ),
        (
            "Tractor",
            "Farm tractor",
            "2020-03-14",
            "Good",
            "2025-02-28",
        ),
        (
            "Milking Machine",
            "Automated milking",
            "2023-01-22",
            "Good",
            "2025-06-05",
        ),
    ];
    for (name, desc, purchase, condition, maintenance) in equipments {
        conn.execute(
            "INSERT INTO equipment (name, description, purchase_date, condition, last_maintenance) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, desc, purchase, condition, maintenance],
        )?;
    }

    // Insert ~100 sensors
    let sensor_types = vec![
        "Camera",
        "RFID Scanner",
        "Health Monitor",
        "Temp Sensor",
        "Humidity Sensor",
    ];
    let locations = vec!["Enclosure 1", "Field 3", "Barn", "Fence", "Water Station"];

    for i in 1..=100 {
        let sensor_type = sensor_types[rng.gen_range(0..sensor_types.len())];
        let location = locations[rng.gen_range(0..locations.len())];
        let last_reading = rng.gen_range(0.0..100.0);
        let last_reading_time = random_date("2025-01-01", "2025-08-20").to_string();
        let status = if i % 20 == 0 { "Inactive" } else { "Active" };

        conn.execute(
            "INSERT INTO sensors (sensor_type, location, last_reading, last_reading_time, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![sensor_type, location, last_reading, last_reading_time, status],
        )?;
    }

    // Insert some spaces
    let spaces = vec![
        ("Enclosure 1", "enclosure", 50, "Good", "Healthy"),
        ("Grazing Field A", "grazing_field", 100, "Fair", "Healthy"),
        ("Barn", "other", 10, "-", "-"),
        ("Enclosure 2", "enclosure", 60, "Good", "Healthy"),
    ];
    for (name, typ, capacity, grass_cond, health) in spaces {
        conn.execute(
            "INSERT INTO spaces (name, type, capacity, grass_condition, health) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, typ, capacity, grass_cond, health],
        )?;
    }

    println!("Sample livestock database generated successfully.");
    Ok(())
}
