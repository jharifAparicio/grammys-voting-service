use crate::AppState;
use crate::handlers::{
    create_vote_handler, get_all_votes_handler, get_votes_by_user_handler, health_check,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/votes/health", get(health_check))
                .route("/votes", post(create_vote_handler))
                .route("/votes", get(get_all_votes_handler))
                .route("/votes/user/{user_id}", get(get_votes_by_user_handler)),
        )
        .with_state(state)
}
