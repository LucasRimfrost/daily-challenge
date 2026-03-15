mod common;

use serde_json::json;
use serial_test::serial;

// ── Today's challenge ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn today_returns_challenge_when_one_exists() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let challenge_id = app.seed_today_challenge().await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/today"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], challenge_id.to_string());
    assert_eq!(body["title"], "Test Challenge");
    assert_eq!(body["description"], "What is 2 + 2?");
    assert_eq!(body["difficulty"], "easy");
    assert_eq!(body["max_attempts"], 3);
    assert_eq!(body["attempts_used"], 0);
    assert_eq!(body["is_solved"], false);
    // Answer should be hidden before solving or exhausting attempts
    assert!(body["correct_answer"].is_null());
}

#[tokio::test]
#[serial]
async fn today_returns_404_when_no_challenge_scheduled() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    // No challenge seeded
    let resp = app
        .client
        .get(app.url("/api/v1/trivia/today"))
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
        .get(app.url("/api/v1/trivia/today"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

// ── Submit ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn submit_correct_answer_returns_is_correct() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "4"
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
async fn submit_incorrect_answer_returns_not_correct() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "5"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], false);
    assert_eq!(body["attempt_number"], 1);
    assert_eq!(body["attempts_remaining"], 2);
    // Hint not revealed until attempt 3
    assert!(body["hint"].is_null());
}

#[tokio::test]
#[serial]
async fn submit_is_case_insensitive() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let today = chrono::Utc::now().date_naive();
    let challenge_id = app
        .seed_challenge("Case Test", "Answer is 'Hello'", "easy", "Hello", 3, today)
        .await;

    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "hello"
        }))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_correct"], true);
}

#[tokio::test]
#[serial]
async fn submit_trims_whitespace() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({
            "challenge_id": challenge_id,
            "answer": "  4  "
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
    let challenge_id = app.seed_today_challenge().await;

    // Solve it
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    // Try again
    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
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
    let challenge_id = app.seed_today_challenge().await; // max_attempts = 3

    // Use all 3 attempts with wrong answers
    for _ in 0..3 {
        app.client
            .post(app.url("/api/v1/trivia/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    // 4th attempt should be rejected
    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[tokio::test]
#[serial]
async fn submit_reveals_hint_on_third_wrong_attempt() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let today = chrono::Utc::now().date_naive();
    let challenge_id = app
        .seed_challenge(
            "Hint Test",
            "What is 2+2?",
            "easy",
            "4",
            5, // more than 3 attempts so we can reach attempt 3 wrong
            today,
        )
        .await;

    // Add a hint directly
    sqlx::query("UPDATE trivia_challenges SET hint = 'Think simple math' WHERE id = $1")
        .bind(challenge_id)
        .execute(&app.pool)
        .await
        .unwrap();

    // Wrong attempts 1 and 2 — no hint
    for i in 1..=2 {
        let resp = app
            .client
            .post(app.url("/api/v1/trivia/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
        let body: serde_json::Value = resp.json().await.unwrap();
        assert!(body["hint"].is_null(), "hint should be null on attempt {i}");
    }

    // Wrong attempt 3 — hint revealed
    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["hint"], "Think simple math");
}

#[tokio::test]
#[serial]
async fn submit_for_nonexistent_challenge_returns_404() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let fake_id = uuid::Uuid::new_v4();
    let resp = app
        .client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": fake_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
#[serial]
async fn submit_requires_auth() {
    let app = common::TestApp::spawn().await;
    let challenge_id = app.seed_today_challenge().await;

    let client = reqwest::Client::new();
    let resp = client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
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
    let challenge_id = app.seed_today_challenge().await;

    // Solve it
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    // Fetch today — answer should now be visible
    let resp = app
        .client
        .get(app.url("/api/v1/trivia/today"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], true);
    assert_eq!(body["correct_answer"], "4");
    assert_eq!(body["attempts_used"], 1);
}

#[tokio::test]
#[serial]
async fn today_reveals_answer_after_exhausting_attempts() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await; // max_attempts = 3

    for _ in 0..3 {
        app.client
            .post(app.url("/api/v1/trivia/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/today"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["is_solved"], false);
    assert_eq!(body["correct_answer"], "4");
    assert_eq!(body["attempts_used"], 3);
}

// ── Stats integration ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn solving_today_updates_user_stats() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    // Solve it
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    // Check stats via /me
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let stats = &body["stats"];
    assert_eq!(stats["total_solved"], 1);
    assert_eq!(stats["current_streak"], 1);
    assert_eq!(stats["longest_streak"], 1);
    // 1 correct attempt
    assert_eq!(stats["total_attempts"], 1);
}

#[tokio::test]
#[serial]
async fn wrong_attempts_increment_total_attempts() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    // Two wrong attempts
    for _ in 0..2 {
        app.client
            .post(app.url("/api/v1/trivia/submit"))
            .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
            .send()
            .await
            .unwrap();
    }

    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stats"]["total_attempts"], 2);
    assert_eq!(body["stats"]["total_solved"], 0);
}

// ── Challenge by date ───────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn by_date_returns_challenge_for_past_date() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
    app.seed_challenge_for_date(date).await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/2025-01-15"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["title"], "Past Challenge");
}

#[tokio::test]
#[serial]
async fn by_date_returns_404_for_date_without_challenge() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/2099-12-31"))
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
        .get(app.url("/api/v1/trivia/history"))
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
    let challenge_id = app.seed_today_challenge().await;

    // Submit an answer
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/history"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let history = body.as_array().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["title"], "Test Challenge");
    assert_eq!(history[0]["is_correct"], true);
}

#[tokio::test]
#[serial]
async fn history_respects_limit_param() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/history?limit=5"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

// ── Archive ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn archive_returns_past_challenges() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);
    app.seed_challenge_for_date(yesterday).await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/archive"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["is_solved"], false);
    assert_eq!(entries[0]["attempts_used"], 0);
}

#[tokio::test]
#[serial]
async fn archive_does_not_include_today() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;

    // Seed only today's challenge
    app.seed_today_challenge().await;

    let resp = app
        .client
        .get(app.url("/api/v1/trivia/archive"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}
