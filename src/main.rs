#[macro_use]
extern crate diesel;

use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use dotenv::dotenv;

use openssl;
use rand;

use std::env;

pub mod models;
pub mod schema;

use self::models::User;
fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn check(password: String, salt: String, password_hash: Vec<u8>) -> bool {
    // TODO: optimize here
    openssl::sha::sha256(&(password + &salt).into_bytes()).to_vec() == password_hash
}

fn main() {
    use schema::users::dsl::*;
    let connection = establish_connection();
    let results = users
        .load::<User>(&connection)
        .expect("Error loading users");
    for r in results {
        println!("{:?}", r);
        println!("check: {}", check("python".to_owned(), r.salt, r.passwd));
    }
}
