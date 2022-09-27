// @generated automatically by Diesel CLI.

diesel::table! {
    floor (id) {
        id -> Integer,
        post_id -> Integer,
        floor_number -> Integer,
        author -> Integer,
        content -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        author -> Integer,
        title -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        passwd -> Binary,
        salt -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    floor,
    posts,
    users,
);
