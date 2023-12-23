use std::vec;

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::account_tools::get_user_name;
use crate::account_tools::get_user_name_with_default;
use crate::constants;
use crate::helper;
use crate::helper::*;
use crate::models::Floor;
use crate::models::Post;

#[derive(Debug, Serialize)]
pub struct ResponseGetPost {
    state: ResponseResult,
    info: Option<PostInfo>,
}

#[derive(Debug, Serialize)]
pub struct PostInfo {
    title: String,
    author: String,
    author_id: i32,
    floor_num: u32,
}

pub async fn get_post(
    Path(post_id): Path<i32>,
    State(pool): State<ConnectionPool>,
) -> Json<ResponseGetPost> {
    panic!();
    let post_inner: Post = match sqlx::query_as("SELECT * FROM posts").fetch_one(&pool).await {
        Ok(p) => p,
        Err(_) => {
            return ResponseGetPost {
                state: ResponseResult::Err,
                info: None,
            }
            .into();
        }
    };
    let author_name = get_user_name_with_default(post_inner.author, pool.clone()).await;
    let post: PostWithAuthor = PostWithAuthor {
        post: post_inner,
        author_name,
    };
    #[derive(FromRow)]
    struct FloorNum {
        floor_num: i32,
    }
    let floor_num = sqlx::query_as::<_, FloorNum>("SELECT COUNT(id) FROM floors WHERE post_id = ?")
        .bind(post_id)
        .fetch_one(&pool)
        .await
        .unwrap_or(FloorNum { floor_num: 0 });

    ResponseGetPost {
        state: ResponseResult::Ok,
        info: Some(PostInfo {
            title: post.post.title,
            author: post.author_name,
            author_id: post.post.author.unwrap(),
            floor_num: floor_num.floor_num as u32,
        }),
    }
    .into()
}

#[derive(Deserialize)]
pub struct FloorRange {
    start: u32,
    end: u32,
}

#[derive(Debug, Serialize)]
pub struct ResponseGetFloor {
    state: ResponseResult,
    info: Option<GetFloorInfo>,
}

#[derive(Debug, Serialize)]
pub struct GetFloorInfo {
    floors: Vec<Floor>,
}

pub async fn get_floors(
    Path(post_id): Path<i32>,
    Query(floor_range): Query<FloorRange>,
    State(pool): State<ConnectionPool>,
) -> Json<ResponseGetFloor> {
    if floor_range.start > floor_range.end {
        return ResponseGetFloor {
            state: ResponseResult::Err,
            info: None,
        }
        .into();
    }
    let start = floor_range.start;
    let end = if floor_range.end - start < constants::MAX_REQUEST_FLOOR_NUMBER as u32 {
        floor_range.end
    } else {
        start + constants::MAX_REQUEST_FLOOR_NUMBER as u32 - 1
    };
    let floors_vec: Vec<Floor> = sqlx::query_as(
        r#"SELECT * from floors
                    WHERE post_id = ?
                    WHERE floor_number BETWEEN ? AND ?
            "#,
    )
    .bind(post_id)
    .bind(start as i32)
    .bind(end as i32)
    .fetch_all(&pool)
    .await
    .unwrap();

    ResponseGetFloor {
        state: ResponseResult::Ok,
        info: Some(GetFloorInfo { floors: floors_vec }),
    }
    .into()
}
