mod common;

use serde_json::json;
use serial_test::serial;

// ── Registration ────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn register_returns_201_and_user_data() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .register("newuser", "new@example.com", "password123")
        .await;

    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "newuser");
    assert_eq!(body["email"], "new@example.com");
    assert!(body["id"].is_string());
}

#[tokio::test]
#[serial]
async fn register_sets_access_token_cookie() {
    let app = common::TestApp::spawn().await;

    app.register("cookieuser", "cookie@example.com", "password123")
        .await;

    // After register, /me should work (cookie was set and stored by the client)
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
#[serial]
async fn register_rejects_short_password() {
    let app = common::TestApp::spawn().await;

    let resp = app.register("user", "short@example.com", "short").await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
#[serial]
async fn register_rejects_short_username() {
    let app = common::TestApp::spawn().await;

    let resp = app.register("ab", "short@example.com", "password123").await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
#[serial]
async fn register_rejects_invalid_email() {
    let app = common::TestApp::spawn().await;

    let resp = app.register("user", "not-an-email", "password123").await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
#[serial]
async fn register_rejects_duplicate_email() {
    let app = common::TestApp::spawn().await;

    app.register("first", "dupe@example.com", "password123")
        .await;

    let resp = app
        .register("second", "dupe@example.com", "password123")
        .await;
    assert_eq!(resp.status(), 409);
}

// ── Login ───────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn login_with_valid_credentials_returns_200() {
    let app = common::TestApp::spawn().await;

    app.register("loginuser", "login@example.com", "password123")
        .await;

    let resp = app.login("login@example.com", "password123").await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["email"], "login@example.com");
    assert_eq!(body["username"], "loginuser");
}

#[tokio::test]
#[serial]
async fn login_with_wrong_password_returns_401() {
    let app = common::TestApp::spawn().await;

    app.register("wrongpw", "wrongpw@example.com", "password123")
        .await;

    let resp = app.login("wrongpw@example.com", "wrongpassword").await;
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn login_with_nonexistent_email_returns_401() {
    let app = common::TestApp::spawn().await;

    let resp = app.login("nobody@example.com", "password123").await;
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn login_rejects_invalid_email_format() {
    let app = common::TestApp::spawn().await;

    let resp = app.login("not-an-email", "password123").await;
    assert_eq!(resp.status(), 422);
}

#[tokio::test]
#[serial]
async fn login_rejects_empty_password() {
    let app = common::TestApp::spawn().await;

    let resp = app.login("test@example.com", "").await;
    assert_eq!(resp.status(), 422);
}

// ── Me ──────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn me_without_cookie_returns_401() {
    let app = common::TestApp::spawn().await;

    // Fresh client — no cookie jar history
    let client = reqwest::Client::new();
    let resp = client.get(app.url("/api/v1/auth/me")).send().await.unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn me_after_register_returns_profile_with_stats() {
    let app = common::TestApp::spawn().await;

    app.register_and_login().await;

    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "testuser");
    assert_eq!(body["email"], "test@example.com");

    // Fresh user should have zeroed stats
    let stats = &body["stats"];
    assert_eq!(stats["current_streak"], 0);
    assert_eq!(stats["longest_streak"], 0);
    assert_eq!(stats["total_solved"], 0);
    assert_eq!(stats["total_attempts"], 0);
}

// ── Logout ──────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn logout_clears_session() {
    let app = common::TestApp::spawn().await;

    app.register_and_login().await;

    // Verify we're authenticated
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Logout
    let resp = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // me should now fail
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
#[serial]
async fn login_after_logout_works() {
    let app = common::TestApp::spawn().await;

    app.register("relogin", "relogin@example.com", "password123")
        .await;

    // Logout
    app.client
        .post(app.url("/api/v1/auth/logout"))
        .send()
        .await
        .unwrap();

    // Login again
    let resp = app.login("relogin@example.com", "password123").await;
    assert_eq!(resp.status(), 200);

    // me should work again
    let resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn register_with_missing_fields_returns_422() {
    let app = common::TestApp::spawn().await;

    // Missing password entirely
    let resp = app
        .client
        .post(app.url("/api/v1/auth/register"))
        .json(&json!({
            "username": "user",
            "email": "test@example.com"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 422);
}

#[tokio::test]
#[serial]
async fn login_with_missing_fields_returns_422() {
    let app = common::TestApp::spawn().await;

    // Missing password
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "email": "test@example.com"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 422);
}
