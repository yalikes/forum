use axum::extract::Path;
use axum::extract::State;
use axum::Json;

use diesel::expression_methods::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use serde::Serialize;

use crate::helper::*;
use crate::models::Post;
use crate::schema;

#[derive(Debug, Serialize)]
pub struct ResponseGetPost {
    state: Result<(), ()>,
    info: Option<ResponseGetPostInfo>,
}

#[derive(Debug, Serialize)]
pub struct ResponseGetPostInfo {
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
                state: Err(()),
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
        state: Ok(()),
        info: Some(ResponseGetPostInfo {
            title: post.post.title,
            author: post.author_name,
            author_id: post.post.author,
            floor_num,
        }),
    }
    .into()
}

#[allow(unused)]
pub async fn get_floors(Path(post_id): Path<i32>) {}
