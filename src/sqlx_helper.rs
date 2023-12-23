use sqlx::FromRow;

#[derive(FromRow)]
pub struct SqlxI32{
    pub val: i32
}

#[derive(FromRow)]
pub struct SqlxString{
    pub val: String
}