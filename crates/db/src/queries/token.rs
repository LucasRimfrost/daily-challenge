use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{PasswordResetToken, RefreshToken};

// ── Refresh tokens ──────────────────────────────────────────────────────

/// Stores a hashed refresh token in the database with the given expiration.
#[tracing::instrument(skip(pool, token_hash))]
pub async fn create_refresh_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token_hash,
        expires_at,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%user_id, "refresh token created");
    Ok(())
}

/// Looks up a refresh token row by its SHA-256 hash.
#[tracing::instrument(skip(pool, token_hash))]
pub async fn find_refresh_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> AppResult<Option<RefreshToken>> {
    let result = sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT id, user_id, token_hash, expires_at, created_at, revoked_at
        FROM refresh_tokens
        WHERE token_hash = $1
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "refresh token lookup");
    Ok(result)
}

/// Marks a single refresh token as revoked by setting `revoked_at`.
#[tracing::instrument(skip(pool))]
pub async fn revoke_refresh_token(pool: &PgPool, token_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE id = $1 AND revoked_at IS NULL
        "#,
        token_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%token_id, "refresh token revoked");
    Ok(())
}

/// Revokes every active refresh token for a user (e.g. on password change or
/// detected token reuse).
#[tracing::instrument(skip(pool))]
pub async fn revoke_all_user_refresh_tokens(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    let result = sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE user_id = $1 AND revoked_at IS NULL
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, revoked = result.rows_affected(), "all refresh tokens revoked");
    Ok(())
}

// ── Password reset tokens ───────────────────────────────────────────────

/// Creates a password-reset token, revoking any previous unused tokens for the
/// same user first.
#[tracing::instrument(skip(pool, token_hash))]
pub async fn create_password_reset_token(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: chrono::DateTime<chrono::Utc>,
) -> AppResult<()> {
    // Revoke any existing unused reset tokens for this user
    sqlx::query!(
        r#"
        UPDATE password_reset_tokens
        SET used_at = now()
        WHERE user_id = $1 AND used_at IS NULL
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    sqlx::query!(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user_id,
        token_hash,
        expires_at,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%user_id, "password reset token created");
    Ok(())
}

/// Looks up a password-reset token by its SHA-256 hash.
#[tracing::instrument(skip(pool, token_hash))]
pub async fn find_password_reset_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> AppResult<Option<PasswordResetToken>> {
    let result = sqlx::query_as!(
        PasswordResetToken,
        r#"
        SELECT id, user_id, token_hash, expires_at, created_at, used_at
        FROM password_reset_tokens
        WHERE token_hash = $1
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "password reset token lookup");
    Ok(result)
}

/// Marks a password-reset token as consumed by setting `used_at`.
#[tracing::instrument(skip(pool))]
pub async fn mark_password_reset_token_used(pool: &PgPool, token_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE password_reset_tokens
        SET used_at = now()
        WHERE id = $1 AND used_at IS NULL
        "#,
        token_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%token_id, "password reset token marked as used");
    Ok(())
}

/// Atomically resets a user's password, marks the reset token as used, and
/// revokes all refresh tokens — all within a single transaction.
///
/// Prevents partial state corruption if the server crashes mid-sequence.
#[tracing::instrument(skip(pool, password_hash))]
pub async fn reset_password_atomic(
    pool: &PgPool,
    user_id: Uuid,
    token_id: Uuid,
    password_hash: &str,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    // 1. Update the password.
    sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $2
        WHERE id = $1
        "#,
        user_id,
        password_hash,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // 2. Mark the reset token as used.
    sqlx::query!(
        r#"
        UPDATE password_reset_tokens
        SET used_at = now()
        WHERE id = $1 AND used_at IS NULL
        "#,
        token_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // 3. Revoke all refresh tokens — force re-login everywhere.
    sqlx::query!(
        r#"
        UPDATE refresh_tokens
        SET revoked_at = now()
        WHERE user_id = $1 AND revoked_at IS NULL
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    tracing::info!(%user_id, %token_id, "password reset completed (atomic)");
    Ok(())
}
