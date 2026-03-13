use shared::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::User;

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> Result<User, AppError> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, AppError> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}
