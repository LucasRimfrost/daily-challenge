use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use shared::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| {
            tracing::error!("Failed to hash password: {}", e);
            AppError::InternalError
        })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        tracing::error!("Failed to parse password hash: {}", e);
        AppError::InternalError
    })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_produces_valid_argon2_string() {
        let hash = hash_password("test-password-123").unwrap();
        assert!(hash.starts_with("$argon2"));
        // Should be parseable by argon2
        PasswordHash::new(&hash).expect("hash should be a valid PHC string");
    }

    #[test]
    fn correct_password_verifies() {
        let hash = hash_password("correct-horse-battery").unwrap();
        assert!(verify_password("correct-horse-battery", &hash).unwrap());
    }

    #[test]
    fn wrong_password_fails_verification() {
        let hash = hash_password("correct-horse-battery").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }
}
