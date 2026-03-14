use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};

const ISSUER: &str = "daily-challenge";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: String, // user id
    pub exp: usize,  // expiration
    pub iat: usize,  // issued at
    pub iss: String, // issuer (daily_challenge/domain)
}

pub fn create_access_token(user_id: &str, secret: &str, expiry_minutes: i64) -> AppResult<String> {
    let now = Utc::now();
    let exp = (now + chrono::Duration::minutes(expiry_minutes)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now.timestamp() as usize,
        iss: ISSUER.to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to encode JWT: {}", e);
        AppError::InternalError
    })
}

pub fn validate_token(token: &str, secret: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.set_issuer(&[ISSUER]);

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| {
        tracing::warn!("JWT validation failed: {}", e);
        AppError::Unauthorized
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret-key-for-unit-tests";
    const USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

    #[test]
    fn create_access_token_produces_valid_token() {
        let token = create_access_token(USER_ID, SECRET, 60).unwrap();
        // JWT has three dot-separated base64 segments
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn validate_token_accepts_valid_token() {
        let token = create_access_token(USER_ID, SECRET, 60).unwrap();
        let claims = validate_token(&token, SECRET).unwrap();

        assert_eq!(claims.sub, USER_ID);
        assert_eq!(claims.iss, "daily-challenge");
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn validate_token_rejects_expired_token() {
        // -5 minutes ensures it's well past jsonwebtoken's default 60s leeway
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
