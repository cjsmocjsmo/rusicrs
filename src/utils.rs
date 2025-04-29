use axum::response::IntoResponse;
use rusqlite::Connection;
use rand::seq::SliceRandom;
use serde_json::json;

// Handler for the "/" route
pub async fn root_handler() -> &'static str {
    "Hello From Rusic"
}

// Handler for the "/test" route
pub async fn randomart_handler() -> impl IntoResponse {
    let db_path = std::env::var("RUSIC_DB_PATH").expect("RUSIC_DB_PATH must be set");
    println!("DB Path: {}", db_path);
    let conn = Connection::open(db_path).expect("Failed to connect to the database");

    let mut stmt = conn.prepare("SELECT idx FROM music_images;").expect("Failed to prepare query");
    let rows = stmt
        .query_map([], |row| row.get(0))
        .expect("Failed to fetch data");

    let index_list: Vec<i32> = rows
        .filter_map(Result::ok)
        .collect();

    println!("Index List: {:?}", index_list);

    let mut rng = rand::thread_rng();
    let random_indices: Vec<i32> = index_list
        .choose_multiple(&mut rng, 5)
        .cloned()
        .collect();

    json!(random_indices).to_string()
}
