use std::vec;

use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::Json;

use diesel::expression_methods::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

use crate::constants;
use crate::helper;
use crate::helper::*;
use crate::models::Floor;
use crate::models::Post;
use crate::schema;

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
    use schema::floors::dsl::{floors, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};

    let post_inner: Post = match posts
        .find(post_id)
        .select((dsl_id, author_id, title))
        .first::<Post>(&mut pool.get().unwrap())
    {
        Ok(p) => p,
        Err(_) => {
            return ResponseGetPost {
                state: ResponseResult::Err,
                info: None,
            }
            .into();
        }
    };

    let post: PostWithAuthor = match users
        .find(post_inner.author)
        .select(name)
        .first::<String>(&mut pool.get().unwrap())
    {
        Ok(author_name) => PostWithAuthor {
            post: post_inner,
            author_name: author_name,
        },
        Err(_) => PostWithAuthor {
            post: post_inner,
            author_name: "unknown".to_owned(),
        },
    };
    let floor_num = floors
        .filter(post_id_dsl.eq(post.post.id))
        .count()
        .get_result::<i64>(&mut pool.get().unwrap())
        .unwrap_or(0) as u32;

    ResponseGetPost {
        state: ResponseResult::Ok,
        info: Some(PostInfo {
            title: post.post.title,
            author: post.author_name,
            author_id: post.post.author,
            floor_num,
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
    use schema::floors::dsl::{floor_number, floors, post_id};
    let floors_vec: Vec<Floor> = floors
        .filter(post_id.eq(post_id))
        .filter(floor_number.between(start as i32, end as i32))
        .get_results::<Floor>(&mut pool.get().unwrap())
        .unwrap();
    ResponseGetFloor {
        state: ResponseResult::Ok,
        info: Some(GetFloorInfo { floors: floors_vec }),
    }
    .into()
}
