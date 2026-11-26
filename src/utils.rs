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

use crate::types::{AlbumCount, AlbumStruct, ArtistCount, ArtistForAlphaStruct, MusicImageInfo, MusicInfo, PlayList};

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
    let random_art = randomart_handler().await;
    random_art
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

    let mut stmt = match conn.prepare("SELECT alpha, count FROM artistcount") {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing artist_starts_with query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok(ArtistCount {
            alpha: row.get(0)?,
            count: row.get(1)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing artist_starts_with query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut results = Vec::new();
    for row in rows {
        match row {
            Ok(item) => results.push(item),
            Err(err) => {
                eprintln!("ArtistStartsWith Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(results).into_response()
}

pub async fn album_starts_with_handler() -> impl IntoResponse {
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

    let mut stmt = match conn.prepare("SELECT alpha, count FROM albumcount") {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing album_starts_with query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([], |row| {
        Ok(AlbumCount {
            alpha: row.get(0)?,
            count: row.get(1)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing album_starts_with query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut results = Vec::new();
    for row in rows {
        match row {
            Ok(item) => results.push(item),
            Err(err) => {
                eprintln!("AlbumStartsWith Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(results).into_response()
}

#[derive(Deserialize)]
pub struct CurrentPlayingImageQuery {
    #[serde(rename = "albumId")]
    pub album_id: String,
}

pub async fn current_playing_image_handler(Query(params): Query<CurrentPlayingImageQuery>) -> impl IntoResponse {
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
        "SELECT id, rusicid, width, height, artist, artistid, album, albumid, filesize, fullpath, thumbpath, idx, page, httpthumbpath FROM music_images WHERE albumid = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing current_playing_image query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.album_id], |row| {
        Ok(MusicImageInfo {
            id: row.get(0)?,
            rusicid: row.get(1)?,
            width: row.get(2)?,
            height: row.get(3)?,
            artist: row.get(4)?,
            artistid: row.get(5)?,
            album: row.get(6)?,
            albumid: row.get(7)?,
            filesize: row.get(8)?,
            fullpath: row.get(9)?,
            thumbpath: row.get(10)?,
            idx: row.get(11)?,
            page: row.get(12)?,
            httpthumbpath: row.get(13)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing current_playing_image query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut image = MusicImageInfo::default();
    for row in rows {
        match row {
            Ok(item) => image = item,
            Err(err) => {
                eprintln!("CurrentPlayingImg Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(image).into_response()
}

#[derive(Deserialize)]
pub struct ArtistForAlphaQuery {
    pub alpha: String,
}

pub async fn artist_for_alpha_handler(Query(params): Query<ArtistForAlphaQuery>) -> impl IntoResponse {
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
        "SELECT DISTINCT artist, artistid FROM startswith WHERE artist_first_letter = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing artist_for_alpha query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.alpha], |row| {
        Ok(ArtistForAlphaStruct {
            artist: row.get(0)?,
            artistid: row.get(1)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing artist_for_alpha query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut artists = Vec::new();
    for row in rows {
        match row {
            Ok(item) => artists.push(item),
            Err(err) => {
                eprintln!("ArtistForAlpha Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(artists).into_response()
}

#[derive(Deserialize)]
pub struct AlbumForAlphaQuery {
    pub alpha: String,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone)]
struct AlbumForAlphaStruct {
    albumid: String,
}

pub async fn album_for_alpha_handler(Query(params): Query<AlbumForAlphaQuery>) -> impl IntoResponse {
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
        "SELECT DISTINCT album, albumid FROM startswith WHERE album_first_letter = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing album_for_alpha query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.alpha], |row| {
        Ok(AlbumForAlphaStruct {
            albumid: row.get(1)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing album_for_alpha query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut albums = Vec::new();
    for row in rows {
        match row {
            Ok(item) => albums.push(item),
            Err(err) => {
                eprintln!("AlbumForAlpha Error scanning row: {}", err);
                continue;
            }
        }
    }

    let mut album_list = Vec::new();
    for alb in albums {
        let mut stmt = match conn.prepare(
            "SELECT DISTINCT album, albumid, httpthumbpath FROM music_images WHERE albumid = ?1",
        ) {
            Ok(stmt) => stmt,
            Err(err) => {
                eprintln!("Error preparing album image lookup: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to prepare query"})),
                )
                    .into_response();
            }
        };

        let image_rows = match stmt.query_map([alb.albumid], |row| {
            Ok(AlbumStruct {
                album: row.get(0)?,
                albumid: row.get(1)?,
                httpthumbpath: row.get(2)?,
            })
        }) {
            Ok(rows) => rows,
            Err(err) => {
                eprintln!("Error executing album image lookup: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": "Failed to execute query"})),
                )
                    .into_response();
            }
        };

        for row in image_rows {
            match row {
                Ok(item) => album_list.push(item),
                Err(err) => {
                    eprintln!("AlbumForAlpha image scan error: {}", err);
                    continue;
                }
            }
        }
    }

    Json(album_list).into_response()
}

#[derive(Deserialize)]
pub struct AlbumsForArtistQuery {
    #[serde(rename = "artistid", alias = "artid", alias = "artistId")]
    pub artist_id: String,
}

pub async fn albums_for_artist_handler(Query(params): Query<AlbumsForArtistQuery>) -> impl IntoResponse {
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
        "SELECT DISTINCT albumid, album, httpthumbpath FROM music_images WHERE artistid = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing albums_for_artist query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.artist_id], |row| {
        Ok(AlbumStruct {
            albumid: row.get(0)?,
            album: row.get(1)?,
            httpthumbpath: row.get(2)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing albums_for_artist query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut albums = Vec::new();
    for row in rows {
        match row {
            Ok(item) => albums.push(item),
            Err(err) => {
                eprintln!("AlbumsForArtist Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(albums).into_response()
}

#[derive(Deserialize)]
pub struct AlbumsForArtistSongsQuery {
    #[serde(rename = "albumId", alias = "albumid")]
    pub album_id: String,
}

pub async fn albums_for_artist_songs_handler(Query(params): Query<AlbumsForArtistSongsQuery>) -> impl IntoResponse {
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
            eprintln!("Error preparing albums_for_artist_songs query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.album_id], |row| {
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
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing albums_for_artist_songs query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut songs = Vec::new();
    for row in rows {
        match row {
            Ok(item) => songs.push(item),
            Err(err) => {
                eprintln!("AlbumsForArtistSongs Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(songs).into_response()
}

pub async fn song_pages_handler() -> impl IntoResponse {
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

    let mut stmt = match conn.prepare("SELECT DISTINCT page FROM music") {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing song_pages query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([], |row| row.get::<_, String>(0)) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing song_pages query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut pages = Vec::new();
    for row in rows {
        match row {
            Ok(page) => pages.push(page),
            Err(err) => {
                eprintln!("SongPages Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(pages).into_response()
}

#[derive(Deserialize)]
pub struct SongsForPageQuery {
    pub page: String,
}

pub async fn songs_for_page_handler(Query(params): Query<SongsForPageQuery>) -> impl IntoResponse {
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
        "SELECT id, rusicid, imgurl, playpath, artist, artistid, album, albumid, song, fullpath, extension, idx, page, fsizeresults FROM music WHERE page = ?1",
    ) {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing songs_for_page query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to prepare query"})),
            )
                .into_response();
        }
    };

    let rows = match stmt.query_map([params.page], |row| {
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
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing songs_for_page query: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to execute query"})),
            )
                .into_response();
        }
    };

    let mut songs = Vec::new();
    for row in rows {
        match row {
            Ok(item) => songs.push(item),
            Err(err) => {
                eprintln!("SongsForPage Error scanning row: {}", err);
                continue;
            }
        }
    }

    Json(songs).into_response()
}

pub async fn playlist_check_handler() -> impl IntoResponse {
    Json(json!({"exists": playlist_check()})).into_response()
}

pub fn playlist_check() -> bool {
    let db_path = match std::env::var("RUSIC_DB_PATH") {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Missing RUSIC_DB_PATH: {}", err);
            return false;
        }
    };

    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error opening database at {}: {}", db_path, err);
            return false;
        }
    };

    let mut stmt = match conn.prepare("SELECT 1 FROM playlists LIMIT 1") {
        Ok(stmt) => stmt,
        Err(err) => {
            eprintln!("Error preparing playlist check query: {}", err);
            return false;
        }
    };

    let mut rows = match stmt.query([]) {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing playlist check query: {}", err);
            return false;
        }
    };

    match rows.next() {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(err) => {
            eprintln!("Error reading playlist check result: {}", err);
            false
        }
    }
}

#[derive(Deserialize)]
pub struct CreateEmptyPlaylistQuery {
    #[serde(rename = "plname", alias = "name", alias = "playlistName")]
    pub plname: String,
}

pub async fn create_empty_playlist_handler(
    Query(params): Query<CreateEmptyPlaylistQuery>,
) -> impl IntoResponse {
    let trimmed_name = params.plname.trim();
    if trimmed_name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "plname is required"})),
        )
            .into_response();
    }

    let playlist = create_empty_playlist(trimmed_name);
    Json(playlist).into_response()
}

fn create_md5_hash(aname: &str) -> String {
    format!("{:x}", md5::compute(aname.as_bytes()))
}

pub fn create_empty_playlist(plname: &str) -> PlayList {
    let pllist = PlayList {
        rusicid: create_md5_hash(plname),
        name: plname.to_string(),
        songs: "None".to_string(),
        numsongs: "0".to_string(),
    };

    let db_path = match std::env::var("RUSIC_DB_PATH") {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Missing RUSIC_DB_PATH: {}", err);
            return pllist;
        }
    };

    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error opening database at {}: {}", db_path, err);
            return pllist;
        }
    };

    if let Err(err) = conn.execute(
        "INSERT INTO playlists (rusicid, name, songs, numsongs) VALUES (?1, ?2, ?3, ?4)",
        (&pllist.rusicid, &pllist.name, &pllist.songs, &pllist.numsongs),
    ) {
        eprintln!("Error inserting playlist: {}", err);
    }

    pllist
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

pub async fn play_playlist_handler() -> impl IntoResponse {
    // Placeholder implementation
    json!({"message": "Play playlist endpoint"}).to_string()
}