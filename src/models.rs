#[derive(Queryable, Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub passwd: Vec<u8>,
    pub salt: String,
}
