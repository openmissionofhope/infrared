//! Infrared - A privacy-preserving system for detecting signs of life at scale.
//!
//! # Overview
//!
//! Infrared tracks aggregate "warmth" (activity/presence) at a bucket/region level
//! without tracking individuals. It measures population-level life signals and
//! detects large-scale drops or disappearances.
//!
//! # Privacy Guarantees
//!
//! Infrared is designed to be **privacy-safe by construction**:
//!
//! - No identity tracking (no usernames, emails, account IDs)
//! - No location tracking (no GPS, IP addresses)
//! - No device tracking (no device IDs, fingerprints)
//! - No behavioral profiling (no cross-session linking)
//!
//! If the entire database were leaked publicly, **no individual could be
//! identified, located, or profiled**.
//!
//! # API Endpoints
//!
//! - `POST /signal` - Record a life signal
//! - `GET /warmth` - Query the warmth index for a bucket
//! - `GET /alerts/recent` - Get alerts for buckets in distress
//! - `GET /health` - Health check

use std::env;
use std::net::SocketAddr;

use axum::{Router, routing::get, routing::post};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use infrared::api::{AppState, get_alerts, get_warmth, health_check, post_signal};
use infrared::storage::Storage;

/// Default port if not specified via environment variable.
const DEFAULT_PORT: u16 = 3000;

/// Default database path if not specified via environment variable.
const DEFAULT_DB_PATH: &str = "sqlite:infrared.db?mode=rwc";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with environment filter
    // PRIVACY NOTE: Default log level is INFO to avoid accidentally logging sensitive data
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("infrared=info".parse()?))
        .init();

    // Load configuration from environment
    let port: u16 = env::var("INFRARED_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let db_url = env::var("INFRARED_DATABASE_URL").unwrap_or_else(|_| DEFAULT_DB_PATH.to_string());

    info!(port, db_url = %db_url, "Starting Infrared server");

    // Initialize storage
    let storage = Storage::new(&db_url).await?;
    info!("Database initialized");

    // Create application state
    let state = AppState { storage };

    // Build router
    // PRIVACY NOTE: We do NOT use any middleware that logs IP addresses or headers
    let app = Router::new()
        .route("/signal", post(post_signal))
        .route("/warmth", get(get_warmth))
        .route("/alerts/recent", get(get_alerts))
        .route("/health", get(health_check))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "Infrared is listening");
    info!("Privacy mode: ENABLED (no PII logging, no IP tracking)");

    axum::serve(listener, app).await?;

    Ok(())
}
