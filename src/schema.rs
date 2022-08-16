table! {
    floor (id) {
        id -> Integer,
        post_id -> Integer,
        floor_number -> Integer,
        author -> Integer,
        content -> Text,
    }
}

table! {
    posts (id) {
        id -> Integer,
        author -> Integer,
        title -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        name -> Text,
        passwd -> Binary,
        salt -> Text,
    }
}

allow_tables_to_appear_in_same_query!(floor, posts, users,);
