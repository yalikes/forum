-- Your SQL goes here
DROP TABLE IF EXISTS floors ;
DROP TABLE IF EXISTS posts ;
DROP TABLE IF EXISTS users ;

CREATE TABLE users(
        id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        name text NOT NULL,
        passwd bytea,
        salt TEXT NOT NULL
);

CREATE TABLE posts(
        id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        author integer REFERENCES users(id),
        title text NOT NULL
);
CREATE TABLE floors(
        id integer GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
        post_id integer REFERENCES posts(id),
        floor_number INTEGER NOT NULL,
        author integer REFERENCES users(id),
        content text NOT NULL
);
