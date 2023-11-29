use crate::schema::{users, posts, floors};
use serde::Serialize;
use time::PrimitiveDateTime;

#[derive(Queryable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: Vec<u8>,
    pub user_create_time: Option<PrimitiveDateTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableUser {
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: Vec<u8>,
    pub user_create_time: Option<PrimitiveDateTime>,
}

#[derive(Queryable, Debug, Serialize)]
#[diesel(table_name = post)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub author: Option<i32>,
    pub title: String,
}

#[derive(Insertable, Debug, Serialize)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertablePost {
    pub author: i32,
    pub title: String,
}

#[derive(Queryable, Debug, Serialize)]
#[diesel(table_name = floors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Floor {
    pub id: i32,
    pub post_id: Option<i32>,
    pub floor_number: i32,
    pub author: Option<i32>,
    pub content: String,
    pub floor_create_time: Option<PrimitiveDateTime>,
}

#[derive(Insertable, Debug, Serialize)]
#[diesel(table_name = floors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableFloor {
    pub post_id: i32,
    pub floor_number: i32,
    pub author: i32,
    pub content: String,
}
