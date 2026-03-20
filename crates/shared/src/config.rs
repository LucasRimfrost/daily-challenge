use std::{env, fmt};

/// Application configuration loaded from environment variables.
///
/// Required variables: `DATABASE_URL`, `JWT_SECRET`,
/// `JWT_ACCESS_TOKEN_EXPIRY_MINUTES`, `REFRESH_TOKEN_EXPIRY_DAYS`,
/// `BACKEND_HOST`, `BACKEND_PORT`.
///
/// Optional variables: `STATIC_DIR` (enables SPA file serving),
/// `CORS_ORIGIN` (defaults to `http://localhost:3000`).
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
    pub host: String,
    pub port: String,
    pub static_dir: Option<String>,
    pub cors_origin: Option<String>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"[REDACTED]")
            .field("jwt_secret", &"[REDACTED]")
            .field(
                "jwt_access_token_expiry_minutes",
                &self.jwt_access_token_expiry_minutes,
            )
            .field("refresh_token_expiry_days", &self.refresh_token_expiry_days)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("static_dir", &self.static_dir)
            .field("cors_origin", &self.cors_origin)
            .finish()
    }
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns [`env::VarError`] if any required variable is missing.
    ///
    /// # Panics
    ///
    /// Panics if `JWT_ACCESS_TOKEN_EXPIRY_MINUTES` or `REFRESH_TOKEN_EXPIRY_DAYS`
    /// cannot be parsed as integers.
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_access_token_expiry_minutes: env::var("JWT_ACCESS_TOKEN_EXPIRY_MINUTES")?
                .parse()
                .expect("JWT_ACCESS_TOKEN_EXPIRY_MINUTES must be a valid number"),
            refresh_token_expiry_days: env::var("REFRESH_TOKEN_EXPIRY_DAYS")?
                .parse()
                .expect("JWT_ACCESS_TOKEN_EXPIRY_MINUTES must be a valid number"),
            host: env::var("BACKEND_HOST")?,
            port: env::var("BACKEND_PORT")?,
            static_dir: env::var("STATIC_DIR").ok(),
            cors_origin: env::var("CORS_ORIGIN").ok(),
        })
    }
}
