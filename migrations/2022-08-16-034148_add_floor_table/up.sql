-- Your SQL goes here
CREATE TABLE floor(
        id INTEGER PRIMARY KEY,
        post_id INTEGER NOT NULL,
        floor_number INTEGER NOT NULL,
        author INTEGER NOT NULL,
        content TEXT NOT NULL
);
