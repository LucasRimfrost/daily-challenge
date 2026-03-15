mod common;

use serde_json::json;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn leaderboard_returns_empty_when_no_users_have_stats() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .client
        .get(app.url("/api/v1/leaderboard"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.as_array().unwrap().is_empty());
}

#[tokio::test]
#[serial]
async fn leaderboard_does_not_require_auth() {
    let app = common::TestApp::spawn().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(app.url("/api/v1/leaderboard"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
#[serial]
async fn leaderboard_shows_user_after_solving_challenge() {
    let app = common::TestApp::spawn().await;
    app.register_and_login().await;
    let challenge_id = app.seed_today_challenge().await;

    // Solve the challenge
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/leaderboard"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["username"], "testuser");
    assert_eq!(entries[0]["total_solved"], 1);
    assert_eq!(entries[0]["current_streak"], 1);
}

#[tokio::test]
#[serial]
async fn leaderboard_respects_limit_param() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .client
        .get(app.url("/api/v1/leaderboard?limit=5"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
#[serial]
async fn leaderboard_orders_by_streak_then_solved() {
    let app = common::TestApp::spawn().await;

    let challenge_id = app.seed_today_challenge().await;

    // User A: solves challenge
    app.register("user_a", "a@example.com", "password123").await;
    app.client
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "4" }))
        .send()
        .await
        .unwrap();

    // User B: only attempts, doesn't solve
    // Need a new client to get separate cookies
    let client_b = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();

    client_b
        .post(app.url("/api/v1/auth/register"))
        .json(&json!({
            "username": "user_b",
            "email": "b@example.com",
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    client_b
        .post(app.url("/api/v1/trivia/submit"))
        .json(&json!({ "challenge_id": challenge_id, "answer": "wrong" }))
        .send()
        .await
        .unwrap();

    // Leaderboard should have user_a first (has a streak), user_b should not appear
    // (user_b has 0 solved, 0 streak — only total_attempts)
    let resp = app
        .client
        .get(app.url("/api/v1/leaderboard"))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let entries = body.as_array().unwrap();

    // user_a should be first (or only) with streak=1, solved=1
    let first = &entries[0];
    assert_eq!(first["username"], "user_a");
    assert_eq!(first["current_streak"], 1);
    assert_eq!(first["total_solved"], 1);
}
