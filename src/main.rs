mod config;
mod handlers;
mod routes;
mod models;
mod services;

use config::AppConfig;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port)); // 0.0.0.0 para escuchar dentro de Docker

    let app = routes::create_router();

    tracing::info!("[VOTING-SERVICE] Inicializado en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}