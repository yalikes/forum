use std::str::FromStr;

use axum::{extract::State, Json, debug_handler};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbErr, Set};
use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
use tracing::debug;

use crate::{
    entity::{floors, posts},
    helper::{ConnectionPool, SessionMap},
    models::Post,
    tools::now,
    user_utils::check_session_valid, app_state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    session_id: String,
    title: String,
    content: String,
}

#[derive(Debug, Serialize)]
pub enum CreatePostState {
    NeedLogin,
    InnerServerError,
    Success,
}

#[derive(Debug, Serialize)]
pub struct ResponseCreatePost {
    state: CreatePostState,
    info: Option<CreatePostInfo>,
}

#[derive(Debug, Serialize)]
pub struct CreatePostInfo {
    post_id: i32,
}

#[debug_handler(state = AppState)]
pub async fn post_create_post(
    State(sessions): State<SessionMap>,
    State(pool): State<ConnectionPool>,
    Json(request_info): Json<CreatePostRequest>,
) -> Json<ResponseCreatePost> {
    let user_id_and_time = get_user_id_info(
        sessions,
        uuid::Uuid::from_str(&request_info.session_id).unwrap(),
    );
    let user_id_and_time = match user_id_and_time {
        Some(id) => id,
        None => {
            return ResponseCreatePost {
                state: CreatePostState::NeedLogin,
                info: None,
            }
            .into();
        }
    };
    if !check_session_valid(user_id_and_time.1, user_id_and_time.2) {
        return ResponseCreatePost {
            state: CreatePostState::NeedLogin,
            info: None,
        }
        .into();
    }
    let now = now();
    let new_post =
        match insert_new_post(pool.clone(), user_id_and_time.0, request_info.title, now).await {
            Ok(p) => p,
            Err(e) => {
                return ResponseCreatePost {
                    state: CreatePostState::InnerServerError,
                    info: None,
                }
                .into();
            }
        };
    let new_floor = match insert_new_floor(
        &pool,
        new_post.id,
        user_id_and_time.0,
        request_info.content,
        now,
    )
    .await
    {
        Ok(f) => f,
        Err(e) => {
            return ResponseCreatePost {
                state: CreatePostState::InnerServerError,
                info: None,
            }
            .into();
        }
    };
    ResponseCreatePost {
        state: CreatePostState::Success,
        info: Some(CreatePostInfo {
            post_id: new_post.id,
        }),
    }
    .into()
}

async fn insert_new_post(
    pool: ConnectionPool,
    author_id: i32,
    title: String,
    time_now: PrimitiveDateTime,
) -> Result<posts::Model, DbErr> {
    let new_post = posts::ActiveModel {
        id: NotSet,
        author: Set(Some(author_id)),
        title: Set(title),
        post_create_time: Set(Some(time_now)),
    };
    new_post.insert(&pool).await
}

async fn insert_new_floor(
    pool: &ConnectionPool,
    post_id: i32,
    author_id: i32,
    content: String,
    time_now: PrimitiveDateTime,
) -> Result<floors::Model, DbErr> {
    let new_floor = floors::ActiveModel {
        id: NotSet,
        post_id: Set(Some((post_id))),
        floor_number: Set(1),
        author: Set(Some(author_id)),
        content: Set(content),
        floor_create_time: Set(Some(time_now)),
    };
    new_floor.insert(pool).await
}

pub fn get_user_id_info(
    sessions: SessionMap,
    session_id: uuid::Uuid,
) -> Option<(i32, PrimitiveDateTime, time::Duration)> {
    sessions.read().unwrap().get(&session_id).map(|e| *e)
}
