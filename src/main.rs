use axum::{
    routing::{get},
    Router,
};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};

pub mod utils;
pub mod types;

#[tokio::main]
async fn main() {
    // Load environment variables from a .env file
    dotenv().ok();
    // Create a CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any) // Allow requests from any origin
        .allow_methods(Any); // Allow any HTTP method

    // Build the application with two routes and apply the CORS layer
    let app = Router::new()
        .layer(cors) // Add the CORS layer to the app
        .route("/", get(utils::root_handler))
        .route("/randomart", get(utils::randomart_handler))
        .route("/albumofinterest", get(utils::album_of_interest_handler))   
        .route("/songsforalbum", get(utils::songs_for_album_handler))
        .route("/artiststartswith", get(utils::artist_starts_with_handler))
        .route("/albumstartswith", get(utils::album_starts_with_handler))
        .route("/currentPlayingImg", get(utils::current_playing_image_handler))
        .route("/artistforalpha", get(utils::artist_for_alpha_handler))
        .route("/albumforalpha", get(utils::album_for_alpha_handler))
        .route("/albumsforartist", get(utils::albums_for_artist_handler))
        .route("/albumsforartistsongs", get(utils::albums_for_artist_songs_handler))
        .route("/songpages", get(utils::song_pages_handler))
        .route("/songsforpage", get(utils::songs_for_page_handler))
        .route("/playlistcheck", get(utils::playlist_check_handler))
        .route("/createemptyplaylist", get(utils::create_empty_playlist_handler))
        .route("/createrandomplaylist", get(utils::create_random_playlist_handler))
        .route("/allplaylists", get(utils::all_playlists_handler))
        .route("/editplaylistpage", get(utils::edit_playlist_page_handler))
        .route("/addsongtoplaylist", get(utils::add_song_to_playlist_handler))
        .route("/removesongfromplaylist", get(utils::remove_song_from_playlist_handler))
        .route("/deleteplaylist", get(utils::delete_playlist_handler))
        .route("/coverartfromplaypath", get(utils::cover_art_from_play_path_handler))
        .route("/playmusic", get(utils::play_music_handler))
        .route("/playplaylist", get(utils::play_playlist_handler));

    //Define the address to run the server on
    let addr = SocketAddr::from(([10, 0, 4, 76], 3000));
    println!("Listening on {}", addr);

    // let raw_addr = std::env::var("RUSIC_SOCKET_ADDR").unwrap();
    // let raw_port = std::env::var("RUSIC_PORT").unwrap();
    // let addr = SocketAddr::new(raw_addr.parse().unwrap(), raw_port.parse().unwrap());
    println!("Listening on {}", addr);
    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}