use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use validator::Validate;

use crate::{AppState, middleware::AuthUser};

// ── Request types ───────────────────────────────────────────────────────────

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(
        min = 3,
        max = 30,
        message = "Username must be between 3 and 30 characters"
    ))]
    pub username: String,

    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

// ── Response types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AuthResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

// ── Router ────────────────────────────────────────────────────────────────
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
}

// ── Handlers ────────────────────────────────────────────────────────────────

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let password = payload.password.clone();
    let hashed = tokio::task::spawn_blocking(move || auth::password::hash_password(&password))
        .await
        .map_err(|_| AppError::InternalError)??;

    // TODO: Create the user
    let user =
        db::queries::create_user(&state.pool, &payload.username, &payload.email, &hashed).await?;

    tracing::info!("New user registered: {} ({})", user.id, user.email);

    let token = auth::jwt::create_access_token(
        &user.id.to_string(),
        &state.config.jwt_secret,
        state.config.jwt_access_token_expiry_minutes,
    )?;

    let cookie = build_access_cookie(token, state.config.jwt_access_token_expiry_minutes);

    Ok((
        StatusCode::CREATED,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let user = db::queries::find_user_by_email(&state.pool, &payload.email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let password = payload.password.clone();
    let hash = user.password_hash.clone();
    let is_valid =
        tokio::task::spawn_blocking(move || auth::password::verify_password(&password, &hash))
            .await
            .map_err(|_| AppError::InternalError)??;

    if !is_valid {
        tracing::warn!("Failed login attempt for user: {}", user.id);
        return Err(AppError::InvalidCredentials);
    }

    let token = auth::jwt::create_access_token(
        &user.id.to_string(),
        &state.config.jwt_secret,
        state.config.jwt_access_token_expiry_minutes,
    )?;

    let cookie = build_access_cookie(token, state.config.jwt_access_token_expiry_minutes);

    Ok((
        StatusCode::OK,
        [(header::SET_COOKIE, cookie.to_string())],
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let user = db::queries::find_user_by_id(&state.pool, auth_user.id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    tracing::debug!("Fetched profile for user: {}", auth_user.id);

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// POST /api/v1/auth/logout
pub async fn logout() -> impl IntoResponse {
    let cookie = build_logout_cookie();
    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, cookie.to_string())],
    )
}

// ── Private helpers ─────────────────────────────────────────────────────────

fn build_access_cookie(
    token: String,
    expiry_minutes: i64,
) -> axum_extra::extract::cookie::Cookie<'static> {
    use axum_extra::extract::cookie::{Cookie, SameSite};

    Cookie::build(("access_token", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .max_age(time::Duration::minutes(expiry_minutes))
        .build()
}

fn build_logout_cookie() -> axum_extra::extract::cookie::Cookie<'static> {
    use axum_extra::extract::cookie::{Cookie, SameSite};

    Cookie::build(("access_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .max_age(time::Duration::ZERO)
        .build()
}
