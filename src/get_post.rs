use std::vec;

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;

use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
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
    use crate::entity::floors;
    use crate::entity::posts;
    let post_inner: Post = match posts::Entity::find_by_id(post_id).one(&pool).await {
        Ok(p) => match p {
            Some(post) => Post {
                id: post.id,
                author: post.author,
                title: post.title,
                post_create_time: post.post_create_time,
            },
            None => {
                return ResponseGetPost {
                    state: ResponseResult::Err,
                    info: None,
                }
                .into();
            }
        },
        Err(e) => {
            return ResponseGetPost {
                state: ResponseResult::Err,
                info: None,
            }
            .into();
        }
    };
    let author_name = get_user_name_with_default(post_inner.author, &pool.clone()).await;
    let post: PostWithAuthor = PostWithAuthor {
        post: post_inner,
        author_name,
    };
    let floor_num = floors::Entity::find()
        .filter(floors::Column::PostId.eq(post_id))
        .count(&pool)
        .await
        .unwrap_or(0);
    ResponseGetPost {
        state: ResponseResult::Ok,
        info: Some(PostInfo {
            title: post.post.title,
            author: post.author_name,
            author_id: post.post.author.unwrap(),
            floor_num: floor_num as u32,
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

    use crate::entity::floors;

    let floors_vec: Vec<Floor> = floors::Entity::find()
        .filter(floors::Column::PostId.eq(post_id))
        .filter(floors::Column::FloorNumber.between(start, end))
        .all(&pool)
        .await
        .unwrap_or(vec![])
        .iter()
        .map(|m| Floor {
            id: m.id,
            post_id: m.post_id,
            floor_number: m.floor_number,
            author: m.author,
            content: m.content.clone(),
            floor_create_time: m.floor_create_time,
        })
        .collect();
    ResponseGetFloor {
        state: ResponseResult::Ok,
        info: Some(GetFloorInfo { floors: floors_vec }),
    }
    .into()
}
