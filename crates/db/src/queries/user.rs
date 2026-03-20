use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{User, UserProfile};

// ── Users ───────────────────────────────────────────────────────────────────

/// Inserts a new user and returns the created row.
///
/// # Errors
///
/// Returns [`AppError::Conflict`] if the email or username already exists.
#[tracing::instrument(skip(pool, password_hash), fields(email = %email))]
pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id, username, email, password_hash, created_at
        "#,
        username,
        email,
        password_hash
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(user_id = %user.id, "user created");
    Ok(user)
}

/// Looks up a user by email address.
#[tracing::instrument(skip(pool))]
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, created_at
        FROM users
        WHERE email = $1
        "#,
        email
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "user lookup by email");
    Ok(result)
}

/// Looks up a user by primary key.
#[tracing::instrument(skip(pool))]
pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
    let result = sqlx::query_as!(
        User,
        r#"
        SELECT id, username, email, password_hash, created_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "user lookup by id");
    Ok(result)
}

/// Looks up a user's profile (without password_hash) by primary key.
#[tracing::instrument(skip(pool))]
pub async fn find_user_profile_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<UserProfile>> {
    let result = sqlx::query_as!(
        UserProfile,
        r#"
        SELECT id, username, email, created_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "user profile lookup by id");
    Ok(result)
}

/// Updates a user's display name and returns the modified row.
///
/// # Errors
///
/// Returns [`AppError::Conflict`] if the new username is already taken.
#[tracing::instrument(skip(pool))]
pub async fn update_username(pool: &PgPool, user_id: Uuid, username: &str) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET username = $2
        WHERE id = $1
        RETURNING id, username, email, password_hash, created_at
        "#,
        user_id,
        username,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, %username, "username updated");
    Ok(user)
}

/// Updates a user's email address and returns the modified row.
///
/// # Errors
///
/// Returns [`AppError::Conflict`] if the new email is already registered.
#[tracing::instrument(skip(pool))]
pub async fn update_email(pool: &PgPool, user_id: Uuid, email: &str) -> AppResult<User> {
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET email = $2
        WHERE id = $1
        RETURNING id, username, email, password_hash, created_at
        "#,
        user_id,
        email,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, %email, "email updated");
    Ok(user)
}

/// Replaces a user's password hash.
#[tracing::instrument(skip(pool, password_hash))]
pub async fn update_user_password(
    pool: &PgPool,
    user_id: Uuid,
    password_hash: &str,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET password_hash = $2
        WHERE id = $1
        "#,
        user_id,
        password_hash,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, "user password updated");
    Ok(())
}
