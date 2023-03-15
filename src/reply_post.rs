use axum::{
    extract::Extension,
    extract::Path,
    http::StatusCode,
    response::{Form, Html, Response, IntoResponse, },
};

use diesel::expression_methods::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};
use tera::{Context, Tera};

use crate::helper::*;
use crate::models::{Floor, Post};
use crate::schema;

pub async fn reply_post(
    Path(post_id): Path<i32>,
    Form(reply): Form<ReplyPost>,
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> impl IntoResponse {
    use schema::floor::dsl::{floor, floor_number, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};
    let user_id = match may_user_id{
        UserIdFromSession::NotFound => {
            return Err(StatusCode::UNAUTHORIZED);
        },
        UserIdFromSession::FoundUserId(user_id) => user_id
    };
    println!("{:?}", user_id);
    return Ok(())
}
