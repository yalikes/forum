use axum::extract::State;
use axum::{extract::Form, response::Html};

use rand;

use diesel::{QueryDsl, RunQueryDsl};
use tokio::io::AsyncReadExt;

use crate::helper::*;
use crate::models::{InsertableFloor, InsertablePost};
use crate::schema;

pub async fn newpost(may_user_id: UserIdFromSession) -> Html<String> {
    if let UserIdFromSession::FoundUserId(_) = may_user_id {
        let mut file = match tokio::fs::File::open("dist/newpost.html").await {
            Ok(file) => file,
            Err(_) => return "".to_owned().into(),
        };
        let mut body = String::new();
        file.read_to_string(&mut body).await.unwrap();
        return body.into();
    } else {
        return "please login".to_owned().into();
    }
}

pub async fn newpost_post(
    may_user_id: UserIdFromSession,
    State(pool): State<ConnectionPool>,
    Form(submited_post): Form<NewPost>,
) -> Html<String> {
    use schema::floors::dsl::floors;
    use schema::posts::dsl::{id as dsl_post_id, posts};
    let user_id = match may_user_id {
        UserIdFromSession::FoundUserId(id) => id,
        UserIdFromSession::NotFound => return "please login".to_owned().into(),
    };

    let post_id: i32 = diesel::insert_into(posts)
        .values(InsertablePost {
            author: user_id,
            title: submited_post.title,
        })
        .returning(dsl_post_id)
        .get_result(&mut pool.get().unwrap())
        .unwrap();
    diesel::insert_into(floors)
        .values(InsertableFloor {
            post_id,
            floor_number: 1,
            author: user_id,
            content: submited_post.content,
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();
    "create post sucess!".to_owned().into()
}
