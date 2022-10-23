#[macro_use]
extern crate diesel;

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, env};

use axum::{
    Router,
    extract::{Extension},
    http::{
        StatusCode,
    },
    response::IntoResponse,
    routing::{get, get_service},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


use tower_http::{services::ServeDir, trace::TraceLayer};

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use tera::{Tera};

use dotenvy::dotenv;

mod models;
mod schema;
mod utils;
mod helper;
mod index;
mod register_login;
mod get_post;
mod new_post;
mod constants;

use helper::*;

use crate::index::index;
use crate::register_login::*;
use new_post::*;
use get_post::*;

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

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
