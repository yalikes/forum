use axum::extract::FromRef;

use crate::helper::{SessionMap, SqliteConnectionPool};


#[derive(Clone)]
pub struct AppState {
    pub sessions: SessionMap,
    pub database_conn_pool: SqliteConnectionPool,
}

impl FromRef<AppState> for SessionMap {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

impl FromRef<AppState> for SqliteConnectionPool {
    fn from_ref(input: &AppState) -> Self {
        input.database_conn_pool.clone()
    }
}