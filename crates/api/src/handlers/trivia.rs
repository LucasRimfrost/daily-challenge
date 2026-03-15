use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use db::{
    models::Difficulty,
    queries::{
        create_trivia_submission, find_trivia_challenge_by_date, find_trivia_challenge_by_id,
        find_trivia_challenge_history, find_trivia_past_challenges,
        find_trivia_submissions_by_user_and_challenge, increment_trivia_attempts,
        upsert_trivia_stats_on_solve,
    },
};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use uuid::Uuid;

use crate::{AppState, middleware::AuthUser};

// ── Request types ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SubmitRequest {
    pub answer: String,
    pub challenge_id: Uuid,
}

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct TriviaChallengeResponse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub hint: Option<String>,
    pub max_attempts: i32,
    pub scheduled_date: chrono::NaiveDate,
    pub attempts_used: i32,
    pub is_solved: bool,
    pub correct_answer: Option<String>,
}

#[derive(Serialize)]
pub struct SubmitResponse {
    pub is_correct: bool,
    pub attempt_number: i32,
    pub attempts_remaining: i32,
    pub hint: Option<String>,
}

// ── Query params ──────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct HistoryParams {
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct ArchiveEntry {
    pub id: Uuid,
    pub title: String,
    pub difficulty: Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub is_solved: bool,
    pub attempts_used: i32,
    pub max_attempts: i32,
}

// ── Router ──────────────────────────────────────────────────────────
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/today", get(today))
        .route("/submit", post(submit))
        .route("/history", get(history))
        .route("/archive", get(archive))
        .route("/{date}", get(by_date))
}

// ── Handlers ──────────────────────────────────────────────────────────

/// GET /api/v1/challenge/today
pub async fn today(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let user_id = auth_user.id;
    let date = Utc::now().date_naive();

    tracing::debug!(user_id = %user_id, %date, "fetching today's trivia challenge");

    let challenge = find_trivia_challenge_by_date(&state.pool, date)
        .await?
        .ok_or_else(|| {
            tracing::warn!(%date, "no trivia challenge scheduled for today");
            AppError::NotFound
        })?;

    let submissions =
        find_trivia_submissions_by_user_and_challenge(&state.pool, user_id, challenge.id).await?;

    let is_solved = submissions.iter().any(|s| s.is_correct);
    let attempts_used = submissions.len() as i32;
    let is_exhausted = attempts_used >= challenge.max_attempts;

    let correct_answer = if is_solved || is_exhausted {
        Some(challenge.expected_answer)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(TriviaChallengeResponse {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            difficulty: challenge.difficulty,
            hint: challenge.hint,
            max_attempts: challenge.max_attempts,
            scheduled_date: challenge.scheduled_date,
            attempts_used,
            is_solved,
            correct_answer,
        }),
    ))
}

/// POST /api/v1/challenge/submit
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
        "trivia submission attempt"
    );

    let challenge = find_trivia_challenge_by_id(&state.pool, challenge_id)
        .await?
        .ok_or_else(|| {
            tracing::warn!(challenge_id = %challenge_id, "trivia challenge not found");
            AppError::NotFound
        })?;

    let submissions =
        find_trivia_submissions_by_user_and_challenge(&state.pool, user_id, challenge.id).await?;

    if submissions.iter().any(|s| s.is_correct) {
        tracing::debug!(user_id = %user_id, challenge_id = %challenge_id, "rejected — already solved");
        return Err(AppError::BadRequest("Challenge already solved".into()));
    }

    if submissions.len() as i32 >= challenge.max_attempts {
        tracing::debug!(user_id = %user_id, challenge_id = %challenge_id, "rejected — no attempts remaining");
        return Err(AppError::BadRequest("No attempts remaining".into()));
    }

    let attempt_number = submissions.len() as i32 + 1;
    let user_answer = payload.answer.trim().to_lowercase();
    let challenge_answer = challenge.expected_answer.trim().to_lowercase();
    let is_correct = user_answer == challenge_answer;

    create_trivia_submission(
        &state.pool,
        user_id,
        challenge.id,
        &payload.answer,
        is_correct,
        attempt_number,
    )
    .await?;

    increment_trivia_attempts(&state.pool, user_id).await?;

    if is_correct {
        let today = Utc::now().date_naive();
        if challenge.scheduled_date == today {
            upsert_trivia_stats_on_solve(&state.pool, user_id, today).await?;
        }
        tracing::info!(
            user_id = %user_id,
            challenge_id = %challenge_id,
            attempt_number,
            "trivia challenge solved"
        );
    } else {
        tracing::debug!(
            user_id = %user_id,
            challenge_id = %challenge_id,
            attempt_number,
            "incorrect trivia answer"
        );
    }

    let attempts_remaining = challenge.max_attempts - attempt_number;
    let hint = if !is_correct && attempt_number >= 3 {
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

/// GET /api/v1/challenge/history
pub async fn history(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<HistoryParams>,
) -> AppResult<impl IntoResponse> {
    let limit = params.limit.unwrap_or(30);

    tracing::debug!(user_id = %auth_user.id, limit, "fetching trivia history");

    let history = find_trivia_challenge_history(&state.pool, auth_user.id, limit).await?;

    Ok((StatusCode::OK, Json(history)))
}

/// GET /api/v1/challenge/archive
pub async fn archive(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let today = Utc::now().date_naive();

    tracing::debug!(user_id = %auth_user.id, "fetching trivia archive");

    let rows = find_trivia_past_challenges(&state.pool, auth_user.id, today).await?;

    let entries: Vec<ArchiveEntry> = rows
        .into_iter()
        .map(|r| ArchiveEntry {
            id: r.id,
            title: r.title,
            difficulty: r.difficulty,
            scheduled_date: r.scheduled_date,
            is_solved: r.is_solved,
            attempts_used: r.attempts_used as i32,
            max_attempts: r.max_attempts,
        })
        .collect();

    Ok((StatusCode::OK, Json(entries)))
}

/// GET /api/v1/challenge/:date
pub async fn by_date(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Path(date): Path<chrono::NaiveDate>,
) -> AppResult<impl IntoResponse> {
    tracing::debug!(user_id = %auth_user.id, %date, "fetching trivia by date");

    let challenge = find_trivia_challenge_by_date(&state.pool, date)
        .await?
        .ok_or_else(|| {
            tracing::warn!(%date, "no trivia challenge found for date");
            AppError::NotFound
        })?;

    let submissions =
        find_trivia_submissions_by_user_and_challenge(&state.pool, auth_user.id, challenge.id)
            .await?;

    let is_solved = submissions.iter().any(|s| s.is_correct);
    let attempts_used = submissions.len() as i32;
    let is_exhausted = attempts_used >= challenge.max_attempts;

    let correct_answer = if is_solved || is_exhausted {
        Some(challenge.expected_answer)
    } else {
        None
    };

    Ok((
        StatusCode::OK,
        Json(TriviaChallengeResponse {
            id: challenge.id,
            title: challenge.title,
            description: challenge.description,
            difficulty: challenge.difficulty,
            hint: challenge.hint,
            max_attempts: challenge.max_attempts,
            scheduled_date: challenge.scheduled_date,
            attempts_used,
            is_solved,
            correct_answer,
        }),
    ))
}
