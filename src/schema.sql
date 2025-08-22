
-- Table for goats
CREATE TABLE IF NOT EXISTS goats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    breed TEXT NOT NULL,
    name TEXT NOT NULL,
    gender TEXT CHECK(gender IN ('Male', 'Female')) NOT NULL,
    offspring INTEGER DEFAULT 0,
    cost REAL,
    weight REAL,
    current_price REAL,
    diet TEXT,
    last_bred DATE,
    health_status TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Vaccines master table
CREATE TABLE IF NOT EXISTS vaccines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

-- Diseases master table
CREATE TABLE IF NOT EXISTS diseases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

-- Join table for goats and vaccines (many-to-many)
CREATE TABLE IF NOT EXISTS goat_vaccines (
    goat_id INTEGER NOT NULL,
    vaccine_id INTEGER NOT NULL,
    PRIMARY KEY (goat_id, vaccine_id),
    FOREIGN KEY (goat_id) REFERENCES goats(id) ON DELETE CASCADE,
    FOREIGN KEY (vaccine_id) REFERENCES vaccines(id) ON DELETE CASCADE
);

-- Join table for goats and diseases (many-to-many)
CREATE TABLE IF NOT EXISTS goat_diseases (
    goat_id INTEGER NOT NULL,
    disease_id INTEGER NOT NULL,
    PRIMARY KEY (goat_id, disease_id),
    FOREIGN KEY (goat_id) REFERENCES goats(id) ON DELETE CASCADE,
    FOREIGN KEY (disease_id) REFERENCES diseases(id) ON DELETE CASCADE
);

-- Table for workers
CREATE TABLE IF NOT EXISTS workers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    hours_worked INTEGER DEFAULT 0,
    leaves INTEGER DEFAULT 0,
    role TEXT,
    contact TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Equipment table
CREATE TABLE IF NOT EXISTS equipment (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    purchase_date DATE,
    condition TEXT,
    last_maintenance DATE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Sensors table
CREATE TABLE IF NOT EXISTS sensors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sensor_type TEXT NOT NULL,
    location TEXT,
    last_reading REAL,
    last_reading_time TIMESTAMP,
    status TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Spaces table
CREATE TABLE IF NOT EXISTS spaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT CHECK(type IN ('enclosure', 'grazing_field', 'other')),
    capacity INTEGER,
    grass_condition TEXT,
    health TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
