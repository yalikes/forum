use core::panic;
use std::borrow::BorrowMut;

use crate::models::{InsertableUser, User};
use crate::sqlx_helper::SqlxI32;
use crate::utils::generate_salt_and_hash;
use axum::extract::State;
use axum::Json;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
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
    if register_info.password.len() < 9 {
        return ResponseRegisterUser {
            state: ResponseRegisterUserState::PasswordTooShort,
            info: None,
        }
        .into();
    }
    let user_name_exists = check_user_name_exists(&pool, &register_info.user_name).await;
    if user_name_exists.is_err() {
        return ResponseRegisterUser {
            state: ResponseRegisterUserState::InnerServerError,
            info: None,
        }
        .into();
    }
    if user_name_exists.unwrap() {
        return ResponseRegisterUser {
            state: ResponseRegisterUserState::UserNameExists,
            info: None,
        }
        .into();
    }

    let user_session_id = Uuid::new_v4();
    let user_id = insert_user(pool, &register_info.user_name, &register_info.password)
        .await
        .id;
    sessions
        .write()
        .unwrap()
        .insert(user_session_id, (user_id, 24.0 * 60.0 * 60.0));
    ResponseRegisterUser {
        state: ResponseRegisterUserState::Success,
        info: Some(RegisterUserInfo {
            id: user_id,
            session_id: user_session_id.to_string(),
        }),
    }
    .into()
}

pub async fn insert_user(pool: ConnectionPool, name: &str, passwd: &str) -> User {
    let (passwd, salt) = generate_salt_and_hash(&passwd);
    let current_time = time::OffsetDateTime::now_utc();

    use crate::entity::users;
    let user = users::ActiveModel {
        id: NotSet,
        name: Set(name.to_owned()),
        passwd: Set(passwd.to_vec()),
        salt: Set(salt.to_vec()),
        user_create_time: Set(Some(PrimitiveDateTime::new(
            current_time.date(),
            current_time.time(),
        ))),
    };
    let user = user.insert(&pool).await.unwrap();
    return User {
        id: user.id,
        name: user.name,
        passwd: user.passwd,
        salt: user.salt,
        user_create_time: user.user_create_time,
    };
}

pub async fn check_user_name_exists(pool: &ConnectionPool, name: &str) -> Result<bool, DbErr> {
    use crate::entity::users;
    users::Entity::find()
        .filter(users::Column::Name.eq(""))
        .count(pool)
        .await
        .map(|c| c != 0)
}
