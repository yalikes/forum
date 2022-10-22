#[macro_use]
extern crate diesel;

use rand;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, env};

use axum::{
    async_trait,
    body::StreamBody,
    extract::Form,
    extract::{Extension, FromRequest, Path, RequestParts},
    headers::Cookie,
    http::{
        header::{HeaderMap, HeaderValue},
        StatusCode,
    },
    response::Html,
    response::IntoResponse,
    routing::{get, get_service},
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

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use dotenvy::dotenv;

mod models;
mod schema;
mod utils;

use models::{Floor, InsertablePost, InsertableUser, Post};
use utils::generate_salt_and_hash;

use crate::models::InsertableFloor;
use crate::utils::check;

const SESSON_ID_COOKIE_NAME: &str = "session_id";

type SessionMap = Arc<RwLock<HashMap<Uuid, (i32, f32)>>>;
type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let sessions: HashMap<SessionId, (i32, f32)> = HashMap::new(); //session_id => (userId, expr time limit)
    let sessions: SessionMap = Arc::new(RwLock::new(sessions));
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
        .route("/post/:post_id", get(get_post))
        .route("/post/:post_id/:page_id", get(get_post_with_page))
        .route("/newpost", get(newpost).post(newpost_post))
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
    Extension((tera, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> Html<String> {
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};
    let mut posts_with_author_name: Vec<PostWithAuthor> = Vec::new();
    let conn = &mut pool.get().unwrap();
    if let Ok(some_post) = posts
        .select((dsl_id, author_id, title))
        .limit(10)
        .get_results::<Post>(conn)
    {
        tracing::debug!("{:?}", &some_post);
        for post in some_post {
            let author_name = users
                .select(name)
                .find(post.author)
                .get_result::<String>(conn)
                .unwrap_or_else(|_| "unknown user".to_owned());
            posts_with_author_name.push(PostWithAuthor { post, author_name });
        }
    }

    let logined: bool = match may_user_id {
        UserIdFromSession::FoundUserId(_) => true,
        UserIdFromSession::NotFound => false,
    };
    let mut context = Context::new();
    context.insert("logined", &logined);
    context.insert("posts", &posts_with_author_name);
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
    Extension((_, pool)): Extension<(Tera, SqliteConnectionPool)>,
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

async fn get_post_with_page(
    Path((post_id, page)): Path<(i32, i32)>,
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> Html<String> {
    use schema::floor::dsl::{floor, floor_number, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};

    let post_inner: Post = match posts
        .find(post_id)
        .select((dsl_id, author_id, title))
        .first::<Post>(&mut pool.get().unwrap())
    {
        Ok(p) => p,
        Err(_) => {
            return "post not found".to_owned().into();
        }
    };

    let post: PostWithAuthor = match users
        .find(post_inner.author)
        .select(name)
        .first::<String>(&mut pool.get().unwrap())
    {
        Ok(author_name) => PostWithAuthor {
            post: post_inner,
            author_name: author_name,
        },
        Err(_) => PostWithAuthor {
            post: post_inner,
            author_name: "unknown".to_owned(),
        },
    };

    let floors = match floor
        .filter(post_id_dsl.eq(post.post.id))
        .order_by(floor_number)
        .limit(10)
        .offset(((if page < 0 { 0 } else { page - 1 }) * 10).into())
        .get_results::<Floor>(&mut pool.get().unwrap())
    {
        Ok(floors) => floors,
        Err(_) => {
            return "can't find floors".to_owned().into();
        }
    };

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("floors", &floors);
    context.insert(
        "logined",
        &(if let UserIdFromSession::FoundUserId(_) = may_user_id {
            true
        } else {
            false
        }),
    );
    tera.render("post.html", &context).unwrap().into()
}

async fn get_post(
    Path(post_id): Path<i32>,
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> Html<String> {
    use schema::floor::dsl::{floor, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};

    let post_inner: Post = match posts
        .find(post_id)
        .select((dsl_id, author_id, title))
        .first::<Post>(&mut pool.get().unwrap())
    {
        Ok(p) => p,
        Err(_) => {
            return "post not found".to_owned().into();
        }
    };

    let post: PostWithAuthor = match users
        .find(post_inner.author)
        .select(name)
        .first::<String>(&mut pool.get().unwrap())
    {
        Ok(author_name) => PostWithAuthor {
            post: post_inner,
            author_name: author_name,
        },
        Err(_) => PostWithAuthor {
            post: post_inner,
            author_name: "unknown".to_owned(),
        },
    };

    let floors = match floor
        .filter(post_id_dsl.eq(post.post.id))
        .limit(10)
        .get_results::<Floor>(&mut pool.get().unwrap())
    {
        Ok(floors) => floors,
        Err(_) => {
            return "can't find floors".to_owned().into();
        }
    };

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("floors", &floors);
    context.insert(
        "logined",
        &(if let UserIdFromSession::FoundUserId(_) = may_user_id {
            true
        } else {
            false
        }),
    );
    tera.render("post.html", &context).unwrap().into()
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

#[derive(Debug, Serialize)]
struct PostWithAuthor {
    pub post: Post,
    pub author_name: String,
}

async fn login_post(
    Form(login_info): Form<LoginInfo>,
    Extension((_, pool)): Extension<(Tera, SqliteConnectionPool)>,
    Extension(sessions): Extension<SessionMap>,
) -> impl IntoResponse {
    use schema::users::dsl::{id, name, passwd, salt, users};
    let queryed_result = users
        .filter(name.eq(&login_info.username))
        .select((name, passwd, salt, id))
        .get_result::<(String, Vec<u8>, String, i32)>(&mut pool.get().unwrap());
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

async fn register_post(
    Form(register_info): Form<RegisterStruct>,
    Extension((_, pool)): Extension<(Tera, SqliteConnectionPool)>,
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

fn register_user(username: &str, password: &str, pool: &SqliteConnectionPool) {
    use schema::users::dsl::users;
    let (password_hash, salt) = generate_salt_and_hash(password);
    let new_user = InsertableUser {
        name: username.to_owned(),
        passwd: password_hash.to_vec(),
        salt: salt.iter().collect(),
    };
    diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut pool.get().unwrap())
        .unwrap();
}

async fn newpost(may_user_id: UserIdFromSession) -> Html<String> {
    if let UserIdFromSession::FoundUserId(_) = may_user_id {
        let mut file = match tokio::fs::File::open("dist/newpost.html").await {
            Ok(file) => file,
            Err(_) => return "".to_owned().into(),
        };
        let mut body = String::new();
        file.read_to_string(&mut body).await.unwrap();
        return body.into();
    } else {
        return "please login".to_owned().into();
    }
}

async fn newpost_post(
    may_user_id: UserIdFromSession,
    Form(submited_post): Form<NewPost>,
    Extension((_, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> Html<String> {
    use schema::floor::dsl::floor;
    use schema::posts::dsl::{extra, id as dsl_post_id, posts};
    let user_id = match may_user_id {
        UserIdFromSession::FoundUserId(id) => id,
        UserIdFromSession::NotFound => return "please login".to_owned().into(),
    };

    //diesel currently not support returning for sqlite, i have to use a wired rourting to keep thing work
    // 1. insert a row with unique extra property
    // 2. find row with this property
    // 3. get id and remove this property
    let extra_id: i32 = rand::random();
    diesel::insert_into(posts)
        .values(InsertablePost {
            author: user_id,
            title: submited_post.title,
            extra: extra_id,
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();
    let post_id = posts
        .filter(extra.eq(extra_id))
        .select(dsl_post_id)
        .get_result::<i32>(&mut pool.get().unwrap()).unwrap();
    diesel::update(posts.find(post_id))
        .set(extra.eq(0)).execute(&mut pool.get().unwrap()).unwrap();
    diesel::insert_into(floor)
        .values(InsertableFloor{
            post_id,
            floor_number:1,
            author:user_id,
            content: submited_post.content
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();
    "create post sucess!".to_owned().into()
}

#[derive(Deserialize)]
struct NewPost {
    title: String,
    content: String,
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

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
