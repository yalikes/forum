use core::panic;
use std::borrow::BorrowMut;

use crate::models::{InsertableUser, User};
use crate::sqlx_helper::SqlxI32;
use crate::utils::generate_salt_and_hash;
use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use tracing::debug;
use uuid::Uuid;

use crate::helper::{ConnectionPool, ResponseResult, SessionId, SessionMap};

#[derive(Serialize, Debug)]
pub struct RegisterUserInfo {
    id: i32,
    session_id: String,
}

#[derive(Debug, Serialize)]
pub enum ResponseRegisterUserState {
    Success,
    UserNameExists,
    PasswordTooShort,
    InnerServerError,
}

#[derive(Debug, Serialize)]
pub struct ResponseRegisterUser {
    state: ResponseRegisterUserState,
    info: Option<RegisterUserInfo>,
}
#[derive(Deserialize, Debug)]
pub struct RequestRegisterUser {
    user_name: String,
    password: String,
}

pub async fn register_user(
    State(sessions): State<SessionMap>,
    State(pool): State<ConnectionPool>,
    Json(register_info): Json<RequestRegisterUser>,
) -> Json<ResponseRegisterUser> {
    // TODO: add more return infomation
    // if register_info.password.len() < 9 {
    //     return ResponseRegisterUser {
    //         state: ResponseRegisterUserState::PasswordTooShort,
    //         info: None,
    //     }
    //     .into();
    // }
    // let user_name_exists =
    //     match sqlx::query_as::<_, SqlxI32>("SELECT COUNT(*) from users WHERE name = $1 LIMIT 1")
    //         .bind(&register_info.user_name)
    //         .fetch_one(&pool)
    //         .await
    //     {
    //         Err(e) => {
    //             debug!("{:?}", e);
    //             return ResponseRegisterUser {
    //                 state: ResponseRegisterUserState::InnerServerError,
    //                 info: None,
    //             }
    //             .into();
    //         }
    //         Ok(i) => i.val.is_positive(),
    //     };

    // if user_name_exists {
    //     return ResponseRegisterUser {
    //         state: ResponseRegisterUserState::UserNameExists,
    //         info: None,
    //     }
    //     .into();
    // }

    // let user_session_id = Uuid::new_v4();
    // let user_id = insert_user(pool, &register_info.user_name, &register_info.password)
    //     .await
    //     .id;
    // sessions
    //     .write()
    //     .unwrap()
    //     .insert(user_session_id, (user_id, 24.0 * 60.0 * 60.0));
    // ResponseRegisterUser {
    //     state: ResponseRegisterUserState::Success,
    //     info: Some(RegisterUserInfo {
    //         id: user_id,
    //         session_id: user_session_id.to_string(),
    //     }),
    // }
    // .into()
    panic!()
}

pub async fn insert_user(pool: ConnectionPool, name: &str, passwd: &str) -> User {
    let (passwd, salt) = generate_salt_and_hash(&passwd);
    let current_time = time::OffsetDateTime::now_utc();
    let user = InsertableUser {
        name: name.to_string(),
        passwd: passwd.to_vec(),
        salt: salt.to_vec(),
        user_create_time: Some(PrimitiveDateTime::new(
            current_time.date(),
            current_time.time(),
        )),
    };
    sqlx::query_as::<_, User>(
        "INSERT INTO users(name, passwd, salt, user_create_time) VALUES ($1, $2, $3, $4)",
    )
    .bind(user.name)
    .bind(user.passwd)
    .bind(user.salt)
    .bind(user.user_create_time)
    .fetch_one(&pool)
    .await
    .unwrap()
}
