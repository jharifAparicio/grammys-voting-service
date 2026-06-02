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

// 🟢 NUEVO: Importamos las estructuras de conexión de RabbitMQ
use lapin::{Connection, ConnectionProperties};

// 1. Expandimos el estado global para inyectar el Canal de RabbitMQ
pub struct AppState {
    pub db: Database,
    pub amqp_channel: lapin::Channel, // 🟢 Canal multiplexado inyectado
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();

    // === CONFIGURACIÓN DE MONGODB ===
    tracing::info!(
        "[VOTING-SERVICE] Conectando a MongoDB en: {}",
        config.mongo_uri
    );
    let client = Client::with_uri_str(&config.mongo_uri)
        .await
        .expect("Error crítico al inicializar el cliente de MongoDB");
    let db = client.database("votes_db");
    tracing::info!("[VOTING-SERVICE] Conexión a MongoDB establecida con éxito.");

    // === CONFIGURACIÓN DE RABBITMQ ===
    tracing::info!(
        "[VOTING-SERVICE] Conectando a RabbitMQ en: {}",
        config.amqp_uri
    );

    // Inicializar la conexión TCP asíncrona usando el runtime de Tokio
    let amqp_conn = Connection::connect(&config.amqp_uri, ConnectionProperties::default())
        .await
        .expect("Error crítico: No se pudo conectar con el servidor de RabbitMQ");

    // Crear el canal de transmisión sobre la conexión persistente
    let amqp_channel = amqp_conn
        .create_channel()
        .await
        .expect("Error crítico: No se pudo crear el canal de comunicación AMQP");

    tracing::info!("[VOTING-SERVICE] Conexión y canal de RabbitMQ inicializados con éxito.");

    let amqp_channel = amqp_conn
        .create_channel()
        .await
        .expect("Error crítico: No se pudo crear el canal de comunicación AMQP");

    // 🟢 NUEVO: Declarar el Exchange de forma segura al arrancar
    tracing::info!("[VOTING-SERVICE] Asegurando la infraestructura en RabbitMQ...");
    amqp_channel
        .exchange_declare(
            "votes.exchange".into(),     // Nombre del Exchange
            lapin::ExchangeKind::Direct, // Tipo de enrutamiento
            lapin::options::ExchangeDeclareOptions {
                durable: true, // Se mantiene vivo aunque RabbitMQ se reinicie
                ..lapin::options::ExchangeDeclareOptions::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
        .expect("Error crítico: No se pudo declarar el Exchange 'votes.exchange'");

    tracing::info!(
        "[VOTING-SERVICE] Conexión, canal y Exchange de RabbitMQ inicializados con éxito."
    );

    // 2. Empaquetamos ambos pools de conexiones en el Arc global
    let shared_state = Arc::new(AppState { db, amqp_channel });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let app = routes::create_router(shared_state);

    tracing::info!(
        "[VOTING-SERVICE] Servidor escuchando exitosamente en http://{}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
