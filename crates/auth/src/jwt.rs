use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};

const ISSUER: &str = "brainforge";

/// JWT claims payload embedded in every access token.
#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    /// Subject — the user's UUID.
    pub sub: String,
    /// Expiration time (UTC epoch seconds).
    pub exp: usize,
    /// Issued-at time (UTC epoch seconds).
    pub iat: usize,
    /// Issuer identifier (`"brainforge"`).
    pub iss: String,
}

/// Creates a signed JWT access token for the given user.
///
/// # Errors
///
/// Returns [`AppError::InternalError`] if JWT encoding fails.
#[tracing::instrument(skip(secret))]
pub fn create_access_token(user_id: &str, secret: &str, expiry_minutes: i64) -> AppResult<String> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::minutes(expiry_minutes)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now.timestamp() as usize,
        iss: ISSUER.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!(error = %e, "JWT encoding failed");
        AppError::InternalError
    })?;

    tracing::debug!(user_id, expiry_minutes, "access token issued");
    Ok(token)
}

/// Validates a JWT access token and returns its [`Claims`].
///
/// Checks the signature, expiration, and issuer (`"brainforge"`).
///
/// # Errors
///
/// Returns [`AppError::Unauthorized`] if the token is invalid, expired, or
/// signed with a different secret.
#[tracing::instrument(skip(token, secret), fields(token_len = token.len()))]
pub fn validate_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::warn!(error = %e, "JWT validation failed");
        AppError::Unauthorized
    })?;

    tracing::debug!(user_id = %claims.sub, "token validated");
    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-key-for-unit-tests";
    const USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn create_access_token_produces_valid_token() {
        let token = create_access_token(USER_ID, SECRET, 60).unwrap();
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn validate_token_accepts_valid_token() {
        let token = create_access_token(USER_ID, SECRET, 60).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();
        assert_eq!(claims.sub, USER_ID);
        assert_eq!(claims.iss, "brainforge");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn validate_token_rejects_expired_token() {
        let token = create_access_token(USER_ID, SECRET, -5).unwrap();
        let result = validate_token(&token, SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn validate_token_rejects_wrong_secret() {
        let token = create_access_token(USER_ID, SECRET, 60).unwrap();
        let result = validate_token(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn validate_token_rejects_malformed_string() {
        let result = validate_token("not.a.jwt", SECRET);
        assert!(result.is_err());
        let result = validate_token("total-garbage", SECRET);
        assert!(result.is_err());
    }
}
