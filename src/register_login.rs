use axum::{
    body::StreamBody,
    extract::{Form, State},
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    response::IntoResponse,
};

use tokio::io::AsyncReadExt;

use hyper::header;
use tokio_util::io::ReaderStream;

use diesel::{QueryDsl, RunQueryDsl};
use diesel::expression_methods::ExpressionMethods;
use uuid::Uuid;

use crate::constants::SESSON_ID_COOKIE_NAME;
use crate::helper::*;
use crate::models::InsertableUser;
use crate::schema;
use crate::utils::{check, generate_salt_and_hash};

pub async fn register() -> impl IntoResponse {
    let file = match tokio::fs::File::open("dist/register.html").await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    Ok(([(header::CONTENT_TYPE, "text/html")], body))
}

pub async fn login(
    may_user_id: UserIdFromSession,
    State(pool): State<ConnectionPool>
) -> impl IntoResponse {
    use schema::users::dsl::{name, users};
    tracing::debug!("{:?}", may_user_id);
    if let UserIdFromSession::FoundUserId(user_id) = may_user_id {
        if let Ok(my_user_name) = users
            .find(user_id)
            .select(name)
            .get_result::<String>(&mut pool.get().unwrap())
        {
            return Ok((StatusCode::OK, HeaderMap::new(), my_user_name));
        }
    }
    let mut header = HeaderMap::new();
    let mut file = match tokio::fs::File::open("dist/login.html").await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };
    // tracing::debug!("inside login() session_id {:?}", may_user_id);
    header.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str("text/html").unwrap(),
    );
    let mut body = String::new();
    file.read_to_string(&mut body).await.unwrap();

    Ok((StatusCode::OK, header, body))
}

pub async fn login_post(
    State(pool): State<ConnectionPool>,
    State(sessions): State<SessionMap>,
    Form(login_info): Form<LoginInfo>,
) -> impl IntoResponse {
    use schema::users::dsl::{id, name, passwd, salt, users};
    let queryed_result = users
        .filter(name.eq(&login_info.username))
        .select((name, passwd, salt, id))
        .get_result::<(String, Vec<u8>, Vec<u8>, i32)>(&mut pool.get().unwrap());
    match queryed_result {
        Err(diesel::NotFound) => (
            StatusCode::NOT_FOUND,
            HeaderMap::new(),
            "username not found!".to_owned(),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::new(),
            "database querry error occur".to_owned(),
        ),
        Ok(queryed_result) => {
            if check(
                &login_info.password,
                &queryed_result.2,
                &queryed_result.1,
            ) {
                let session_id = Uuid::new_v4();
                let mut header = HeaderMap::new();
                header.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("text/html; charset=utf-8").unwrap(),
                );
                header.insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(
                        format!("{}={}", SESSON_ID_COOKIE_NAME, session_id).as_str(),
                    )
                    .unwrap(),
                );
                sessions
                    .write()
                    .unwrap()
                    .insert(session_id, (queryed_result.3, 0.0));
                return (
                    StatusCode::ACCEPTED,
                    header,
                    format!("hello, {}", queryed_result.0),
                );
            }
            (
                StatusCode::FORBIDDEN,
                HeaderMap::new(),
                "username or password not correct".to_owned(),
            )
        }

    }
}

pub async fn register_post(
    State(pool): State<ConnectionPool>,
    Form(register_info): Form<RegisterStruct>,
) -> impl IntoResponse {
    use schema::users::dsl::{name, users};

    if register_info.password.len() < 6 {
        // todo: flash user with a "password too short" message
    }

    let queryed_names = users
        .filter(name.eq(&register_info.username))
        .select(name)
        .get_result::<String>(&mut pool.get().unwrap());
    match queryed_names {
        Err(diesel::NotFound) => {
            // because this name previously didn't exists, it can be used to register new user.
            register_user(&register_info.username, &register_info.password, &pool);
            (StatusCode::OK, "register success!")
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "database querry error occur",
        ),
        Ok(_) => (StatusCode::OK, "username already exists"),
    }
}

fn register_user(username: &str, password: &str, pool: &ConnectionPool) {
    use schema::users::dsl::users;
    let (password_hash, salt) = generate_salt_and_hash(password);
    let new_user = InsertableUser {
        name: username.to_owned(),
        passwd: password_hash.to_vec(),
        salt: salt.to_vec(),
    };
    diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut pool.get().unwrap())
        .unwrap();
}
