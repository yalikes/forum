use axum::{
    extract::Extension,
    extract::Path,
    response::Html,
};


use diesel::{QueryDsl, RunQueryDsl};
use diesel::expression_methods::ExpressionMethods;
use tera::{Tera, Context};

use crate::helper::*;
use crate::models::{Post, Floor};
use crate::schema;

pub async fn get_post_with_page(
    Path((post_id, page)): Path<(i32, i32)>,
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> Html<String> {
    use schema::floor::dsl::{floor, floor_number, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};

    let post_inner: Post = match posts
        .find(post_id)
        .select((dsl_id, author_id, title))
        .first::<Post>(&mut pool.get().unwrap())
    {
        Ok(p) => p,
        Err(_) => {
            return "post not found".to_owned().into();
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

    let floors = match floor
        .filter(post_id_dsl.eq(post.post.id))
        .order_by(floor_number)
        .limit(10)
        .offset(((if page < 0 { 0 } else { page - 1 }) * 10).into())
        .get_results::<Floor>(&mut pool.get().unwrap())
    {
        Ok(floors) => floors,
        Err(_) => {
            return "can't find floors".to_owned().into();
        }
    };

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("floors", &floors);
    context.insert(
        "logined",
        &(if let UserIdFromSession::FoundUserId(_) = may_user_id {
            true
        } else {
            false
        }),
    );
    tera.render("post.html", &context).unwrap().into()
}

pub async fn get_post(
    Path(post_id): Path<i32>,
    may_user_id: UserIdFromSession,
    Extension((tera, pool)): Extension<(Tera, SqliteConnectionPool)>,
) -> Html<String> {
    use schema::floor::dsl::{floor, post_id as post_id_dsl};
    use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
    use schema::users::dsl::{name, users};

    let post_inner: Post = match posts
        .find(post_id)
        .select((dsl_id, author_id, title))
        .first::<Post>(&mut pool.get().unwrap())
    {
        Ok(p) => p,
        Err(_) => {
            return "post not found".to_owned().into();
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

    let floors = match floor
        .filter(post_id_dsl.eq(post.post.id))
        .limit(10)
        .get_results::<Floor>(&mut pool.get().unwrap())
    {
        Ok(floors) => floors,
        Err(_) => {
            return "can't find floors".to_owned().into();
        }
    };

    let mut context = Context::new();
    context.insert("post", &post);
    context.insert("floors", &floors);
    context.insert(
        "logined",
        &(if let UserIdFromSession::FoundUserId(_) = may_user_id {
            true
        } else {
            false
        }),
    );
    tera.render("post.html", &context).unwrap().into()
}
