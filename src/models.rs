use serde::Serialize;
use sqlx::prelude::FromRow;
use sqlx::types::time::PrimitiveDateTime;

pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: Vec<u8>,
    pub user_create_time: Option<PrimitiveDateTime>,
}

pub struct InsertableUser {
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: Vec<u8>,
    pub user_create_time: Option<PrimitiveDateTime>,
}

#[derive(Debug, Serialize,FromRow)]
pub struct Post {
    pub id: i32,
    pub author: Option<i32>,
    pub title: String,
    pub post_create_time: Option<PrimitiveDateTime>,
}


#[derive(Debug, Serialize)]
pub struct Floor {
    pub id: i32,
    pub post_id: Option<i32>,
    pub floor_number: i32,
    pub author: Option<i32>,
    pub content: String,
    pub floor_create_time: Option<PrimitiveDateTime>,
}

pub struct InsertableFloor {
    pub post_id: i32,
    pub floor_number: i32,
    pub author: i32,
    pub content: String,
}
