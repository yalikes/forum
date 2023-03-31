use axum::extract::FromRef;

use crate::helper::{SessionMap, ConnectionPool};


#[derive(Clone)]
pub struct AppState {
    pub sessions: SessionMap,
    pub database_conn_pool: ConnectionPool,
}

impl FromRef<AppState> for SessionMap {
    fn from_ref(input: &AppState) -> Self {
        input.sessions.clone()
    }
}

impl FromRef<AppState> for ConnectionPool {
    fn from_ref(input: &AppState) -> Self {
        input.database_conn_pool.clone()
    }
}