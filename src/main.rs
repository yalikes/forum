#![allow(unused)]
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, env};
use std::time::Duration;

use axum::extract::MatchedPath;
use axum::{
    http::Request,
    routing::{get, post},
    Router,
};

use hyper::Method;
use tracing::info_span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use dotenvy::dotenv;

use sqlx::postgres::PgPoolOptions;
use sea_orm::{ConnectOptions, Database};

mod account_tools;
mod app_state;
mod constants;
mod get_post;
mod helper;
mod index;
mod models;
mod new_post;
mod register_login;
mod sqlx_helper;
mod utils;
mod entity;

use helper::*;

use crate::index::get_recent_post;
use crate::register_login::register_user;
use app_state::*;
use get_post::*;

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

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);

    
    let db = Database::connect(opt).await.expect("failed to connect database");

    let state = AppState {
        sessions: sessions,
        database_conn_pool: db,
    };

    let app = Router::new()
        .route("/post/recent", get(get_recent_post))
        .route("/post/get/:post_id", get(get_post))
        .route("/post/get/floor/:post_id", get(get_floors))
        .route("/account/create", post(register_user))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any) // TODO: set allow origin
                .allow_headers(Any)
                .allow_methods([Method::GET, Method::POST]),
        );

    let addr = SocketAddr::new(IpAddr::from(Ipv6Addr::UNSPECIFIED), 3000);
    tracing::debug!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        // .with_graceful_shutdown(async move {
        //TODO: implement graceful shutdown
        // })
        .await
        .unwrap();
}
