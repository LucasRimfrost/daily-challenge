use sha2::{Digest, Sha256};

/// Generate a cryptographically random refresh token (base64url, 32 bytes).
pub fn generate_refresh_token() -> String {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("Failed to generate random bytes");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Hash a refresh token with SHA-256 for safe storage in the database.
pub fn hash_refresh_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_unique_tokens() {
        let t1 = generate_refresh_token();
        let t2 = generate_refresh_token();
        assert_ne!(t1, t2);
        // Base64url encoded 32 bytes = 43 characters
        assert_eq!(t1.len(), 43);
    }

    #[test]
    fn hash_is_deterministic() {
        let token = "test-token-123";
        assert_eq!(hash_refresh_token(token), hash_refresh_token(token));
    }
}
