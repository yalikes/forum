use core::panic;

use axum::extract::State;
use axum::response::Json;
use serde::Serialize;
use sqlx::prelude::FromRow;
use tracing::debug;

use crate::account_tools::{get_user_name, get_user_name_with_default};
use crate::helper::ResponseResult;
use crate::helper::*;
use crate::models::Post;

#[derive(Debug, Serialize)]
pub struct ResponseGetRecentPost {
    state: ResponseResult,
    info: Option<ResponseGetRecentPostInfo>,
}

#[derive(Debug, Serialize)]
pub struct ResponseGetRecentPostInfo {
    posts: Vec<PostWithAuthor>,
}

pub async fn get_recent_post(State(pool): State<ConnectionPool>) -> Json<ResponseGetRecentPost> {
    let mut posts_with_author_name: Vec<PostWithAuthor> = Vec::new();
    match sqlx::query_as::<_, Post>("SELECT * FROM posts LIMIT 10")
        .fetch_all(&pool)
        .await
    {
        Ok(posts) => {
            for post in posts {
                let author_name = get_user_name_with_default(post.author, pool.clone()).await;
                posts_with_author_name.push(PostWithAuthor { post, author_name });
            }
        }
        Err(e) => {
            debug!("{}", e);
        }
    }
    ResponseGetRecentPost {
        state: ResponseResult::Ok,
        info: Some(ResponseGetRecentPostInfo {
            posts: posts_with_author_name,
        }),
    }
    .into()
}
