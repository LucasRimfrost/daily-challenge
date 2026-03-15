#![allow(dead_code)]
mod common;

use serde_json::json;
use serial_test::serial;

// ── Today's challenge ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn today_returns_challenge_when_one_exists() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let challenge_id = app.seed_today_code_output_challenge().await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], challenge_id.to_string());
    assert_eq!(body["title"], "Print Test");
    assert_eq!(body["language"], "python");
    assert!(body["code_snippet"].as_str().unwrap().contains("print"));
    assert_eq!(body["max_attempts"], 3);
    assert_eq!(body["attempts_used"], 0);
    assert_eq!(body["is_solved"], false);
    // Answer hidden before solving
    assert!(body["correct_answer"].is_null());
}

#[tokio::test]
#[serial]
async fn today_returns_404_when_no_challenge_scheduled() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
#[serial]
async fn today_requires_auth() {
    let app = common::TestApp::spawn().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

// ── Submit ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn submit_correct_answer() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "[2, 3]"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], true);
    assert_eq!(body["attempt_number"], 1);
    assert_eq!(body["attempts_remaining"], 2);
}

#[tokio::test]
#[serial]
async fn submit_incorrect_answer() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "[1, 2, 3]"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], false);
    assert_eq!(body["attempt_number"], 1);
    // Hint not revealed until attempt 2 for code output
    assert!(body["hint"].is_null());
}

#[tokio::test]
#[serial]
async fn submit_is_case_sensitive() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let today = chrono::Utc::now().date_naive();
    let challenge_id = app
        .seed_code_output_challenge(
            "Case Test",
            "What does this print?",
            "python",
            "print('Hello World')",
            "Hello World",
            3,
            today,
        )
        .await;

    // Lowercase should be wrong — output is case-sensitive
    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "hello world"
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], false);
}

#[tokio::test]
#[serial]
async fn submit_trims_whitespace() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "  [2, 3]  "
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], true);
}

#[tokio::test]
#[serial]
async fn submit_rejects_after_already_solved() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    // Solve it
    app.client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    // Try again
    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
#[serial]
async fn submit_rejects_after_max_attempts_exhausted() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    for _ in 0..3 {
        app.client
            .post(app.url("/api/v1/code-output/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
#[serial]
async fn submit_reveals_hint_on_second_wrong_attempt() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let today = chrono::Utc::now().date_naive();
    let challenge_id = app
        .seed_code_output_challenge(
            "Hint Test",
            "What does this print?",
            "python",
            "print(type([]))",
            "<class 'list'>",
            5,
            today,
        )
        .await;

    // Add hint
    sqlx::query(
        "UPDATE code_output_challenges SET hint = 'Think about the type function' WHERE id = $1",
    )
    .bind(challenge_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Attempt 1 — no hint
    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "list" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["hint"].is_null());

    // Attempt 2 — hint revealed
    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "list" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["hint"], "Think about the type function");
}

#[tokio::test]
#[serial]
async fn submit_for_nonexistent_challenge_returns_404() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let fake_id = uuid::Uuid::new_v4();
    let resp = app
        .client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": fake_id, "answer": "test" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
#[serial]
async fn submit_requires_auth() {
    let app = common::TestApp::spawn().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    let client = reqwest::Client::new();
    let resp = client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

// ── Today after solving ─────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn today_reveals_answer_after_solving() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    app.client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], true);
    assert_eq!(body["correct_answer"], "[2, 3]");
    assert_eq!(body["attempts_used"], 1);
}

#[tokio::test]
#[serial]
async fn today_reveals_answer_after_exhausting_attempts() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    for _ in 0..3 {
        app.client
            .post(app.url("/api/v1/code-output/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], false);
    assert_eq!(body["correct_answer"], "[2, 3]");
    assert_eq!(body["attempts_used"], 3);
}

// ── Stats ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn solving_updates_code_output_stats_not_trivia() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    app.client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    // Trivia stats should still be zero (different game)
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stats"]["total_solved"], 0);
    assert_eq!(body["stats"]["total_attempts"], 0);
}

// ── By date ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn by_date_returns_challenge() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let date = chrono::NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
    app.seed_code_output_challenge(
        "Past Code",
        "Old challenge",
        "javascript",
        "console.log(typeof null)",
        "object",
        3,
        date,
    )
    .await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/2025-06-15"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Past Code");
    assert_eq!(body["language"], "javascript");
}

#[tokio::test]
#[serial]
async fn by_date_returns_404_for_missing_date() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/2099-12-31"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

// ── History ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn history_returns_empty_for_new_user() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/history"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn history_includes_submitted_challenges() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_code_output_challenge().await;

    app.client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/history"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let history = body.as_array().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["title"], "Print Test");
    assert_eq!(history[0]["language"], "python");
    assert_eq!(history[0]["is_correct"], true);
}

// ── Archive ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn archive_returns_past_challenges() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
    app.seed_code_output_challenge(
        "Yesterday's Code",
        "Old puzzle",
        "python",
        "print(1 + 1)",
        "2",
        3,
        yesterday,
    )
    .await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/archive"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["is_solved"], false);
    assert_eq!(entries[0]["language"], "python");
}

#[tokio::test]
#[serial]
async fn archive_does_not_include_today() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    app.seed_today_code_output_challenge().await;

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/archive"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

// ── Games isolation ─────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn trivia_and_code_output_are_independent() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    // Seed both games for today
    let trivia_id = app.seed_today_challenge().await;
    // Need a different date since both have UNIQUE on scheduled_date
    // Actually they're in different tables, so same date is fine
    let code_id = app.seed_today_code_output_challenge().await;

    // Solve trivia
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": trivia_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    // Code output should still be unsolved
    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], false);
    assert_eq!(body["attempts_used"], 0);

    // Solve code output
    app.client
        .post(app.url("/api/v1/code-output/submit"))
        .json(&json!({ "challenge_id": code_id, "answer": "[2, 3]" }))
        .send()
        .await
        .unwrap();

    // Both should now be solved independently
    let resp = app
        .client
        .get(app.url("/api/v1/trivia/today"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], true);

    let resp = app
        .client
        .get(app.url("/api/v1/code-output/today"))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], true);
}
