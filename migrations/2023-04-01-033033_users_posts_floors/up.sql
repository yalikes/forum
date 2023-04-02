-- Your SQL goes here
DROP TABLE IF EXISTS users ;
DROP TABLE IF EXISTS posts ;
DROP TABLE IF EXISTS floors ;

CREATE TABLE users(
        id integer PRIMARY KEY,
        name text NOT NULL,
        passwd bytea,
        salt TEXT NOT NULL
);

CREATE TABLE posts(
        id integer PRIMARY KEY,
        author integer REFERENCES users(id),
        title text NOT NULL
);
CREATE TABLE floors(
        id integer PRIMARY KEY,
        post_id integer REFERENCES posts(id),
        floor_number INTEGER NOT NULL,
        author integer REFERENCES users(id),
        content text NOT NULL
);
