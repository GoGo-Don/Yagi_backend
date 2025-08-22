CREATE TABLE IF NOT EXISTS vaccines (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS diseases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS goat_vaccines (
    goat_id INTEGER NOT NULL,
    vaccine_id INTEGER NOT NULL,
    PRIMARY KEY (goat_id, vaccine_id),
    FOREIGN KEY (goat_id) REFERENCES goats(id) ON DELETE CASCADE,
    FOREIGN KEY (vaccine_id) REFERENCES vaccines(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS goat_diseases (
    goat_id INTEGER NOT NULL,
    disease_id INTEGER NOT NULL,
    PRIMARY KEY (goat_id, disease_id),
    FOREIGN KEY (goat_id) REFERENCES goats(id) ON DELETE CASCADE,
    FOREIGN KEY (disease_id) REFERENCES diseases(id) ON DELETE CASCADE
);
