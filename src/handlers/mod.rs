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

pub async fn create_vote_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVoteDto>,
) -> impl IntoResponse {
    // 1. Persistencia síncrona en la base de datos
    match VoteService::create_vote(&state.db, payload).await {
        Ok(saved_vote) => {
            // 2. 🟢 NUEVO: Publicación asíncrona del evento en RabbitMQ
            // Usamos un clon del canal AMQP (es muy barato de clonar porque es un puntero interno)
            if let Err(err) =
                VoteService::publish_vote_event(&state.amqp_channel, &saved_vote).await
            {
                // Si la mensajería falla, lanzamos un warning en logs, pero NO le rompemos la experiencia
                // al usuario (el voto ya está seguro en MongoDB)
                tracing::error!(
                    "[RABBITMQ-ERROR] No se pudo publicar el evento de votación: {:?}",
                    err
                );
            }

            // 3. Responder de inmediato al cliente (201 Created)
            (StatusCode::CREATED, Json(serde_json::json!(saved_vote)))
        }
        Err(err) => {
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
