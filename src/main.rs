#[macro_use]
extern crate diesel;

use std::io;
use std::net::SocketAddr;
use std::{collections::HashMap, env};
use std::sync::{RwLock, Arc};

use axum::routing::{get_service};
use axum::{
    async_trait,
    body::StreamBody,
    extract::Form,
    extract::{Extension, FromRequest, RequestParts},
    headers::Cookie,
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    response::Html,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};

use tokio::io::AsyncReadExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use hyper::header;
use tokio_util::io::ReaderStream;

use tower_http::{services::ServeDir, trace::TraceLayer};

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use tera::{Context, Tera};

use serde::Deserialize;

use uuid::Uuid;

use dotenv::dotenv;

mod models;
mod schema;
mod utils;

use models::InsertableUser;
use utils::generate_salt_and_hash;

use crate::utils::check;

const SESSON_ID_COOKIE_NAME: &str = "session_id";
#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let sessions: HashMap<SessionId, (i32, f32)> = HashMap::new(); //session_id => (userId, expr time limit)
    let sessions:Arc<RwLock<HashMap<Uuid,(i32, f32)>>> = Arc::new(RwLock::new(sessions));
    let sessions_ptr = sessions.clone();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = Pool::builder()
        .max_size(1)
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap();
    let tera = match Tera::new("templates/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            println!("templates parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login).post(login_post))
        .route("/register", get(register).post(register_post))
        .fallback(get_service(ServeDir::new("./dist")).handle_error(handle_error))
        .layer(Extension((tera, pool)))
        .layer(Extension(sessions_ptr))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> Html<String> {
    use schema::users::dsl::{name, users};
    let my_user_names = users
        .select(name)
        .load::<String>(&pool.get().unwrap())
        .unwrap();
    let logined:bool;
    match may_user_id{
        UserIdFromSession::FoundUserId(_) => {logined = true}
        UserIdFromSession::NotFound => {logined=false}
    }
    let mut context = Context::new();
    context.insert("names", &my_user_names);
    context.insert("logined", &logined);
    tera.render("index.html", &context).unwrap().into()
}

async fn register() -> impl IntoResponse {
    let file = match tokio::fs::File::open("dist/register.html").await {
        Ok(file) => file,
        Err(err) => return Err((StatusCode::NOT_FOUND, format!("File not found: {}", err))),
    };

    let stream = ReaderStream::new(file);
    let body = StreamBody::new(stream);
    Ok(([(header::CONTENT_TYPE, "text/html")], body))
}

async fn login(
    may_user_id: UserIdFromSession,
    Extension((_, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> impl IntoResponse {
    use schema::users::dsl::{name, users};
    tracing::debug!("{:?}", may_user_id);
    if let UserIdFromSession::FoundUserId(user_id) = may_user_id {
        if let Ok(my_user_name) = users
            .find(user_id)
            .select(name)
            .get_result::<String>(&pool.get().unwrap())
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

    return Ok((StatusCode::OK, header, body));
}
#[derive(Deserialize)]
struct RegisterStruct {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginInfo {
    username: String,
    password: String,
}

async fn login_post(
    Form(login_info): Form<LoginInfo>,
    Extension((_, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
    Extension(sessions): Extension<Arc<RwLock<HashMap<Uuid,(i32, f32)>>>>,
) -> impl IntoResponse {
    use schema::users::dsl::{id, name, passwd, salt, users};
    let queryed_result = users
        .filter(name.eq(&login_info.username))
        .select((name, passwd, salt, id))
        .get_result::<(String, Vec<u8>, String, i32)>(&pool.get().unwrap());
    match queryed_result {
        Err(diesel::NotFound) => {
            return (
                StatusCode::NOT_FOUND,
                HeaderMap::new(),
                "username not found!".to_owned(),
            );
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                HeaderMap::new(),
                "database querry error occur".to_owned(),
            );
        }
        Ok(queryed_result) => {
            if check(
                &login_info.password,
                &queryed_result.2,
                queryed_result.1.try_into().unwrap_or_else(|v: Vec<u8>| {
                    panic!(
                        "Expected a Vec of length {} but got length of {}",
                        32,
                        v.len()
                    )
                }),
            ) {
                let session_id = Uuid::new_v4();
                let mut header = HeaderMap::new();
                header.insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str("text/html").unwrap(),
                );
                header.insert(
                    header::SET_COOKIE,
                    HeaderValue::from_str(
                        format!("{}={}", SESSON_ID_COOKIE_NAME, session_id).as_str(),
                    )
                    .unwrap(),
                );
                sessions.write().unwrap().insert(session_id, (queryed_result.3, 0.0));
                return (
                    StatusCode::ACCEPTED,
                    header,
                    format!("hello, {}", queryed_result.0),
                );
            }
            return (
                StatusCode::FORBIDDEN,
                HeaderMap::new(),
                "username or password not correct".to_owned(),
            );
        }
    };
}

async fn register_post(
    Form(register_info): Form<RegisterStruct>,
    Extension((_, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> impl IntoResponse {
    use schema::users::dsl::{name, users};

    if register_info.password.len() < 6 {
        // todo: flash user with a "password too short" message
    }

    let queryed_names = users
        .filter(name.eq(&register_info.username))
        .select(name)
        .get_result::<String>(&pool.get().unwrap());
    match queryed_names {
        Err(diesel::NotFound) => {
            // because this name previously didn't exists, it can be used to register new user.
            register_user(&register_info.username, &register_info.password, pool);
            return (StatusCode::OK, "register success!");
        }
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "database querry error occur",
            );
        }
        Ok(_) => {
            return (StatusCode::OK, "username already exists");
        }
    };
}

fn register_user(username: &str, password: &str, pool: Pool<ConnectionManager<SqliteConnection>>) {
    use schema::users::dsl::users;
    let (password_hash, salt) = generate_salt_and_hash(password);
    let new_user = InsertableUser {
        name: username.to_owned(),
        passwd: password_hash.to_vec(),
        salt: salt.iter().collect(),
    };
    diesel::insert_into(users)
        .values(&new_user)
        .execute(&pool.get().unwrap())
        .unwrap();
}

#[derive(Debug)]
enum UserIdFromSession {
    FoundUserId(i32),
    NotFound,
}

type SessionId = Uuid;

#[async_trait]
impl<B> FromRequest<B> for UserIdFromSession
where
    B: Send,
{
    type Rejection = (StatusCode, String);
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(sessions) = Extension::<Arc<RwLock<HashMap<Uuid,(i32, f32)>>>>::from_request(req)
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
                let user_id: i32 = match Uuid::parse_str(session_id_str) {
                    //TODO: parse to UUID
                    Ok(session_id) => match sessions.read().unwrap().get(&session_id) {
                        Some((uid, _)) => *uid,
                        None => {
                            return Ok(UserIdFromSession::NotFound);
                        }
                    }, // TODO: check id if is exists or is not expr
                    Err(_) => {
                        tracing::debug!("parse session_id error");
                        return Ok(UserIdFromSession::NotFound);
                    }
                };
                return Ok(UserIdFromSession::FoundUserId(user_id));
            }
        }
    }
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}