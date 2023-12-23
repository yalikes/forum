use sqlx::FromRow;

use crate::helper::ConnectionPool;

pub async fn get_user_name(user_id: i32, pool: ConnectionPool) -> Option<String> {
    #[derive(FromRow)]
    struct AuthorName {
        author_name: String,
    }
    sqlx::query_as::<_, AuthorName>("SELECT name from users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .ok()
        .map(|author_name| author_name.author_name)
}

pub async fn get_user_name_with_default(user_id: Option<i32>, pool: ConnectionPool) -> String {
    match user_id {
        None => "unknow user".to_string(),
        Some(id) => get_user_name(id, pool.clone())
            .await
            .unwrap_or("unknow user".to_string()),
    }
}
