use axum::{Router, routing::get};
use crate::handlers::health_check;

pub fn create_router() -> Router {
    Router::new()
        .nest("/api", Router::new()
            .route("/votes/health", get(health_check))
        )
}