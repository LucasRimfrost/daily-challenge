use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use db::queries::{
    create_code_output_submission_atomic, find_code_output_challenge_by_date,
    find_code_output_challenge_by_id, find_code_output_challenge_history,
    find_code_output_past_challenges, find_code_output_submissions_by_user_and_challenge,
};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};

/// Normalizes code output for comparison by unifying quote style and spacing.
///
/// Converts double quotes to single quotes and removes spaces after `,` and `:`,
/// so that e.g. `["a", "b"]` and `['a', 'b']` and `['a','b']` all match.
fn normalize_output(s: &str) -> String {
    s.trim()
        .replace('"', "'")
        .replace(", ", ",")
        .replace(": ", ":")
}

// ── Request types ──────────────────────────────────────────────────────────

/// Payload for `POST /code-output/submit`.
#[derive(Deserialize)]
pub struct SubmitRequest {
    pub answer: String,
    pub challenge_id: Uuid,
}

// ── Response types ──────────────────────────────────────────────────────────

/// Full challenge view returned to the client, including the code snippet and user progress.
#[derive(Serialize)]
pub struct CodeOutputChallengeResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub language: String,
    pub code_snippet: String,
    pub difficulty: db::models::Difficulty,
    pub hint: Option<String>,
    pub max_attempts: i32,
    pub scheduled_date: chrono::NaiveDate,
    pub attempts_used: i32,
    pub is_solved: bool,
    pub correct_answer: Option<String>,
    pub previous_guesses: Vec<String>,
}

/// Outcome of a submission attempt.
#[derive(Serialize)]
pub struct SubmitResponse {
    pub is_correct: bool,
    pub attempt_number: i32,
    pub attempts_remaining: i32,
    /// Revealed after the second incorrect attempt.
    pub hint: Option<String>,
}

/// Optional query parameters for the history endpoint.
#[derive(Deserialize)]
pub struct HistoryParams {
    pub limit: Option<i64>,
}

/// Summary of a past code-output challenge for the archive view.
#[derive(Serialize)]
pub struct ArchiveEntry {
    pub id: Uuid,
    pub title: String,
    pub language: String,
    pub difficulty: db::models::Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub is_solved: bool,
    pub attempts_used: i32,
    pub max_attempts: i32,
}

// ── Router ──────────────────────────────────────────────────────────

/// Mounts code-output routes: `today`, `submit`, `history`, `archive`, `{date}`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/today", get(today))
        .route("/submit", post(submit))
        .route("/history", get(history))
        .route("/archive", get(archive))
        .route("/{date}", get(by_date))
}

// ── Handlers ──────────────────────────────────────────────────────────

/// GET /api/v1/code-output/today
pub async fn today(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let user_id = auth_user.id;
    let date = Utc::now().date_naive();

    tracing::debug!(user_id = %user_id, %date, "fetching today's code output challenge");

    let challenge = find_code_output_challenge_by_date(&state.pool, date)
        .await?
        .ok_or_else(|| {
            tracing::warn!(%date, "no code output challenge scheduled for today");
            AppError::NotFound
        })?;

    let submissions =
        find_code_output_submissions_by_user_and_challenge(&state.pool, user_id, challenge.id)
            .await?;

    let is_solved = submissions.iter().any(|s| s.is_correct);
    let attempts_used = submissions.len() as i32;
    let is_exhausted = attempts_used >= challenge.max_attempts;

    let previous_guesses: Vec<String> = submissions.iter().map(|s| s.answer.clone()).collect();

    // Don't expose the code_snippet's expected output in the response
    // Only reveal after solving or exhausting attempts
    let correct_answer = if is_solved || is_exhausted {
        Some(challenge.expected_output.clone())
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(CodeOutputChallengeResponse {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            language: challenge.language,
            code_snippet: challenge.code_snippet,
            difficulty: challenge.difficulty,
            hint: challenge.hint,
            max_attempts: challenge.max_attempts,
            scheduled_date: challenge.scheduled_date,
            attempts_used,
            is_solved,
            correct_answer,
            previous_guesses,
        }),
    ))
}

/// POST /api/v1/code-output/submit
pub async fn submit(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<SubmitRequest>,
) -> AppResult<impl IntoResponse> {
    let user_id = auth_user.id;
    let challenge_id = payload.challenge_id;

    tracing::info!(
        user_id = %user_id,
        challenge_id = %challenge_id,
        "code output submission attempt"
    );

    let challenge = find_code_output_challenge_by_id(&state.pool, challenge_id)
        .await?
        .ok_or_else(|| {
            tracing::warn!(challenge_id = %challenge_id, "code output challenge not found");
            AppError::NotFound
        })?;

    // Case-sensitive match with normalized quotes and spacing
    let is_correct =
        normalize_output(&payload.answer) == normalize_output(&challenge.expected_output);

    let today = Utc::now().date_naive();
    let solved_date = if is_correct && challenge.scheduled_date == today {
        Some(today)
    } else {
        None
    };

    // Atomic check-and-insert: locks the challenge row so concurrent
    // requests cannot bypass the attempt limit.
    let submission = create_code_output_submission_atomic(
        &state.pool,
        user_id,
        challenge.id,
        &payload.answer,
        is_correct,
        challenge.max_attempts,
        solved_date,
    )
    .await?;

    let attempt_number = submission.attempt_number;
    let attempts_remaining = challenge.max_attempts - attempt_number;
    let hint = if !is_correct && attempt_number >= 2 {
        challenge.hint
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(SubmitResponse {
            is_correct,
            attempt_number,
            attempts_remaining,
            hint,
        }),
    ))
}

/// GET /api/v1/code-output/history
pub async fn history(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<HistoryParams>,
) -> AppResult<impl IntoResponse> {
    let limit = params.limit.unwrap_or(30);

    tracing::debug!(user_id = %auth_user.id, limit, "fetching code output history");

    let history = find_code_output_challenge_history(&state.pool, auth_user.id, limit).await?;

    Ok((StatusCode::OK, Json(history)))
}

/// GET /api/v1/code-output/archive
pub async fn archive(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let today = Utc::now().date_naive();

    tracing::debug!(user_id = %auth_user.id, "fetching code output archive");

    let rows = find_code_output_past_challenges(&state.pool, auth_user.id, today).await?;

    let entries: Vec<ArchiveEntry> = rows
        .into_iter()
        .map(|r| ArchiveEntry {
            id: r.id,
            title: r.title,
            language: r.language,
            difficulty: r.difficulty,
            scheduled_date: r.scheduled_date,
            is_solved: r.is_solved,
            attempts_used: r.attempts_used as i32,
            max_attempts: r.max_attempts,
        })
        .collect();

    Ok((StatusCode::OK, Json(entries)))
}

/// GET /api/v1/code-output/:date
pub async fn by_date(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(date): Path<chrono::NaiveDate>,
) -> AppResult<impl IntoResponse> {
    tracing::debug!(user_id = %auth_user.id, %date, "fetching code output by date");

    let challenge = find_code_output_challenge_by_date(&state.pool, date)
        .await?
        .ok_or_else(|| {
            tracing::warn!(%date, "no code output challenge found for date");
            AppError::NotFound
        })?;

    let submissions =
        find_code_output_submissions_by_user_and_challenge(&state.pool, auth_user.id, challenge.id)
            .await?;

    let is_solved = submissions.iter().any(|s| s.is_correct);
    let attempts_used = submissions.len() as i32;
    let is_exhausted = attempts_used >= challenge.max_attempts;

    let previous_guesses: Vec<String> = submissions.iter().map(|s| s.answer.clone()).collect();

    let correct_answer = if is_solved || is_exhausted {
        Some(challenge.expected_output.clone())
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(CodeOutputChallengeResponse {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            language: challenge.language,
            code_snippet: challenge.code_snippet,
            difficulty: challenge.difficulty,
            hint: challenge.hint,
            max_attempts: challenge.max_attempts,
            scheduled_date: challenge.scheduled_date,
            attempts_used,
            is_solved,
            correct_answer,
            previous_guesses,
        }),
    ))
}
