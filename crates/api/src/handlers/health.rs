use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;
use shared::error::AppResult;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

async fn health_check(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    sqlx::query!("SELECT 1 as result")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    }))
}
