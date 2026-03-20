#![allow(dead_code)]

mod common;

use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Serialize;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn cors_rejects_disallowed_origin() {
    let app = common::TestApp::spawn().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(app.url("/api/v1/health"))
        .header("Origin", "https://evil-site.com")
        .send()
        .await
        .unwrap();

    // Server should not echo back the evil origin
    let allow_origin = resp
        .headers()
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok());
    assert_ne!(allow_origin, Some("https://evil-site.com"));
}

#[tokio::test]
#[serial]
async fn cors_allows_configured_origin() {
    let app = common::TestApp::spawn().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(app.url("/api/v1/health"))
        .header("Origin", "http://localhost:3000")
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.headers().get("access-control-allow-origin").unwrap(),
        "http://localhost:3000"
    );
}

#[tokio::test]
#[serial]
async fn security_headers_are_present() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .client
        .get(app.url("/api/v1/health"))
        .send()
        .await
        .unwrap();

    let headers = resp.headers();
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("x-xss-protection").unwrap(), "1; mode=block");
    assert!(headers.get("strict-transport-security").is_some());
    assert!(headers.get("referrer-policy").is_some());
    assert!(headers.get("permissions-policy").is_some());
    assert!(headers.get("content-security-policy").is_some());
}

#[tokio::test]
#[serial]
async fn request_id_header_is_returned() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .client
        .get(app.url("/api/v1/health"))
        .send()
        .await
        .unwrap();

    let request_id = resp
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap();
    // Should be a valid UUID v4
    assert!(uuid::Uuid::parse_str(request_id).is_ok());
}

#[tokio::test]
#[serial]
async fn request_id_is_propagated_when_provided() {
    let app = common::TestApp::spawn().await;

    let custom_id = "my-custom-request-id-123";
    let resp = app
        .client
        .get(app.url("/api/v1/health"))
        .header("x-request-id", custom_id)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap(),
        custom_id
    );
}

#[tokio::test]
#[serial]
async fn auth_body_limit_rejects_oversized_payload() {
    let app = common::TestApp::spawn().await;

    // 32 KB of junk — exceeds the 16 KB auth limit
    let big_payload = "x".repeat(32 * 1024);

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .header("content-type", "application/json")
        .body(format!(
            r#"{{"email":"a@b.com","password":"{}"}}"#,
            big_payload
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 413);
}

// ── JWT issuer validation ──────────────────────────────────────────────────

#[derive(Serialize)]
struct FakeClaims {
    sub: String,
    exp: usize,
    iat: usize,
    iss: String,
}

#[tokio::test]
#[serial]
async fn jwt_with_wrong_issuer_is_rejected() {
    let app = common::TestApp::spawn().await;

    // Craft a JWT signed with the correct secret but with the wrong issuer
    let now = Utc::now();
    let claims = FakeClaims {
        sub: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        exp: (now + chrono::Duration::minutes(60)).timestamp() as usize,
        iat: now.timestamp() as usize,
        iss: "wrong-issuer".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(app.config().jwt_secret.as_bytes()),
    )
    .unwrap();

    // Use a fresh client (no cookie jar) and manually set the cookie
    let client = reqwest::Client::new();
    let resp = client
        .get(app.url("/api/v1/auth/me"))
        .header("Cookie", format!("access_token={token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}
