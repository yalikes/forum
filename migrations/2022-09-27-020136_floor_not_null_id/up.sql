-- Your SQL goes here
DROP TABLE IF EXISTS floor;
CREATE TABLE floor(
        id INTEGER PRIMARY KEY NOT NULL,
        post_id INTEGER NOT NULL,
        floor_number INTEGER NOT NULL,
        author INTEGER NOT NULL,
        content TEXT NOT NULL
);
