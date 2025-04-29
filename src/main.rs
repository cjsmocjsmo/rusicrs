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
        .layer(cors); // Add the CORS layer to the app
        .route("/", get(utils::root_handler))
        .route("/randomart", get(utils::randomart_handler))
        

    Define the address to run the server on
    let addr = SocketAddr::from(([10, 0, 4, 1], 3000));
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