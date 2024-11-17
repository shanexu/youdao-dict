use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use crate::{models::History, schema::history};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn list_history(conn: &mut SqliteConnection) -> Vec<History> {
    use super::schema::history::dsl::*;

    let results = history
        .select(History::as_select())
        .load(conn)
        .expect("Error loading posts");
    results
}

#[derive(Insertable)]
#[diesel(table_name = history)]
pub struct NewHistory<'a> {
    pub word: &'a str,
    pub created_at: &'a chrono::NaiveDateTime,
}

pub fn create_history(conn: &mut SqliteConnection, word: &str) -> History {
    use super::schema::history;
    use chrono::Local;

    let new_history = NewHistory {
        word,
        created_at: &Local::now().naive_local(),
    };

    diesel::insert_into(history::table)
        .values(&new_history)
        .returning(History::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}
