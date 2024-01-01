use core::panic;
use std::borrow::BorrowMut;

use crate::models::{InsertableUser, User};
use crate::tools::now;
use crate::utils::{check, generate_salt_and_hash};
use axum::extract::State;
use axum::Json;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::PrimitiveDateTime;
use time::format_description::modifier::Day;
use time::Duration;
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
    let current_time = time::OffsetDateTime::now_utc();
    sessions.write().unwrap().insert(
        user_session_id,
        (
            user_id,
            PrimitiveDateTime::new(current_time.date(), current_time.time()),
            Duration::DAY,
        ),
    );
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

#[derive(Debug, Deserialize)]
pub enum LoginType {
    UserName,
    UserId,
}
#[derive(Debug, Deserialize)]
pub struct RequestUserLogin {
    user_name: Option<String>,
    user_id: Option<i32>,
    password: String,
    login_type: LoginType,
}

#[derive(Debug, Serialize)]
pub enum UserLoginState {
    Success,
    Error,
    UserNameNotExists,
    WrongPassword,
    InnerServerError,
}

#[derive(Debug, Serialize)]
pub struct ResponseUserLogin {
    state: UserLoginState,
    info: Option<UserLoginInfo>,
}

#[derive(Debug, Serialize)]
pub struct UserLoginInfo {
    session_id: String,
}

pub async fn login(
    State(sessions): State<SessionMap>,
    State(pool): State<ConnectionPool>,
    Json(request_info): Json<RequestUserLogin>,
) -> Json<ResponseUserLogin> {
    let login_type = request_info.login_type;
    let user_id = match login_type {
        LoginType::UserId => {
            let user_id = request_info.user_id;
            if user_id.is_none() {
                return ResponseUserLogin {
                    state: UserLoginState::Error,
                    info: None,
                }
                .into();
            }
            user_id.unwrap()
        }
        LoginType::UserName => {
            let user_name = request_info.user_name;
            if user_name.is_none() {
                return ResponseUserLogin {
                    state: UserLoginState::Error,
                    info: None,
                }
                .into();
            }
            let user_name = user_name.unwrap();
            let may_user_id = match get_user_id_from_user_name(&pool, &user_name).await {
                Ok(u) => u,
                Err(e) => {
                    return ResponseUserLogin {
                        state: UserLoginState::InnerServerError,
                        info: None,
                    }
                    .into();
                }
            };
            match may_user_id {
                Some(u) => u,
                None => {
                    return ResponseUserLogin {
                        state: UserLoginState::UserNameNotExists,
                        info: None,
                    }
                    .into();
                }
            }
        }
    };
    let (password, salt) = match get_password_and_salt(&pool, user_id).await {
        Ok(may_passwd_salt) => match may_passwd_salt {
            Some(p_s) => p_s,
            None => {
                return ResponseUserLogin {
                    state: UserLoginState::InnerServerError,
                    info: None,
                }
                .into();
            }
        },
        Err(e) => {
            return ResponseUserLogin {
                state: UserLoginState::InnerServerError,
                info: None,
            }
            .into();
        }
    };
    if !check(&request_info.password, &salt, &password) {
        return ResponseUserLogin {
            state: UserLoginState::WrongPassword,
            info: None,
        }
        .into();
    }
    let new_uuid = uuid::Uuid::new_v4();
    sessions
        .write()
        .unwrap()
        .insert(new_uuid, (user_id, now(), time::Duration::DAY));
    ResponseUserLogin {
        state: UserLoginState::Success,
        info: Some(UserLoginInfo {
            session_id: new_uuid.to_string(),
        }),
    }
    .into()
}

pub async fn get_user_id_from_user_name(
    pool: &ConnectionPool,
    user_name: &str,
) -> Result<Option<i32>, DbErr> {
    use crate::entity::users;
    users::Entity::find()
        .filter(users::Column::Name.eq(user_name))
        .one(pool)
        .await
        .map(|m| m.map(|u| u.id))
}

pub async fn get_password_and_salt(
    pool: &ConnectionPool,
    user_id: i32,
) -> Result<Option<([u8; 32], [u8; 32])>, DbErr> {
    use crate::entity::users;
    users::Entity::find_by_id(user_id).one(pool).await.map(|m| {
        m.map(|u| {
            let password: [u8; 32] = u.passwd.try_into().unwrap();
            let salt: [u8; 32] = u.salt.try_into().unwrap();
            (password, salt)
        })
    })
}
