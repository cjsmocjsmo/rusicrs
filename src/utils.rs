use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use rusqlite::Connection;
use serde::Deserialize;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::types::MusicInfo;

pub async fn root_handler() -> impl IntoResponse {
    let response = json!({
        "message": "Welcome to the Rusic API",
        "version": "1.0.0",
        "endpoints": [
            {
                "path": "/randomart",
                "description": "Get random art IDs"
            }
        ]
    });
    response.to_string()
}

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







pub async fn album_of_interest_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Album of interest endpoint"}).to_string()
}








#[derive(Deserialize)]
pub struct SongsForAlbumQuery {
    #[serde(rename = "albumId")]
    pub album_id: String,
}

pub async fn songs_for_album_handler(Query(params): Query<SongsForAlbumQuery>) -> impl IntoResponse {
    let db_path = match std::env::var("RUSIC_DB_PATH") {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Missing RUSIC_DB_PATH: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "RUSIC_DB_PATH is not set"})),
            )
                .into_response();
        }
    };

    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error opening database at {}: {}", db_path, err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to open database"})),
            )
                .into_response();
        }
    };

    let mut stmt = match conn.prepare(
        "SELECT id, rusicid, imgurl, playpath, artist, artistid, album, albumid, song, fullpath, extension, idx, page, fsizeresults FROM music WHERE albumid = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing songs_for_album query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let songs_iter = match stmt.query_map([params.album_id], |row| {
        Ok(MusicInfo {
            id: row.get(0)?,
            rusicid: row.get(1)?,
            imgurl: row.get(2)?,
            playpath: row.get(3)?,
            artist: row.get(4)?,
            artistid: row.get(5)?,
            album: row.get(6)?,
            albumid: row.get(7)?,
            song: row.get(8)?,
            fullpath: row.get(9)?,
            extension: row.get(10)?,
            idx: row.get(11)?,
            page: row.get(12)?,
            fsizeresults: row.get(13)?,
        })
    }) {
        Ok(iter) => iter,
        Err(err) => {
            eprintln!("Error executing songs_for_album query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut songs = Vec::new();
    for song in songs_iter {
        match song {
            Ok(song) => songs.push(song),
            Err(err) => {
                eprintln!("SongsForAlbum Error scanning row: {}", err);
            }
        }
    }

    Json(songs).into_response()
}







pub async fn artist_starts_with_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Artist starts with endpoint"}).to_string()
}

pub async fn album_starts_with_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Album starts with endpoint"}).to_string()
}

pub async fn current_playing_image_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Current playing image endpoint"}).to_string()
}

pub async fn artist_for_alpha_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Artist for alpha endpoint"}).to_string()
}

pub async fn album_for_alpha_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Album for alpha endpoint"}).to_string()
}

pub async fn albums_for_artist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Albums for artist endpoint"}).to_string()
}

pub async fn albums_for_artist_songs_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Albums for artist songs endpoint"}).to_string()
}

pub async fn song_pages_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Song pages endpoint"}).to_string()
}

pub async fn songs_for_page_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Songs for page endpoint"}).to_string()
}

pub async fn playlist_check_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Playlist check endpoint"}).to_string()
}

pub async fn create_empty_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Create empty playlist endpoint"}).to_string()
}

pub async fn create_random_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Create random playlist endpoint"}).to_string()
}

pub async fn all_playlists_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "All playlists endpoint"}).to_string()
}

pub async fn edit_playlist_page_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Edit playlist page endpoint"}).to_string()
}

pub async fn add_song_to_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Add song to playlist endpoint"}).to_string()
}

pub async fn remove_song_from_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Remove song from playlist endpoint"}).to_string()
}

pub async fn delete_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Delete playlist endpoint"}).to_string()
}

pub async fn cover_art_from_play_path_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Cover art from play path endpoint"}).to_string()
}

pub async fn play_music_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Play music endpoint"}).to_string()
}

pub async fn play_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Play playlist endpoint"}).to_string()
}