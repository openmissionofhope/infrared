//! Integration tests for Infrared API endpoints.
//!
//! These tests verify the full request/response cycle through the HTTP API.

use axum::{Router, routing::get, routing::post};
use axum_test::TestServer;
use serde_json::json;

// Import from the infrared crate
use infrared::api::{AppState, get_alerts, get_warmth, health_check, post_signal};
use infrared::storage::Storage;

async fn create_test_server() -> TestServer {
    let storage = Storage::new("sqlite::memory:").await.unwrap();
    let state = AppState {
        storage,
        dashboard: None, // Dashboard not needed for core API tests
    };

    let app = Router::new()
        .route("/signal", post(post_signal))
        .route("/warmth", get(get_warmth))
        .route("/alerts/recent", get(get_alerts))
        .route("/health", get(health_check))
        .with_state(state);

    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server().await;

    let response = server.get("/health").await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_post_signal() {
    let server = create_test_server().await;

    let response = server
        .post("/signal")
        .json(&json!({
            "bucket": "test-zone",
            "weight": 5
        }))
        .await;

    response.assert_status(axum::http::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_post_signal_default_weight() {
    let server = create_test_server().await;

    let response = server
        .post("/signal")
        .json(&json!({
            "bucket": "test-zone"
        }))
        .await;

    response.assert_status(axum::http::StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_get_warmth_empty_bucket() {
    let server = create_test_server().await;

    let response = server.get("/warmth?bucket=empty-bucket").await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["bucket"], "empty-bucket");
    assert_eq!(body["current_window_total"], 0);
    assert_eq!(body["status"], "alive"); // No baseline = assume alive
}

#[tokio::test]
async fn test_get_warmth_with_signals() {
    let server = create_test_server().await;

    // Post some signals
    for _ in 0..5 {
        server
            .post("/signal")
            .json(&json!({
                "bucket": "active-zone",
                "weight": 10
            }))
            .await;
    }

    let response = server
        .get("/warmth?bucket=active-zone&window_minutes=10")
        .await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["bucket"], "active-zone");
    assert_eq!(body["current_window_total"], 50);
    assert_eq!(body["window_minutes"], 10);
}

#[tokio::test]
async fn test_get_alerts_empty() {
    let server = create_test_server().await;

    let response = server.get("/alerts/recent?minutes=60").await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["lookback_minutes"], 60);
    assert!(body["alerts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_alerts_default_minutes() {
    let server = create_test_server().await;

    let response = server.get("/alerts/recent").await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["lookback_minutes"], 60); // Default value
}

#[tokio::test]
async fn test_full_workflow() {
    let server = create_test_server().await;

    // 1. Health check
    server.get("/health").await.assert_status_ok();

    // 2. Post signals to multiple buckets
    for bucket in ["zone-a", "zone-b", "zone-c"] {
        for _ in 0..10 {
            server
                .post("/signal")
                .json(&json!({
                    "bucket": bucket,
                    "weight": 1
                }))
                .await
                .assert_status(axum::http::StatusCode::ACCEPTED);
        }
    }

    // 3. Query warmth for each bucket
    for bucket in ["zone-a", "zone-b", "zone-c"] {
        let response = server.get(&format!("/warmth?bucket={}", bucket)).await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert_eq!(body["current_window_total"], 10);
    }

    // 4. Check alerts (should be empty since all zones are active)
    let response = server.get("/alerts/recent").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["alerts"].as_array().unwrap().is_empty());
}
