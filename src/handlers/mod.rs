use crate::AppState;
use crate::models::CreateVoteDto;
use crate::services::VoteService;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::sync::Arc;

// Handler existente de tu Health Check
pub async fn health_check() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "status": "ok",
        "service": "grammys-voting-service",
        "version": "1.0.0",
        "runtime": "Rust + Axum"
    });
    (StatusCode::OK, Json(json_response))
}

// 🟢 NUEVO HANDLER: Registrar un voto por POST
pub async fn create_vote_handler(
    State(state): State<Arc<AppState>>, // Extrae de forma asíncrona nuestro pool de Mongo
    Json(payload): Json<CreateVoteDto>, // Parsea automáticamente el cuerpo JSON entrante
) -> impl IntoResponse {
    // Invocar al servicio para persistir el voto en Mongo
    match VoteService::create_vote(&state.db, payload).await {
        Ok(saved_vote) => {
            // Si tiene éxito, responde con 201 Created y el objeto persistido con su ID
            (StatusCode::CREATED, Json(serde_json::json!(saved_vote)))
        }
        Err(err) => {
            // Si algo falla en la base de datos, responde un 500
            tracing::error!("[MONGODB-ERROR] Falló la inserción del voto: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "No se pudo registrar el voto en el servidor" })),
            )
        }
    }
}

pub async fn get_all_votes_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match VoteService::find_all(&state.db).await {
        Ok(votes) => (StatusCode::OK, Json(serde_json::json!(votes))),
        Err(err) => {
            tracing::error!("[MONGODB-ERROR] Falló la lectura global: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Error interno al recuperar los votos" })),
            )
        }
    }
}

pub async fn get_votes_by_user_handler(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match VoteService::find_by_user(&state.db, &user_id).await {
        Ok(votes) => (StatusCode::OK, Json(serde_json::json!(votes))),
        Err(err) => {
            tracing::error!("[MONGODB-ERROR] Falló la lectura por usuario: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(
                    serde_json::json!({ "error": "Error interno al recuperar los votos del usuario" }),
                ),
            )
        }
    }
}
