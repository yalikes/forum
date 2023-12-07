use std::borrow::BorrowMut;

use crate::diesel::RunQueryDsl;
use crate::models::{InsertableUser, User};
use crate::utils::generate_salt_and_hash;
use axum::extract::State;
use axum::Json;
use diesel::insert_into;
use diesel::{ExpressionMethods, QueryDsl};
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::debug;
use uuid::Uuid;

use crate::helper::{ConnectionPool, ResponseResult, SessionId, SessionMap};
use crate::schema;

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
c
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
    use schema::users::dsl::{name, users};
    // TODO: add more return infomation
    if register_info.password.len() < 9 {
        return ResponseRegisterUser {
            state: ResponseRegisterUserState::PasswordTooShort,
            info: None,
        }
        .into();
    }
    let user_name_exists = users
        .filter(name.eq(&register_info.user_name))
        .limit(1)
        .count()
        .get_result::<i64>(&mut pool.get().unwrap())
        .unwrap_or(0)
        .is_positive();
    if user_name_exists {
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
    use schema::users::dsl::{id as dsl_id, users};
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
    insert_into(users)
        .values(&user)
        .get_result(pool.get().unwrap().borrow_mut())
        .unwrap()
}
