use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Difficulty, LeaderboardRow, PasswordResetToken, RefreshToken, TriviaArchiveRow,
    TriviaChallenge, TriviaChallengeHistory, TriviaStats, TriviaSubmission, User,
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

// ── Games registry ──────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_active_games(pool: &PgPool) -> AppResult<Vec<crate::models::Game>> {
    let results = sqlx::query_as!(
        crate::models::Game,
        r#"
        SELECT id, name, description, icon, is_active, sort_order, created_at
        FROM games
        WHERE is_active = true
        ORDER BY sort_order ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched active games");
    Ok(results)
}

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_challenge_by_date(
    pool: &PgPool,
    scheduled_date: chrono::NaiveDate,
) -> AppResult<Option<TriviaChallenge>> {
    let result = sqlx::query_as!(
        TriviaChallenge,
        r#"
        SELECT id, title, description,
               difficulty as "difficulty: Difficulty",
               expected_answer, hint, max_attempts,
               scheduled_date, created_at
        FROM trivia_challenges
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
        "trivia challenge lookup by date"
    );
    Ok(result)
}

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_challenge_by_id(
    pool: &PgPool,
    id: Uuid,
) -> AppResult<Option<TriviaChallenge>> {
    let result = sqlx::query_as!(
        TriviaChallenge,
        r#"
        SELECT id, title, description,
               difficulty as "difficulty: Difficulty",
               expected_answer, hint, max_attempts,
               scheduled_date, created_at
        FROM trivia_challenges
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "trivia challenge lookup by id");
    Ok(result)
}

// ── Submissions ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_submissions_by_user_and_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
) -> AppResult<Vec<TriviaSubmission>> {
    let results = sqlx::query_as!(
        TriviaSubmission,
        r#"
        SELECT id, user_id, challenge_id, answer,
               is_correct, attempt_number, submitted_at
        FROM trivia_submissions
        WHERE user_id = $1 AND challenge_id = $2
        "#,
        user_id,
        challenge_id
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched trivia submissions");
    Ok(results)
}

#[tracing::instrument(skip(pool, answer))]
pub async fn create_trivia_submission(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    attempt_number: i32,
) -> AppResult<TriviaSubmission> {
    let submission = sqlx::query_as!(
        TriviaSubmission,
        r#"
        INSERT INTO trivia_submissions (user_id, challenge_id, answer,
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
        "trivia submission recorded"
    );
    Ok(submission)
}

// ── User history ────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_challenge_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<TriviaChallengeHistory>> {
    let results = sqlx::query_as!(
        TriviaChallengeHistory,
        r#"
        SELECT * FROM (
            SELECT DISTINCT ON (s.challenge_id)
                   c.id as challenge_id, c.title,
                   c.difficulty as "difficulty: Difficulty",
                   c.scheduled_date, s.is_correct,
                   s.attempt_number, s.submitted_at
            FROM trivia_submissions s
            JOIN trivia_challenges c ON c.id = s.challenge_id
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

    tracing::debug!(count = results.len(), "fetched trivia challenge history");
    Ok(results)
}

// ── User stats ──────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn upsert_trivia_stats_on_solve(
    pool: &PgPool,
    user_id: Uuid,
    solved_date: chrono::NaiveDate,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO trivia_stats (user_id, current_streak, longest_streak, total_solved, last_solved_date)
        VALUES ($1, 1, 1, 1, $2)
        ON CONFLICT (user_id) DO UPDATE SET
            current_streak = CASE
                WHEN trivia_stats.last_solved_date = $2 THEN trivia_stats.current_streak
                WHEN trivia_stats.last_solved_date = $2 - 1 THEN trivia_stats.current_streak + 1
                ELSE 1
            END,
            longest_streak = GREATEST(
                trivia_stats.longest_streak,
                CASE
                    WHEN trivia_stats.last_solved_date = $2 THEN trivia_stats.current_streak
                    WHEN trivia_stats.last_solved_date = $2 - 1 THEN trivia_stats.current_streak + 1
                    ELSE 1
                END
            ),
            total_solved = CASE
                WHEN trivia_stats.last_solved_date = $2 THEN trivia_stats.total_solved
                ELSE trivia_stats.total_solved + 1
            END,
            last_solved_date = $2
        "#,
        user_id,
        solved_date,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, %solved_date, "trivia stats updated on solve");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn increment_trivia_attempts(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO trivia_stats (user_id, total_attempts)
        VALUES ($1, 1)
        ON CONFLICT (user_id) DO UPDATE SET
            total_attempts = trivia_stats.total_attempts + 1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%user_id, "trivia total attempts incremented");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_stats(pool: &PgPool, user_id: Uuid) -> AppResult<Option<TriviaStats>> {
    let result = sqlx::query_as!(
        TriviaStats,
        r#"
        SELECT user_id, current_streak, longest_streak,
               total_solved, total_attempts, last_solved_date
        FROM trivia_stats
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "trivia stats lookup");
    Ok(result)
}

// ── Leaderboard ─────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_leaderboard(pool: &PgPool, limit: i64) -> AppResult<Vec<LeaderboardRow>> {
    let results = sqlx::query_as!(
        LeaderboardRow,
        r#"
        SELECT u.username, s.current_streak, s.longest_streak, s.total_solved
        FROM trivia_stats s
        JOIN users u ON u.id = s.user_id
        ORDER BY s.current_streak DESC, s.total_solved DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched trivia leaderboard");
    Ok(results)
}

// ── Archive ─────────────────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_trivia_past_challenges(
    pool: &PgPool,
    user_id: Uuid,
    today: chrono::NaiveDate,
) -> AppResult<Vec<TriviaArchiveRow>> {
    let results = sqlx::query_as!(
        TriviaArchiveRow,
        r#"
        SELECT c.id, c.title,
               c.difficulty as "difficulty: Difficulty",
               c.scheduled_date, c.max_attempts,
               COALESCE(bool_or(s.is_correct), false) as "is_solved!",
               COUNT(s.id) as "attempts_used!"
        FROM trivia_challenges c
        LEFT JOIN trivia_submissions s ON s.challenge_id = c.id AND s.user_id = $1
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

    tracing::debug!(count = results.len(), "fetched trivia past challenges");
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

// ── Password reset tokens ───────────────────────────────────────────────

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

// ── Code Output challenges ──────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_by_date(
    pool: &PgPool,
    scheduled_date: chrono::NaiveDate,
) -> AppResult<Option<crate::models::CodeOutputChallenge>> {
    let result = sqlx::query_as!(
        crate::models::CodeOutputChallenge,
        r#"
        SELECT id, title, description, language, code_snippet,
               expected_output, difficulty as "difficulty: Difficulty",
               hint, max_attempts, scheduled_date, created_at
        FROM code_output_challenges
        WHERE scheduled_date = $1
        "#,
        scheduled_date,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), %scheduled_date, "code output challenge lookup by date");
    Ok(result)
}

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_by_id(
    pool: &PgPool,
    id: Uuid,
) -> AppResult<Option<crate::models::CodeOutputChallenge>> {
    let result = sqlx::query_as!(
        crate::models::CodeOutputChallenge,
        r#"
        SELECT id, title, description, language, code_snippet,
               expected_output, difficulty as "difficulty: Difficulty",
               hint, max_attempts, scheduled_date, created_at
        FROM code_output_challenges
        WHERE id = $1
        "#,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(
        found = result.is_some(),
        "code output challenge lookup by id"
    );
    Ok(result)
}

// ── Code Output submissions ─────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_submissions_by_user_and_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
) -> AppResult<Vec<crate::models::CodeOutputSubmission>> {
    let results = sqlx::query_as!(
        crate::models::CodeOutputSubmission,
        r#"
        SELECT id, user_id, challenge_id, answer,
               is_correct, attempt_number, submitted_at
        FROM code_output_submissions
        WHERE user_id = $1 AND challenge_id = $2
        "#,
        user_id,
        challenge_id,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched code output submissions");
    Ok(results)
}

#[tracing::instrument(skip(pool, answer))]
pub async fn create_code_output_submission(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    attempt_number: i32,
) -> AppResult<crate::models::CodeOutputSubmission> {
    let submission = sqlx::query_as!(
        crate::models::CodeOutputSubmission,
        r#"
        INSERT INTO code_output_submissions (user_id, challenge_id, answer,
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
        "code output submission recorded"
    );
    Ok(submission)
}

// ── Code Output history ─────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<crate::models::CodeOutputChallengeHistory>> {
    let results = sqlx::query_as!(
        crate::models::CodeOutputChallengeHistory,
        r#"
        SELECT * FROM (
            SELECT DISTINCT ON (s.challenge_id)
                   c.id as challenge_id, c.title, c.language,
                   c.difficulty as "difficulty: Difficulty",
                   c.scheduled_date, s.is_correct,
                   s.attempt_number, s.submitted_at
            FROM code_output_submissions s
            JOIN code_output_challenges c ON c.id = s.challenge_id
            WHERE s.user_id = $1
            ORDER BY s.challenge_id, s.is_correct DESC, s.attempt_number DESC
        ) history
        ORDER BY scheduled_date DESC
        LIMIT $2
        "#,
        user_id,
        limit,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched code output history");
    Ok(results)
}

// ── Code Output stats ───────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn upsert_code_output_stats_on_solve(
    pool: &PgPool,
    user_id: Uuid,
    solved_date: chrono::NaiveDate,
) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO code_output_stats (user_id, current_streak, longest_streak, total_solved, last_solved_date)
        VALUES ($1, 1, 1, 1, $2)
        ON CONFLICT (user_id) DO UPDATE SET
            current_streak = CASE
                WHEN code_output_stats.last_solved_date = $2 THEN code_output_stats.current_streak
                WHEN code_output_stats.last_solved_date = $2 - 1 THEN code_output_stats.current_streak + 1
                ELSE 1
            END,
            longest_streak = GREATEST(
                code_output_stats.longest_streak,
                CASE
                    WHEN code_output_stats.last_solved_date = $2 THEN code_output_stats.current_streak
                    WHEN code_output_stats.last_solved_date = $2 - 1 THEN code_output_stats.current_streak + 1
                    ELSE 1
                END
            ),
            total_solved = CASE
                WHEN code_output_stats.last_solved_date = $2 THEN code_output_stats.total_solved
                ELSE code_output_stats.total_solved + 1
            END,
            last_solved_date = $2
        "#,
        user_id,
        solved_date,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::info!(%user_id, %solved_date, "code output stats updated on solve");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn increment_code_output_attempts(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        INSERT INTO code_output_stats (user_id, total_attempts)
        VALUES ($1, 1)
        ON CONFLICT (user_id) DO UPDATE SET
            total_attempts = code_output_stats.total_attempts + 1
        "#,
        user_id,
    )
    .execute(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(%user_id, "code output attempts incremented");
    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_stats(
    pool: &PgPool,
    user_id: Uuid,
) -> AppResult<Option<crate::models::CodeOutputStats>> {
    let result = sqlx::query_as!(
        crate::models::CodeOutputStats,
        r#"
        SELECT user_id, current_streak, longest_streak,
               total_solved, total_attempts, last_solved_date
        FROM code_output_stats
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(found = result.is_some(), "code output stats lookup");
    Ok(result)
}

// ── Code Output leaderboard ─────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_leaderboard(
    pool: &PgPool,
    limit: i64,
) -> AppResult<Vec<crate::models::LeaderboardRow>> {
    let results = sqlx::query_as!(
        crate::models::LeaderboardRow,
        r#"
        SELECT u.username, s.current_streak, s.longest_streak, s.total_solved
        FROM code_output_stats s
        JOIN users u ON u.id = s.user_id
        ORDER BY s.current_streak DESC, s.total_solved DESC
        LIMIT $1
        "#,
        limit,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched code output leaderboard");
    Ok(results)
}

// ── Code Output archive ─────────────────────────────────────────────────────

#[tracing::instrument(skip(pool))]
pub async fn find_code_output_past_challenges(
    pool: &PgPool,
    user_id: Uuid,
    today: chrono::NaiveDate,
) -> AppResult<Vec<crate::models::CodeOutputArchiveRow>> {
    let results = sqlx::query_as!(
        crate::models::CodeOutputArchiveRow,
        r#"
        SELECT c.id, c.title, c.language,
               c.difficulty as "difficulty: Difficulty",
               c.scheduled_date, c.max_attempts,
               COALESCE(bool_or(s.is_correct), false) as "is_solved!",
               COUNT(s.id) as "attempts_used!"
        FROM code_output_challenges c
        LEFT JOIN code_output_submissions s ON s.challenge_id = c.id AND s.user_id = $1
        WHERE c.scheduled_date < $2
        GROUP BY c.id
        ORDER BY c.scheduled_date DESC
        "#,
        user_id,
        today,
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)?;

    tracing::debug!(count = results.len(), "fetched code output past challenges");
    Ok(results)
}
