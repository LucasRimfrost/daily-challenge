use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ArchiveRow, Challenge, ChallengeHistory, Difficulty, LeaderboardRow, Submission, User,
    UserStats,
};

// ── Users ───────────────────────────────────────────────────────────────────

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

// ── Challenges ──────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_challenge_by_date(
    pool: &PgPool,
    scheduled_date: chrono::NaiveDate,
) -> AppResult<Option<Challenge>> {
    let result = sqlx::query_as!(
        Challenge,
        r#"
        SELECT id, title, description,
               difficulty as "difficulty: Difficulty",
               expected_answer, hint, max_attempts,
               scheduled_date, created_at
        FROM challenges
        WHERE scheduled_date = $1
        "#,
        scheduled_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(
        found = result.is_some(),
        %scheduled_date,
        "challenge lookup by date"
    );
    Ok(result)
}

#[tracing::instrument(skip(pool))]
pub async fn find_challenge_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Challenge>> {
    let result = sqlx::query_as!(
        Challenge,
        r#"
        SELECT id, title, description,
               difficulty as "difficulty: Difficulty",
               expected_answer, hint, max_attempts,
               scheduled_date, created_at
        FROM challenges
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "challenge lookup by id");
    Ok(result)
}

// ── Submissions ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_submissions_by_user_and_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
) -> AppResult<Vec<Submission>> {
    let results = sqlx::query_as!(
        Submission,
        r#"
        SELECT id, user_id, challenge_id, answer,
               is_correct, attempt_number, submitted_at
        FROM submissions
        WHERE user_id = $1 AND challenge_id = $2
        "#,
        user_id,
        challenge_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched submissions");
    Ok(results)
}

#[tracing::instrument(skip(pool, answer))]
pub async fn create_submission(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    attempt_number: i32,
) -> AppResult<Submission> {
    let submission = sqlx::query_as!(
        Submission,
        r#"
        INSERT INTO submissions (user_id, challenge_id, answer,
                                 is_correct, attempt_number)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, user_id, challenge_id, answer, is_correct,
                  attempt_number, submitted_at
        "#,
        user_id,
        challenge_id,
        answer,
        is_correct,
        attempt_number,
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        submission_id = %submission.id,
        %user_id,
        %challenge_id,
        is_correct,
        attempt_number,
        "submission recorded"
    );
    Ok(submission)
}

// ── User history ────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_user_challenge_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<ChallengeHistory>> {
    let results = sqlx::query_as!(
        ChallengeHistory,
        r#"
        SELECT * FROM (
            SELECT DISTINCT ON (s.challenge_id)
                   c.id as challenge_id, c.title,
                   c.difficulty as "difficulty: Difficulty",
                   c.scheduled_date, s.is_correct,
                   s.attempt_number, s.submitted_at
            FROM submissions s
            JOIN challenges c ON c.id = s.challenge_id
            WHERE s.user_id = $1
            ORDER BY s.challenge_id, s.is_correct DESC, s.attempt_number DESC
        ) history
        ORDER BY scheduled_date DESC
        LIMIT $2
        "#,
        user_id,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched challenge history");
    Ok(results)
}

// ── User stats ──────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn upsert_user_stats_on_solve(
    pool: &PgPool,
    user_id: Uuid,
    solved_date: chrono::NaiveDate,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO user_stats (user_id, current_streak, longest_streak, total_solved, last_solved_date)
        VALUES ($1, 1, 1, 1, $2)
        ON CONFLICT (user_id) DO UPDATE SET
            current_streak = CASE
                WHEN user_stats.last_solved_date = $2 THEN user_stats.current_streak
                WHEN user_stats.last_solved_date = $2 - 1 THEN user_stats.current_streak + 1
                ELSE 1
            END,
            longest_streak = GREATEST(
                user_stats.longest_streak,
                CASE
                    WHEN user_stats.last_solved_date = $2 THEN user_stats.current_streak
                    WHEN user_stats.last_solved_date = $2 - 1 THEN user_stats.current_streak + 1
                    ELSE 1
                END
            ),
            total_solved = CASE
                WHEN user_stats.last_solved_date = $2 THEN user_stats.total_solved
                ELSE user_stats.total_solved + 1
            END,
            last_solved_date = $2
        "#,
        user_id,
        solved_date,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, %solved_date, "user stats updated on solve");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn increment_total_attempts(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO user_stats (user_id, total_attempts)
        VALUES ($1, 1)
        ON CONFLICT (user_id) DO UPDATE SET
            total_attempts = user_stats.total_attempts + 1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%user_id, "total attempts incremented");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn find_user_stats(pool: &PgPool, user_id: Uuid) -> AppResult<Option<UserStats>> {
    let result = sqlx::query_as!(
        UserStats,
        r#"
        SELECT user_id, current_streak, longest_streak,
               total_solved, total_attempts, last_solved_date
        FROM user_stats
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "user stats lookup");
    Ok(result)
}

// ── Leaderboard ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_leaderboard(pool: &PgPool, limit: i64) -> AppResult<Vec<LeaderboardRow>> {
    let results = sqlx::query_as!(
        LeaderboardRow,
        r#"
        SELECT u.username, s.current_streak, s.longest_streak, s.total_solved
        FROM user_stats s
        JOIN users u ON u.id = s.user_id
        ORDER BY s.current_streak DESC, s.total_solved DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched leaderboard");
    Ok(results)
}

// ── Archive ─────────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_past_challenges_with_status(
    pool: &PgPool,
    user_id: Uuid,
    today: chrono::NaiveDate,
) -> AppResult<Vec<ArchiveRow>> {
    let results = sqlx::query_as!(
        ArchiveRow,
        r#"
        SELECT c.id, c.title,
               c.difficulty as "difficulty: Difficulty",
               c.scheduled_date, c.max_attempts,
               COALESCE(bool_or(s.is_correct), false) as "is_solved!",
               COUNT(s.id) as "attempts_used!"
        FROM challenges c
        LEFT JOIN submissions s ON s.challenge_id = c.id AND s.user_id = $1
        WHERE c.scheduled_date < $2
        GROUP BY c.id
        ORDER BY c.scheduled_date DESC
        "#,
        user_id,
        today
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched past challenges");
    Ok(results)
}

// ── Refresh tokens ──────────────────────────────────────────────────────

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

#[tracing::instrument(skip(pool, token_hash))]
pub async fn find_refresh_token_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> AppResult<Option<crate::models::RefreshToken>> {
    let result = sqlx::query_as!(
        crate::models::RefreshToken,
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
