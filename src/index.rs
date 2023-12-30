use core::panic;

use axum::extract::State;
use axum::response::Json;
use sea_orm::{DbErr, EntityTrait, QueryFilter, QuerySelect};
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
    match recent_posts(&pool).await {
        Ok(posts) => {
            for post in posts {
                let author_name = get_user_name_with_default(post.author, &pool.clone()).await;
                let p = Post {
                    id: post.id,
                    author: post.author,
                    title: post.title,
                    post_create_time: post.post_create_time,
                };
                posts_with_author_name.push(PostWithAuthor {
                    post: p,
                    author_name,
                });
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

async fn recent_posts(pool: &ConnectionPool) -> Result<Vec<crate::entity::posts::Model>, DbErr> {
    use crate::entity::posts::Entity as Posts;
    Posts::find().limit(10).all(pool).await
}
