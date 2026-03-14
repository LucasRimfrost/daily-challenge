use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    ArchiveRow, Challenge, ChallengeHistory, Difficulty, LeaderboardRow, Submission, User,
    UserStats,
};

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password_hash: &str,
) -> AppResult<User> {
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

pub async fn find_user_by_email(pool: &PgPool, email: &str) -> AppResult<Option<User>> {
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

pub async fn find_user_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<User>> {
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

pub async fn find_challenge_by_date(
    pool: &PgPool,
    scheduled_date: chrono::NaiveDate,
) -> AppResult<Option<Challenge>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_challenge_by_id(pool: &PgPool, id: Uuid) -> AppResult<Option<Challenge>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_submissions_by_user_and_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
) -> AppResult<Vec<Submission>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn create_submission(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    attempt_number: i32,
) -> AppResult<Submission> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_user_challenge_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<ChallengeHistory>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

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

    Ok(())
}

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

    Ok(())
}

pub async fn find_leaderboard(pool: &PgPool, limit: i64) -> AppResult<Vec<LeaderboardRow>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_user_stats(pool: &PgPool, user_id: Uuid) -> AppResult<Option<UserStats>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}

pub async fn find_past_challenges_with_status(
    pool: &PgPool,
    user_id: Uuid,
    today: chrono::NaiveDate,
) -> AppResult<Vec<ArchiveRow>> {
    sqlx::query_as!(
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
    .map_err(AppError::from)
}
