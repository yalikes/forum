use crate::schema::{users, posts, floor};
use serde::Serialize;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
pub struct InsertableUser {
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: String,
}

#[derive(Queryable, Debug, Serialize)]
pub struct Post {
    pub id: i32,
    pub author: i32,
    pub title: String,
}

#[derive(Insertable, Debug, Serialize)]
#[diesel(table_name = posts)]
pub struct InsertablePost {
    pub author: i32,
    pub title: String,
    pub extra: i32
}

#[derive(Queryable, Debug, Serialize)]
pub struct Floor {
    pub id: i32,
    pub post_id: i32,
    pub floor_number: i32,
    pub author: i32,
    pub content: String,
}

#[derive(Insertable, Debug, Serialize)]
#[diesel(table_name = floor)]
pub struct InsertableFloor {
    pub post_id: i32,
    pub floor_number: i32,
    pub author: i32,
    pub content: String,
}
