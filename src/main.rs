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
        .route("/", get(utils::root_handler))
        .route("/randomarttest", get(utils::randomart_handler))
        .layer(cors); // Add the CORS layer to the app

    // Define the address to run the server on
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    // Run the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}