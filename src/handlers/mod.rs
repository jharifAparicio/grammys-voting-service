use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "grammys-voting-service",
        "version": "1.0.0",
        "runtime": "Rust + Axum"
    }))
}