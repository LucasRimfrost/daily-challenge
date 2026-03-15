use axum::{
    Json, Router,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use db::queries::{
    create_refresh_token, find_refresh_token_by_hash, find_user_stats,
    revoke_all_user_refresh_tokens, revoke_refresh_token,
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

#[derive(Serialize)]
pub struct MeResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub stats: StatsResponse,
}

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

    let token = auth::jwt::create_access_token(
        &user.id.to_string(),
        &state.config.jwt_secret,
        state.config.jwt_access_token_expiry_minutes,
    )?;

    let cookie = build_access_cookie(token, state.config.jwt_access_token_expiry_minutes);

    tracing::info!(user_id = %user.id, "login successful");

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
        [
            (header::SET_COOKIE, access_cookie.to_string()),
            (header::SET_COOKIE, refresh_cookie.to_string()),
        ],
    ))
}

/// GET /api/v1/auth/me
pub async fn me(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> AppResult<impl IntoResponse> {
    let user_id = auth_user.id;

    let user = db::queries::find_user_by_id(&state.pool, user_id)
        .await?
        .ok_or_else(|| {
            tracing::warn!(user_id = %user_id, "authenticated user not found in database");
            AppError::Unauthorized
        })?;

    let stats = match find_user_stats(&state.pool, user_id).await? {
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
            stats,
        }),
    ))
}

/// POST /api/v1/auth/logout
pub async fn logout() -> impl IntoResponse {
    tracing::info!("user logged out");

    let cookie = build_logout_cookie();
    (
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, cookie.to_string())],
    )
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
