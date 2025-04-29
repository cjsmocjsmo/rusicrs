use axum::response::IntoResponse;
use rusqlite::Connection;
use rand::seq::SliceRandom;
use serde_json::json;

pub async fn randomart_handler() -> impl IntoResponse {
    // let db_path = std::env::var("RUSIC_DB_PATH").unwrap_or_else(|_| "/usr/share/rusicrs/rusic.db".to_string());
    let db_path = "/usr/share/rusicrs/rusic.db".to_string();
    println!("DB Path: {}", db_path);

    let conn = Connection::open(&db_path).expect("Failed to connect to the database");

    // Debugging: Check if the table exists
    let table_check = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='music_images';")
        .expect("Failed to prepare table check query")
        .exists([])
        .expect("Failed to execute table check query");

    if !table_check {
        println!("Table 'music_images' does not exist in the database.");
        return json!({"error": "Table 'music_images' does not exist"}).to_string();
    }

    let mut stmt = conn.prepare("SELECT idx FROM music_images;").expect("Failed to prepare query");
    let rows = stmt
        .query_map([], |row| row.get(0))
        .expect("Failed to fetch data");

    let index_list: Vec<i32> = rows.filter_map(Result::ok).collect();

    // Debugging: Check if the index list is empty
    if index_list.is_empty() {
        println!("No data found in the 'music_images' table.");
        return json!({"error": "No data found"}).to_string();
    }

    println!("Index List: {:?}", index_list);

    let mut rng = rand::thread_rng();
    let random_indices: Vec<i32> = index_list
        .choose_multiple(&mut rng, 5)
        .cloned()
        .collect();

    json!(random_indices).to_string()
}