use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::models::Post;
use sea_orm::DatabaseConnection;
use sqlx::Pool;
use sqlx::Postgres;

use serde::{Deserialize, Serialize};
use time::{Duration, PrimitiveDateTime};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct NewPost {
    pub title: String,
    pub content: String,
}
#[derive(Debug, Serialize)]
pub struct PostWithAuthor {
    pub post: Post,
    pub author_name: String,
}
#[derive(Deserialize)]
pub struct RegisterStruct {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ReplyPost {
    pub reply_content: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}
#[derive(Serialize, Debug)]
pub enum ResponseResult {
    Ok,
    Err,
}

pub type SessionId = Uuid;
pub type SessionMap = Arc<RwLock<HashMap<Uuid, (i32, PrimitiveDateTime, Duration)>>>;
pub type ConnectionPool = DatabaseConnection;
