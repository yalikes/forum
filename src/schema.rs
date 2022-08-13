table! {
    posts (id) {
        id ->Integer,
        author -> Integer,
        title -> Text,
    }
}

table! {
    users (id) {
        id ->Integer,
        name -> Text,
        passwd -> Binary,
        salt -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    posts,
    users,
);
