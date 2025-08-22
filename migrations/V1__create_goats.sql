
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
