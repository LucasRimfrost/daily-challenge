use shared::error::{AppError, AppResult};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Difficulty, LeaderboardRow, TriviaArchiveRow, TriviaChallenge, TriviaChallengeHistory,
    TriviaStats, TriviaSubmission,
};

// ── Trivia challenges ───────────────────────────────────────────────────────

/// Finds the trivia challenge scheduled for a given date.
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

/// Finds a trivia challenge by its primary key.
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

// ── Trivia submissions ──────────────────────────────────────────────────────

/// Returns all trivia submissions by a user for a specific challenge.
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

/// Records a new trivia submission and returns the inserted row.
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

/// Atomically validates attempt limits and records a trivia submission.
///
/// Uses a transaction with `SELECT … FOR UPDATE` on the challenge row to
/// serialise concurrent requests for the same (user, challenge) pair.
/// Also increments the user's attempt counter and, when `solved_date` is
/// provided, upserts their streak / solve stats.
///
/// Returns the inserted submission together with the total attempt count.
///
/// # Errors
///
/// Returns [`AppError::BadRequest`] if the challenge is already solved or
/// the maximum number of attempts has been reached.
#[tracing::instrument(skip(pool, answer))]
pub async fn create_trivia_submission_atomic(
    pool: &PgPool,
    user_id: Uuid,
    challenge_id: Uuid,
    answer: &str,
    is_correct: bool,
    max_attempts: i32,
    solved_date: Option<chrono::NaiveDate>,
) -> AppResult<TriviaSubmission> {
    let mut tx = pool.begin().await.map_err(AppError::from)?;

    // Lock the challenge row to serialise concurrent submissions.
    let locked = sqlx::query_scalar!(
        "SELECT id FROM trivia_challenges WHERE id = $1 FOR UPDATE",
        challenge_id
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::from)?;

    if locked.is_none() {
        return Err(AppError::NotFound);
    }

    // Count existing submissions *inside* the transaction.
    let existing: Vec<TriviaSubmission> = sqlx::query_as!(
        TriviaSubmission,
        r#"
        SELECT id, user_id, challenge_id, answer,
               is_correct, attempt_number, submitted_at
        FROM trivia_submissions
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
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // Increment lifetime attempt counter.
    sqlx::query!(
        r#"
        INSERT INTO trivia_stats (user_id, total_attempts)
        VALUES ($1, 1)
        ON CONFLICT (user_id) DO UPDATE SET
            total_attempts = trivia_stats.total_attempts + 1
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
        "trivia submission recorded (atomic)"
    );
    Ok(submission)
}

// ── Trivia history ──────────────────────────────────────────────────────────

/// Returns a user's best attempt per trivia challenge, most recent first.
///
/// Uses `DISTINCT ON` to pick the best submission for each challenge
/// (correct answers first, then highest attempt number).
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

// ── Trivia stats ────────────────────────────────────────────────────────────

/// Upserts the user's trivia stats after a correct solve.
///
/// Streak logic: if the user solved yesterday's challenge, the streak is
/// extended; if they solved today's already, stats are unchanged; otherwise
/// the streak resets to 1.
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

/// Increments the user's lifetime trivia attempt counter (upserts if no row exists).
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

/// Returns the trivia stats row for a user, if one exists.
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

// ── Trivia leaderboard ──────────────────────────────────────────────────────

/// Returns the top trivia players ranked by current streak, then total solved.
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

// ── Trivia archive ──────────────────────────────────────────────────────────

/// Returns all trivia challenges scheduled before `today`, annotated with the
/// user's solve status and attempt count.
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
        LIMIT 365
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
