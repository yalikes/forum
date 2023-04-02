use axum::extract::State;
use axum::{
    extract::Form,
    response::Html,
};

use rand;

use tokio::io::AsyncReadExt;
use diesel::{QueryDsl, RunQueryDsl};

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
    unimplemented!();
    "create post sucess!".to_owned().into()
}
