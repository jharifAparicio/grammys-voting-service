mod config;
mod handlers;
mod models;
mod routes;
mod services;

use config::AppConfig;
use mongodb::{Client, Database};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// 1. Estructura que almacena el pool de conexiones de MongoDB
pub struct AppState {
    pub db: Database,
}

#[tokio::main]
async fn main() {
    // Inicializar el sistema de logs para ver las trazas en la terminal de Docker
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Cargar la configuración (incluyendo la MONGO_URI)
    let config = AppConfig::from_env();

    // 2. Intentar la conexión asíncrona con MongoDB
    tracing::info!(
        "[VOTING-SERVICE] Conectando a MongoDB en: {}",
        config.mongo_uri
    );
    let client = Client::with_uri_str(&config.mongo_uri)
        .await
        .expect("Error crítico: No se pudo inicializar el cliente de MongoDB");

    // Seleccionar la base de datos (si no existe, Mongo la crea al insertar el primer documento)
    let db = client.database("votes_db");
    tracing::info!("[VOTING-SERVICE] Conexión a MongoDB establecida con éxito.");

    // 3. Envolver el estado en un Arc (Atomic Reference Counted) para compartirlo de forma segura entre hilos
    let shared_state = Arc::new(AppState { db });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));

    // Pasamos el estado inyectado al constructor del enrutador
    let app = routes::create_router(shared_state);

    tracing::info!(
        "[VOTING-SERVICE] Servidor escuchando exitosamente en http://{}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
