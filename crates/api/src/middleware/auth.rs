use auth::jwt;
use axum::extract::FromRequestParts;
use axum_extra::extract::CookieJar;
use shared::error::AppError;
use uuid::Uuid;

use crate::AppState;

/// Axum extractor that validates the `access_token` cookie and provides
/// the authenticated user's ID.
///
/// Add this to a handler's arguments to require authentication:
///
/// ```ignore
/// async fn protected(auth: AuthUser) { /* auth.id is a valid Uuid */ }
/// ```
pub struct AuthUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);

        let cookie = jar.get("access_token").ok_or_else(|| {
            tracing::debug!("auth rejected — no access_token cookie");
            AppError::Unauthorized
        })?;

        let token = cookie.value();

        let claims = jwt::validate_token(token, &state.config.jwt_secret)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
            tracing::error!(
                error = %e,
                sub = %claims.sub,
                "auth rejected — malformed UUID in JWT sub claim"
            );
            AppError::Unauthorized
        })?;

        tracing::debug!(user_id = %user_id, "user authenticated");
        Ok(AuthUser { id: user_id })
    }
}
