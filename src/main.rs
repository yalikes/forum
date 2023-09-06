#[macro_use]
extern crate diesel;

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, env};

use axum::http::HeaderValue;
use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service, post},
    Router,
};

use hyper::Method;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use dotenvy::dotenv;

mod app_state;
mod constants;
mod get_post;
mod helper;
mod index;
mod models;
mod new_post;
mod register_login;
mod schema;
mod utils;

use helper::*;

use crate::index::index;
use crate::register_login::*;
use app_state::*;
use get_post::*;
use new_post::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");
    let sessions: HashMap<SessionId, (i32, f32)> = HashMap::new(); //session_id => (userId, expr time limit)
    let sessions: SessionMap = Arc::new(RwLock::new(sessions));

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let pool = Pool::builder()
        .max_size(6)
        .build(ConnectionManager::<PgConnection>::new(database_url))
        .unwrap();

    let state = AppState {
        sessions: sessions,
        database_conn_pool: pool,
    };

    let app = Router::new()
        .route("/post/recent", get(|| async {}))
        .route("/post/get/:post_id", get(|| async {}))
        .route("/account/create", get(|| async {}))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // TODO: set allow origin
                .allow_methods([Method::GET, Method::POST]),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        // .with_graceful_shutdown(async move {
            //TODO: implement graceful shutdown
        // })
        .await
        .unwrap();
}
