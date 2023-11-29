use axum::extract::State;
use axum::response::Json;
use serde::Serialize;

use diesel::{QueryDsl, RunQueryDsl};
use crate::helper::*;
use crate::schema;
use crate::models::Post;
use crate::helper::ResponseResult;

#[derive(Debug, Serialize)]
pub struct ResponseGetRecentPost{
    state: ResponseResult,
    info: Option<ResponseGetRecentPostInfo>
}

#[derive(Debug, Serialize)]
pub struct ResponseGetRecentPostInfo{
    posts: Vec<PostWithAuthor>
}

pub async fn get_recent_post(
    State(pool): State<ConnectionPool>,
) -> Json<ResponseGetRecentPost> {
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};
    let mut posts_with_author_name: Vec<PostWithAuthor> = Vec::new();
    let conn = &mut pool.get().unwrap();
    if let Ok(some_post) = posts
        .select((dsl_id, author_id, title))
        .limit(10)
        .get_results::<Post>(conn)
    {
        tracing::debug!("{:?}", &some_post);
        for post in some_post {
            let author_name = users
                .select(name)
                .find(post.author.unwrap())
                .get_result::<String>(conn)
                .unwrap_or_else(|_| "unknown user".to_owned());
            posts_with_author_name.push(PostWithAuthor { post, author_name });
        }

    }
    ResponseGetRecentPost{
        state: ResponseResult::Ok,
        info: Some(ResponseGetRecentPostInfo {
            posts: posts_with_author_name
        })
    }.into()
}
