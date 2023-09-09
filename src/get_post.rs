use axum::extract::State;
use axum::{extract::Path, response::Html};

use diesel::expression_methods::ExpressionMethods;
use diesel::{QueryDsl, RunQueryDsl};

use crate::helper::*;
use crate::models::{Floor, Post};
use crate::schema;

// pub async fn get_post_with_page(
//     Path((post_id, page)): Path<(i32, i32)>,
//     may_user_id: UserIdFromSession,
//     State(pool): State<ConnectionPool>,
// ) -> Html<String> {
//     use schema::floors::dsl::{floor_number, floors, post_id as post_id_dsl};
//     use schema::posts::dsl::{author as author_id, id as dsl_id, posts, title};
//     use schema::users::dsl::{name, users};

//     let post_inner: Post = match posts
//         .find(post_id)
//         .select((dsl_id, author_id, title))
//         .first::<Post>(&mut pool.get().unwrap())
//     {
//         Ok(p) => p,
//         Err(_) => {
//             return "post not found".to_owned().into();
//         }
//     };

//     let post: PostWithAuthor = match users
//         .find(post_inner.author)
//         .select(name)
//         .first::<String>(&mut pool.get().unwrap())
//     {
//         Ok(author_name) => PostWithAuthor {
//             post: post_inner,
//             author_name: author_name,
//         },
//         Err(_) => PostWithAuthor {
//             post: post_inner,
//             author_name: "unknown".to_owned(),
//         },
//     };

//     let the_floors = match floors
//         .filter(post_id_dsl.eq(post.post.id))
//         .order_by(floor_number)
//         .limit(10)
//         .offset(((if page < 0 { 0 } else { page - 1 }) * 10).into())
//         .get_results::<Floor>(&mut pool.get().unwrap())
//     {
//         Ok(the_floors) => the_floors,
//         Err(_) => {
//             return "can't find floors".to_owned().into();
//         }
//     };

//     "not implement".to_owned().into()
// }

pub async fn get_post(
    Path(post_id): Path<i32>,
    may_user_id: UserIdFromSession,
    State(pool): State<ConnectionPool>,
) -> Html<String> {
    use schema::floors::dsl::{floors, post_id as post_id_dsl, id as floor_id_dsl, floor_number, author, content};
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

    let the_floors = match floors
        .filter(post_id_dsl.eq(post.post.id))
        .limit(10)
        .select((floor_id_dsl, post_id_dsl, floor_number, author, content))
        .get_results::<Floor>(&mut pool.get().unwrap())
    {
        Ok(the_floors) => the_floors,
        Err(_) => {
            return "can't find floors".to_owned().into();
        }
    };

    "not implement".to_owned().into()
}

pub async fn get_floors(Path(post_id): Path<i32>,) {}
