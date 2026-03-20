//! Domain models mapped to database tables via [`sqlx::FromRow`].

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A registered user account.
#[derive(FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password_hash", &"[REDACTED]")
            .field("created_at", &self.created_at)
            .finish()
    }
}

/// A user's public profile fields (no password_hash).
#[derive(Debug, FromRow, Serialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

/// Challenge difficulty level, stored as lowercase text in PostgreSQL.
#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

// ── Games registry ──────────────────────────────────────────────────────────

/// An entry in the games registry (e.g. "trivia", "code-output").
#[derive(Debug, FromRow, Serialize)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
}

// ── Trivia game ─────────────────────────────────────────────────────────────

/// A trivia question scheduled for a specific date.
#[derive(Debug, FromRow, Serialize)]
pub struct TriviaChallenge {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub expected_answer: String,
    pub hint: Option<String>,
    pub max_attempts: i32,
    pub scheduled_date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
}

/// A single answer attempt by a user for a trivia challenge.
#[derive(Debug, FromRow, Serialize)]
pub struct TriviaSubmission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub challenge_id: Uuid,
    pub answer: String,
    pub is_correct: bool,
    pub attempt_number: i32,
    pub submitted_at: DateTime<Utc>,
}

/// Aggregate trivia statistics for a user (streaks, totals).
#[derive(Debug, FromRow, Serialize)]
pub struct TriviaStats {
    pub user_id: Uuid,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub total_solved: i32,
    pub total_attempts: i32,
    pub last_solved_date: Option<chrono::NaiveDate>,
}

/// Denormalized view of a user's best attempt per trivia challenge, used for history display.
#[derive(Debug, FromRow, Serialize)]
pub struct TriviaChallengeHistory {
    pub challenge_id: Uuid,
    pub title: String,
    pub difficulty: Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub is_correct: bool,
    pub attempt_number: i32,
    pub submitted_at: DateTime<Utc>,
}

/// Row returned by the trivia archive query, combining challenge info with user progress.
#[derive(Debug, FromRow)]
pub struct TriviaArchiveRow {
    pub id: Uuid,
    pub title: String,
    pub difficulty: Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub max_attempts: i32,
    pub is_solved: bool,
    pub attempts_used: i64,
}

/// A single row on the leaderboard, shared by both game types.
#[derive(Debug, FromRow, Serialize)]
pub struct LeaderboardRow {
    pub username: String,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub total_solved: i32,
}

/// A hashed refresh token stored in the database for token rotation.
#[derive(Debug, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// A hashed password-reset token with a one-hour TTL.
#[derive(Debug, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

// ── Code Output game ────────────────────────────────────────────────────────

/// A "predict the output" challenge showing a code snippet in a given language.
#[derive(Debug, FromRow, Serialize)]
pub struct CodeOutputChallenge {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub language: String,
    pub code_snippet: String,
    pub expected_output: String,
    pub difficulty: Difficulty,
    pub hint: Option<String>,
    pub max_attempts: i32,
    pub scheduled_date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
}

/// A single answer attempt by a user for a code-output challenge.
#[derive(Debug, FromRow, Serialize)]
pub struct CodeOutputSubmission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub challenge_id: Uuid,
    pub answer: String,
    pub is_correct: bool,
    pub attempt_number: i32,
    pub submitted_at: DateTime<Utc>,
}

/// Aggregate code-output statistics for a user (streaks, totals).
#[derive(Debug, FromRow, Serialize)]
pub struct CodeOutputStats {
    pub user_id: Uuid,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub total_solved: i32,
    pub total_attempts: i32,
    pub last_solved_date: Option<chrono::NaiveDate>,
}

/// Denormalized view of a user's best attempt per code-output challenge, used for history display.
#[derive(Debug, FromRow, Serialize)]
pub struct CodeOutputChallengeHistory {
    pub challenge_id: Uuid,
    pub title: String,
    pub language: String,
    pub difficulty: Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub is_correct: bool,
    pub attempt_number: i32,
    pub submitted_at: DateTime<Utc>,
}

/// Row returned by the code-output archive query, combining challenge info with user progress.
#[derive(Debug, FromRow)]
pub struct CodeOutputArchiveRow {
    pub id: Uuid,
    pub title: String,
    pub language: String,
    pub difficulty: Difficulty,
    pub scheduled_date: chrono::NaiveDate,
    pub max_attempts: i32,
    pub is_solved: bool,
    pub attempts_used: i64,
}
