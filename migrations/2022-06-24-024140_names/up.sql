-- Your SQL goes here
CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        passwd BLOB NOT NULL,
        salt TEXT NOT NULL
);
