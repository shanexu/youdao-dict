use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

use crate::{models::History, schema::history};

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(v) => v,
        _ => home::home_dir()
            .map(|h| {
                format!(
                    "{}/.config/yd/database.db",
                    h.into_os_string().into_string().unwrap()
                )
            })
            .unwrap(),
    };

    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn list_history(conn: &mut SqliteConnection) -> Vec<History> {
    use super::schema::history::dsl::*;

    let results = history
        .order_by(id.desc())
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
