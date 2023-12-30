use log::debug;
use sea_orm::{entity::ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use sqlx::FromRow;

use crate::helper::ConnectionPool;

pub async fn get_user_name(user_id: i32, pool: &ConnectionPool) -> Option<String> {
    use crate::entity::users;
    use crate::entity::users::Entity as Users;
    Users::find()
        .filter(users::Column::Id.eq(user_id))
        .select_only()
        .column(users::Column::Name)
        .into_tuple()
        .one(pool)
        .await
        .unwrap_or(None)
}

pub async fn get_user_name_with_default(user_id: Option<i32>, pool: &ConnectionPool) -> String {
    match user_id {
        None => "unknow user".to_string(),
        Some(id) => get_user_name(id, &pool.clone())
            .await
            .unwrap_or("unknow user".to_string()),
    }
}
