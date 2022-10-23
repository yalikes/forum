use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    headers::Cookie,
    http::StatusCode,
    TypedHeader,
};

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use serde::{Serialize, Deserialize};

use uuid::Uuid;
use crate::constants::SESSON_ID_COOKIE_NAME;
use crate::models::Post;

#[derive(Deserialize)]
pub struct NewPost {
    pub title: String,
    pub content: String,
}

#[derive(Debug)]
pub enum UserIdFromSession {
    FoundUserId(i32),
    NotFound,
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
pub struct LoginInfo {
    pub username: String,
    pub password: String,
}
pub type SessionId = Uuid;
pub type SessionMap = Arc<RwLock<HashMap<Uuid, (i32, f32)>>>;
pub type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, String);
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(sessions) =
            Extension::<Arc<RwLock<HashMap<Uuid, (i32, f32)>>>>::from_request(req)
                .await
                .expect("sessions extension missing");
        let cookie = Option::<TypedHeader<Cookie>>::from_request(req)
            .await
            .unwrap();
        tracing::debug!("{}", format!("{:?}", &cookie));
        let session_cookie = cookie
            .as_ref()
            .and_then(|cookie| cookie.get(SESSON_ID_COOKIE_NAME));
        match session_cookie {
            None => {
                return Ok(UserIdFromSession::NotFound);
            }
            Some(session_id_str) => {
                tracing::debug!("found session_id: {}", session_id_str);
                tracing::debug!("sessions is {:?}", sessions);

                let user_id: i32 = if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                    match sessions.read().unwrap().get(&session_id) {
                        Some((uid, _)) => *uid,
                        None => {
                            return Ok(UserIdFromSession::NotFound);
                        } // TODO: check expr time
                    }
                } else {
                    tracing::debug!("parse session_id error");
                    return Ok(UserIdFromSession::NotFound);
                };
                return Ok(UserIdFromSession::FoundUserId(user_id));
            }
        }
    }
}
