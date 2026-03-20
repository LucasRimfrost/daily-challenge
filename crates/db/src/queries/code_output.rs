use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    CodeOutputArchiveRow, CodeOutputChallenge, CodeOutputChallengeHistory, CodeOutputStats,
    CodeOutputSubmission, Difficulty, LeaderboardRow,
};

// ── Code Output challenges ──────────────────────────────────────────────────

/// Finds the code-output challenge scheduled for a given date.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_by_date(
    pool: &PgPool,
    scheduled_date: chrono::NaiveDate,
) -> AppResult<Option<CodeOutputChallenge>> {
    let result = sqlx::query_as!(
        CodeOutputChallenge,
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

/// Finds a code-output challenge by its primary key.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_by_id(
    pool: &PgPool,
    id: Uuid,
) -> AppResult<Option<CodeOutputChallenge>> {
    let result = sqlx::query_as!(
        CodeOutputChallenge,
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

/// Returns all code-output submissions by a user for a specific challenge.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_submissions_by_user_and_challenge(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
) -> AppResult<Vec<CodeOutputSubmission>> {
    let results = sqlx::query_as!(
        CodeOutputSubmission,
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

/// Records a new code-output submission and returns the inserted row.
#[tracing::instrument(skip(pool, answer))]
pub async fn create_code_output_submission(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    attempt_number: i32,
) -> AppResult<CodeOutputSubmission> {
    let submission = sqlx::query_as!(
        CodeOutputSubmission,
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

/// Atomically validates attempt limits and records a code-output submission.
///
/// Uses a transaction with `SELECT … FOR UPDATE` on the challenge row to
/// serialise concurrent requests for the same (user, challenge) pair.
/// Also increments the user's attempt counter and, when `solved_date` is
/// provided, upserts their streak / solve stats.
///
/// Returns the inserted submission.
///
/// # Errors
///
/// Returns [`AppError::BadRequest`] if the challenge is already solved or
/// the maximum number of attempts has been reached.
#[tracing::instrument(skip(pool, answer))]
pub async fn create_code_output_submission_atomic(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    max_attempts: i32,
    solved_date: Option<chrono::NaiveDate>,
) -> AppResult<CodeOutputSubmission> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    // Lock the challenge row to serialise concurrent submissions.
    let locked = sqlx::query_scalar!(
        "SELECT id FROM code_output_challenges WHERE id = $1 FOR UPDATE",
        challenge_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::from)?;

    if locked.is_none() {
        return Err(AppError::NotFound);
    }

    // Count existing submissions *inside* the transaction.
    let existing: Vec<CodeOutputSubmission> = sqlx::query_as!(
        CodeOutputSubmission,
        r#"
        SELECT id, user_id, challenge_id, answer,
               is_correct, attempt_number, submitted_at
        FROM code_output_submissions
        WHERE user_id = $1 AND challenge_id = $2
        "#,
        user_id,
        challenge_id,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(AppError::from)?;

    if existing.iter().any(|s| s.is_correct) {
        return Err(AppError::BadRequest("Challenge already solved".into()));
    }

    if existing.len() as i32 >= max_attempts {
        return Err(AppError::BadRequest("No attempts remaining".into()));
    }

    let attempt_number = existing.len() as i32 + 1;

    // Insert the submission.
    let submission = sqlx::query_as!(
        CodeOutputSubmission,
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
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // Increment lifetime attempt counter.
    sqlx::query!(
        r#"
        INSERT INTO code_output_stats (user_id, total_attempts)
        VALUES ($1, 1)
        ON CONFLICT (user_id) DO UPDATE SET
            total_attempts = code_output_stats.total_attempts + 1
        "#,
        user_id,
    )
    .execute(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // Update streak / solve stats when the answer is correct.
    if let Some(date) = solved_date {
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
            date,
        )
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;
    }

    tx.commit().await.map_err(AppError::from)?;

    tracing::info!(
        submission_id = %submission.id,
        %user_id,
        %challenge_id,
        is_correct,
        attempt_number,
        "code output submission recorded (atomic)"
    );
    Ok(submission)
}

// ── Code Output history ─────────────────────────────────────────────────────

/// Returns a user's best attempt per code-output challenge, most recent first.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_challenge_history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<CodeOutputChallengeHistory>> {
    let results = sqlx::query_as!(
        CodeOutputChallengeHistory,
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

/// Upserts the user's code-output stats after a correct solve.
///
/// Streak logic mirrors [`super::trivia::upsert_trivia_stats_on_solve`].
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

/// Increments the user's lifetime code-output attempt counter.
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

/// Returns the code-output stats row for a user, if one exists.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_stats(
    pool: &PgPool,
    user_id: Uuid,
) -> AppResult<Option<CodeOutputStats>> {
    let result = sqlx::query_as!(
        CodeOutputStats,
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

/// Returns the top code-output players ranked by current streak, then total solved.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_leaderboard(
    pool: &PgPool,
    limit: i64,
) -> AppResult<Vec<LeaderboardRow>> {
    let results = sqlx::query_as!(
        LeaderboardRow,
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

/// Returns all code-output challenges scheduled before `today`, annotated with
/// the user's solve status and attempt count.
#[tracing::instrument(skip(pool))]
pub async fn find_code_output_past_challenges(
    pool: &PgPool,
    user_id: Uuid,
    today: chrono::NaiveDate,
) -> AppResult<Vec<CodeOutputArchiveRow>> {
    let results = sqlx::query_as!(
        CodeOutputArchiveRow,
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
        LIMIT 365
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
