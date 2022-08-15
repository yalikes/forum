use crate::schema::users;
use serde::Serialize;

#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: String,
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
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