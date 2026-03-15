mod common;

use serial_test::serial;

#[tokio::test]
#[serial]
async fn health_check_returns_healthy() {
    let app = common::TestApp::spawn().await;

    let resp = app
        .client
        .get(app.url("/api/v1/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert!(body["version"].is_string());
}

#[tokio::test]
#[serial]
async fn health_check_does_not_require_auth() {
    let app = common::TestApp::spawn().await;

    // Fresh client, no cookies
    let client = reqwest::Client::new();
    let resp = client.get(app.url("/api/v1/health")).send().await.unwrap();

    assert_eq!(resp.status(), 200);
}
