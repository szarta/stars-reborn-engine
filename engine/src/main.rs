// stars-server — standalone HTTP game engine
//
// Single-player: launched by the thin launcher on localhost; shut down on exit.
// Multiplayer:   run standalone; clients connect to the configured address.
//
// Configuration via environment variables:
//   SERVER_ADDR  — bind address (default: 127.0.0.1:8080)
//   RUST_LOG     — log level (default: info)

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| "127.0.0.1:8080".parse().unwrap());

    log::info!("stars-server v{} listening on {}", stars_engine::VERSION, addr);

    let app = stars_engine::http::router();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
