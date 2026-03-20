use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use db::queries::{
    create_refresh_token, find_code_output_stats, find_refresh_token_by_hash, find_trivia_stats,
    revoke_all_user_refresh_tokens, revoke_refresh_token,
};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use validator::Validate;

use crate::{AppState, middleware::AuthUser};

// ── Request types ───────────────────────────────────────────────────────────

/// Payload for `POST /auth/register`.
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

/// Payload for `POST /auth/login`.
#[derive(Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
}

/// Payload for `POST /auth/forgot-password`.
#[derive(Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
}

/// Payload for `POST /auth/reset-password`.
#[derive(Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub new_password: String,
}

/// Payload for `PATCH /auth/profile`.
#[derive(Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(
        min = 3,
        max = 30,
        message = "Username must be between 3 and 30 characters"
    ))]
    pub username: String,
}

/// Payload for `PATCH /auth/email`. Requires re-authentication via `current_password`.
#[derive(Deserialize, Validate)]
pub struct UpdateEmailRequest {
    #[validate(email(message = "Invalid email format"))]
    pub new_email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub current_password: String,
}

/// Payload for `PATCH /auth/password`. Requires re-authentication via `current_password`.
#[derive(Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    #[validate(length(min = 1, message = "Current password cannot be empty"))]
    pub current_password: String,
    #[validate(length(min = 8, message = "New password must be at least 8 characters long"))]
    pub new_password: String,
}

// ── Response types ──────────────────────────────────────────────────────────

/// Returned after registration, login, and profile updates.
#[derive(Serialize)]
pub struct AuthResponse {
    pub id: String,
    pub username: String,
    pub email: String,
}

/// Returned by `GET /auth/me` — user profile combined with per-game stats.
#[derive(Serialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub trivia_stats: StatsResponse,
    pub code_output_stats: StatsResponse,
}

/// Aggregate game statistics for a single user and game type.
#[derive(Serialize, Default)]
pub struct StatsResponse {
    pub current_streak: i32,
    pub longest_streak: i32,
    pub total_solved: i32,
    pub total_attempts: i32,
}

// ── Router ────────────────────────────────────────────────────────────────
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/me", get(me))
}

// ── Handlers ────────────────────────────────────────────────────────────────

/// POST /api/v1/auth/register
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    tracing::info!(email = %payload.email, username = %payload.username, "registration attempt");

    let password = payload.password.clone();
    let hashed = tokio::task::spawn_blocking(move || auth::password::hash_password(&password))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "password hashing task panicked");
            AppError::InternalError
        })??;

    let user =
        db::queries::create_user(&state.pool, &payload.username, &payload.email, &hashed).await?;

    tracing::info!(user_id = %user.id, email = %user.email, "user registered");

    let (access_cookie, refresh_cookie) = issue_tokens(&state, user.id).await?;

    Ok((
        StatusCode::CREATED,
        AppendHeaders([
            (header::SET_COOKIE, access_cookie.to_string()),
            (header::SET_COOKIE, refresh_cookie.to_string()),
        ]),
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

    tracing::info!(email = %payload.email, "login attempt");

    let user = db::queries::find_user_by_email(&state.pool, &payload.email)
        .await?
        .ok_or_else(|| {
            tracing::warn!(email = %payload.email, "login failed — unknown email");
            AppError::InvalidCredentials
        })?;

    let user_id = user.id;
    let password = payload.password.clone();
    let hash = user.password_hash.clone();
    let is_valid =
        tokio::task::spawn_blocking(move || auth::password::verify_password(&password, &hash))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "password verification task panicked");
                AppError::InternalError
            })??;

    if !is_valid {
        tracing::warn!(user_id = %user_id, "login failed — wrong password");
        return Err(AppError::InvalidCredentials);
    }

    let (access_cookie, refresh_cookie) = issue_tokens(&state, user.id).await?;

    tracing::info!(user_id = %user.id, "login successful");

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (header::SET_COOKIE, access_cookie.to_string()),
            (header::SET_COOKIE, refresh_cookie.to_string()),
        ]),
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// POST /api/v1/auth/refresh
///
/// Validates the refresh token cookie, rotates it (revoke old, issue new),
/// and returns a fresh access token. No request body needed.
pub async fn refresh(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    // Extract refresh token from cookie
    let refresh_cookie = jar.get("refresh_token").ok_or_else(|| {
        tracing::debug!("refresh rejected — no refresh_token cookie");
        AppError::Unauthorized
    })?;

    let raw_token = refresh_cookie.value();
    let token_hash = auth::token::hash_refresh_token(raw_token);

    // Look up the token in the database
    let stored = find_refresh_token_by_hash(&state.pool, &token_hash)
        .await?
        .ok_or_else(|| {
            tracing::warn!("refresh rejected — token not found in database");
            AppError::Unauthorized
        })?;

    // Check if revoked
    if stored.revoked_at.is_some() {
        // Possible token reuse attack — revoke ALL tokens for this user
        tracing::warn!(
            user_id = %stored.user_id,
            token_id = %stored.id,
            "refresh token reuse detected — revoking all user tokens"
        );
        revoke_all_user_refresh_tokens(&state.pool, stored.user_id).await?;
        return Err(AppError::Unauthorized);
    }

    // Check if expired
    if stored.expires_at < chrono::Utc::now() {
        tracing::debug!(user_id = %stored.user_id, "refresh rejected — token expired");
        revoke_refresh_token(&state.pool, stored.id).await?;
        return Err(AppError::Unauthorized);
    }

    // Revoke the old token (rotation)
    revoke_refresh_token(&state.pool, stored.id).await?;

    // Issue new token pair
    let (access_cookie, refresh_cookie) = issue_tokens(&state, stored.user_id).await?;

    tracing::info!(user_id = %stored.user_id, "tokens refreshed");

    Ok((
        StatusCode::OK,
        AppendHeaders([
            (header::SET_COOKIE, access_cookie.to_string()),
            (header::SET_COOKIE, refresh_cookie.to_string()),
        ]),
    ))
}

/// POST /api/v1/auth/forgot-password
///
/// Always returns 200 regardless of whether the email exists.
/// This prevents email enumeration attacks.
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    tracing::info!(email = %payload.email, "password reset requested");

    // Look up user — but don't reveal whether the email exists
    let user = db::queries::find_user_by_email(&state.pool, &payload.email).await?;

    if let Some(user) = user {
        // Generate reset token
        let raw_token = auth::token::generate_refresh_token();
        let token_hash = auth::token::hash_refresh_token(&raw_token);
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);

        db::queries::create_password_reset_token(&state.pool, user.id, &token_hash, expires_at)
            .await?;

        // In production, send an email. For now, log the reset link.
        let reset_link = format!("http://localhost:3000/reset-password?token={}", raw_token);

        tracing::info!(
            user_id = %user.id,
            "password reset link generated (dev only): {}",
            reset_link
        );

        // TODO: Replace with real email sending
        // email::send_password_reset(&user.email, &reset_link).await?;
    } else {
        tracing::debug!(email = %payload.email, "password reset for unknown email — ignoring silently");
    }

    // Always return success to prevent email enumeration
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "If an account with that email exists, a password reset link has been sent."
        })),
    ))
}

/// POST /api/v1/auth/reset-password
pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let token_hash = auth::token::hash_refresh_token(&payload.token);

    // Find the token
    let stored = db::queries::find_password_reset_token_by_hash(&state.pool, &token_hash)
        .await?
        .ok_or_else(|| {
            tracing::warn!("password reset failed — token not found");
            AppError::BadRequest("Invalid or expired reset token".to_string())
        })?;

    // Check if already used
    if stored.used_at.is_some() {
        tracing::warn!(token_id = %stored.id, "password reset failed — token already used");
        return Err(AppError::BadRequest(
            "This reset link has already been used".to_string(),
        ));
    }

    // Check if expired
    if stored.expires_at < chrono::Utc::now() {
        tracing::warn!(token_id = %stored.id, "password reset failed — token expired");
        db::queries::mark_password_reset_token_used(&state.pool, stored.id).await?;
        return Err(AppError::BadRequest(
            "This reset link has expired".to_string(),
        ));
    }

    // Hash the new password
    let new_password = payload.new_password.clone();
    let hashed = tokio::task::spawn_blocking(move || auth::password::hash_password(&new_password))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "password hashing task panicked");
            AppError::InternalError
        })??;

    // Atomically: update password + mark token used + revoke all refresh tokens.
    // Wrapped in a single transaction to prevent partial state on crash.
    db::queries::reset_password_atomic(&state.pool, stored.user_id, stored.id, &hashed).await?;

    tracing::info!(user_id = %stored.user_id, "password reset successful");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Password has been reset. Please log in with your new password."
        })),
    ))
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let user_id = auth_user.id;

    let user = db::queries::find_user_profile_by_id(&state.pool, user_id)
        .await?
        .ok_or_else(|| {
            tracing::warn!(user_id = %user_id, "authenticated user not found in database");
            AppError::Unauthorized
        })?;

    let trivia_stats = match find_trivia_stats(&state.pool, user_id).await? {
        Some(s) => StatsResponse {
            current_streak: s.current_streak,
            longest_streak: s.longest_streak,
            total_solved: s.total_solved,
            total_attempts: s.total_attempts,
        },
        None => StatsResponse::default(),
    };

    let code_output_stats = match find_code_output_stats(&state.pool, user_id).await? {
        Some(s) => StatsResponse {
            current_streak: s.current_streak,
            longest_streak: s.longest_streak,
            total_solved: s.total_solved,
            total_attempts: s.total_attempts,
        },
        None => StatsResponse::default(),
    };

    tracing::debug!(user_id = %user_id, "profile fetched");

    Ok((
        StatusCode::OK,
        Json(MeResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
            trivia_stats,
            code_output_stats,
        }),
    ))
}

/// POST /api/v1/auth/logout
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<impl IntoResponse> {
    // Revoke the refresh token in the database if present.
    if let Some(refresh_cookie) = jar.get("refresh_token") {
        let token_hash = auth::token::hash_refresh_token(refresh_cookie.value());
        if let Some(stored) = find_refresh_token_by_hash(&state.pool, &token_hash).await?
            && stored.revoked_at.is_none()
        {
            revoke_refresh_token(&state.pool, stored.id).await?;
        }
    }

    tracing::info!("user logged out");

    let access_cookie = build_logout_cookie("access_token", "/");
    let refresh_cookie = build_logout_cookie("refresh_token", "/api/v1/auth");

    Ok((
        StatusCode::NO_CONTENT,
        AppendHeaders([
            (header::SET_COOKIE, access_cookie.to_string()),
            (header::SET_COOKIE, refresh_cookie.to_string()),
        ]),
    ))
}

/// PATCH /api/v1/auth/profile
pub async fn update_profile(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let user_id = auth_user.id;
    tracing::info!(%user_id, username = %payload.username, "profile update attempt");

    let user = db::queries::update_username(&state.pool, user_id, &payload.username).await?;

    tracing::info!(%user_id, "profile updated");

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// PATCH /api/v1/auth/email
pub async fn update_email(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateEmailRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let user_id = auth_user.id;
    tracing::info!(%user_id, new_email = %payload.new_email, "email update attempt");

    let user = db::queries::find_user_by_id(&state.pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let password = payload.current_password.clone();
    let hash = user.password_hash.clone();
    let is_valid =
        tokio::task::spawn_blocking(move || auth::password::verify_password(&password, &hash))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "password verification task panicked");
                AppError::InternalError
            })??;

    if !is_valid {
        tracing::warn!(%user_id, "email update failed — wrong password");
        return Err(AppError::InvalidCredentials);
    }

    let user = db::queries::update_email(&state.pool, user_id, &payload.new_email).await?;

    tracing::info!(%user_id, "email updated");

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            id: user.id.to_string(),
            username: user.username,
            email: user.email,
        }),
    ))
}

/// PATCH /api/v1/auth/password
pub async fn update_password(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    payload.validate()?;

    let user_id = auth_user.id;
    tracing::info!(%user_id, "password change attempt");

    let user = db::queries::find_user_by_id(&state.pool, user_id)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let current = payload.current_password.clone();
    let hash = user.password_hash.clone();
    let is_valid =
        tokio::task::spawn_blocking(move || auth::password::verify_password(&current, &hash))
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "password verification task panicked");
                AppError::InternalError
            })??;

    if !is_valid {
        tracing::warn!(%user_id, "password change failed — wrong current password");
        return Err(AppError::InvalidCredentials);
    }

    let new_password = payload.new_password.clone();
    let hashed = tokio::task::spawn_blocking(move || auth::password::hash_password(&new_password))
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "password hashing task panicked");
            AppError::InternalError
        })??;

    db::queries::update_user_password(&state.pool, user_id, &hashed).await?;

    // Revoke all refresh tokens — force re-login on other devices
    revoke_all_user_refresh_tokens(&state.pool, user_id).await?;

    tracing::info!(%user_id, "password changed");

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Password updated successfully"
        })),
    ))
}

// ── Private helpers ─────────────────────────────────────────────────────────

/// Issue both an access token (JWT) and a refresh token (random + hashed in DB).
/// Returns both cookies ready to be set on the response.
async fn issue_tokens(
    state: &AppState,
    user_id: uuid::Uuid,
) -> AppResult<(
    axum_extra::extract::cookie::Cookie<'static>,
    axum_extra::extract::cookie::Cookie<'static>,
)> {
    // Access token (short-lived JWT in cookie)
    let access_token = auth::jwt::create_access_token(
        &user_id.to_string(),
        &state.config.jwt_secret,
        state.config.jwt_access_token_expiry_minutes,
    )?;

    // Refresh token (long-lived random string, hash stored in DB)
    let raw_refresh = auth::token::generate_refresh_token();
    let refresh_hash = auth::token::hash_refresh_token(&raw_refresh);
    let expires_at =
        chrono::Utc::now() + chrono::Duration::days(state.config.refresh_token_expiry_days);

    create_refresh_token(&state.pool, user_id, &refresh_hash, expires_at).await?;

    let access_cookie =
        build_access_cookie(access_token, state.config.jwt_access_token_expiry_minutes);
    let refresh_cookie = build_refresh_cookie(raw_refresh, state.config.refresh_token_expiry_days);

    Ok((access_cookie, refresh_cookie))
}

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

fn build_refresh_cookie(
    token: String,
    expiry_days: i64,
) -> axum_extra::extract::cookie::Cookie<'static> {
    use axum_extra::extract::cookie::{Cookie, SameSite};

    Cookie::build(("refresh_token", token))
        .path("/api/v1/auth") // Only sent to auth endpoints, not every request
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .max_age(time::Duration::days(expiry_days))
        .build()
}

fn build_logout_cookie(name: &str, path: &str) -> axum_extra::extract::cookie::Cookie<'static> {
    use axum_extra::extract::cookie::{Cookie, SameSite};

    Cookie::build((name.to_owned(), String::new()))
        .path(path.to_owned())
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(!cfg!(debug_assertions))
        .max_age(time::Duration::ZERO)
        .build()
}
