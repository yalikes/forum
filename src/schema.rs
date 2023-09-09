// @generated automatically by Diesel CLI.

diesel::table! {
    floors (id) {
        id -> Int4,
        post_id -> Int4,
        floor_number -> Int4,
        author -> Int4,
        content -> Text,
        floor_create_time -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        author -> Int4,
        title -> Text,
        post_create_time ->Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        name -> Text,
        passwd -> Bytea,
        salt -> Bytea,
        user_create_time -> Timestamp,
    }
}

diesel::joinable!(floors -> posts (post_id));
diesel::joinable!(floors -> users (author));
diesel::joinable!(posts -> users (author));

diesel::allow_tables_to_appear_in_same_query!(
    floors,
    posts,
    users,
);
