table! {
    posts (id) {
        id -> Nullable<Integer>,
        author -> Integer,
        title -> Text,
    }
}

table! {
    users (id) {
        id -> Nullable<Integer>,
        name -> Text,
        passwd -> Binary,
        salt -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
