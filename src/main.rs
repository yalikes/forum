#[macro_use]
extern crate diesel;

use std::env;

use axum::{
    body::StreamBody, extract::Extension, extract::Form, http::StatusCode, response::Html,
    response::IntoResponse, routing::get, Router,
};
use hyper::header;
use tokio_util::io::ReaderStream;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection};
use tera::{Context, Tera};

use serde::Deserialize;

use dotenv::dotenv;

mod models;
mod schema;
mod utils;

use models::InsertableUser;
use utils::generate_salt_and_hash;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");
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
        .route("/login", get())
        .route("/register", get(register).post(register_post))
        .layer(Extension((tera, pool)));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index(
    Extension((tera, pool)): Extension<(Tera, Pool<ConnectionManager<SqliteConnection>>)>,
) -> Html<String> {
    use schema::users::dsl::{name, users};
    let my_user_names = users
        .select(name)
        .load::<String>(&pool.get().unwrap())
        .unwrap();
    let mut context = Context::new();
    context.insert("names", &my_user_names);
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

#[derive(Deserialize)]
struct RegisterStruct {
    username: String,
    password: String,
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

async fn login_handler(){

}