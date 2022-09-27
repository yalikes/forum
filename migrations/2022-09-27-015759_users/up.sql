-- Your SQL goes here
DROP TABLE IF EXISTS users;
CREATE TABLE users (
        id INTEGER PRIMARY KEY NOT NULL,
        name TEXT NOT NULL,
        passwd BLOB NOT NULL,
        salt TEXT NOT NULL
);
