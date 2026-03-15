use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use db::queries::find_active_games;
use shared::error::AppResult;

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list_games))
}

/// GET /api/v1/games
pub async fn list_games(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let games = find_active_games(&state.pool).await?;

    tracing::debug!(count = games.len(), "fetched games list");

    Ok((StatusCode::OK, Json(games)))
}
