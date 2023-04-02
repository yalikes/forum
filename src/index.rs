use axum::response::IntoResponse;
use axum::{
    extract::State,
    response::Html
};

use diesel::r2d2::R2D2Connection;
use diesel::{QueryDsl, RunQueryDsl};
use crate::helper::*;
use crate::schema;
use crate::models::Post;
pub async fn index(
    State(pool): State<ConnectionPool>,
) -> impl IntoResponse {
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
                .find(post.author)
                .get_result::<String>(conn)
                .unwrap_or_else(|_| "unknown user".to_owned());
            posts_with_author_name.push(PostWithAuthor { post, author_name });
        }

    }
    serde_json::to_string(&posts_with_author_name).unwrap()
}