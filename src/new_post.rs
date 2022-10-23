use axum::{
    extract::Extension,
    extract::Form,
    response::Html,
};

use rand;

use tokio::io::AsyncReadExt;
use diesel::{QueryDsl, RunQueryDsl};
use diesel::expression_methods::ExpressionMethods;
use tera::Tera;

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
    Form(submited_post): Form<NewPost>,
    Extension((_, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> Html<String> {
    use schema::floor::dsl::floor;
    use schema::posts::dsl::{extra, id as dsl_post_id, posts};
    let user_id = match may_user_id {
        UserIdFromSession::FoundUserId(id) => id,
        UserIdFromSession::NotFound => return "please login".to_owned().into(),
    };

    //diesel currently not support returning for sqlite, i have to use a wired rourting to keep thing work
    // 1. insert a row with unique extra property
    // 2. find row with this property
    // 3. get id and remove this property
    let extra_id: i32 = rand::random();
    diesel::insert_into(posts)
        .values(InsertablePost {
            author: user_id,
            title: submited_post.title,
            extra: extra_id,
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();
    let post_id = posts
        .filter(extra.eq(extra_id))
        .select(dsl_post_id)
        .get_result::<i32>(&mut pool.get().unwrap()).unwrap();
    diesel::update(posts.find(post_id))
        .set(extra.eq(0)).execute(&mut pool.get().unwrap()).unwrap();
    diesel::insert_into(floor)
        .values(InsertableFloor{
            post_id,
            floor_number:1,
            author:user_id,
            content: submited_post.content
        })
        .execute(&mut pool.get().unwrap())
        .unwrap();
    "create post sucess!".to_owned().into()
}
