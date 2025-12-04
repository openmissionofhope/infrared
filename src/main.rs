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
//! ## Core Endpoints
//!
//! - `POST /signal` - Record a life signal
//! - `GET /warmth` - Query the warmth index for a bucket
//! - `GET /alerts/recent` - Get alerts for buckets in distress
//! - `GET /health` - Health check
//!
//! ## Dashboard Endpoints (requires configuration)
//!
//! - `GET /dashboard` - Aggregated issues from all data sources
//! - `GET /dashboard/summary` - Summary statistics only
//! - `GET /dashboard/country/:code` - Issues for a specific country
//! - `GET /dashboard/source/:source` - Issues from a specific source

use std::env;
use std::net::SocketAddr;

use axum::{Router, routing::get, routing::post};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use infrared::api::{
    AppState, get_alerts, get_dashboard, get_dashboard_by_country, get_dashboard_by_source,
    get_dashboard_summary, get_warmth, health_check, post_signal,
};
use infrared::dashboard::{Dashboard, DashboardConfig};
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

    // Initialize dashboard if configured
    let dashboard = create_dashboard_if_configured();
    let dashboard_enabled = dashboard.is_some();

    // Create application state
    let state = AppState { storage, dashboard };

    // Build router
    // PRIVACY NOTE: We do NOT use any middleware that logs IP addresses or headers
    let mut app = Router::new()
        .route("/signal", post(post_signal))
        .route("/warmth", get(get_warmth))
        .route("/alerts/recent", get(get_alerts))
        .route("/health", get(health_check));

    // Add dashboard routes if configured
    if dashboard_enabled {
        app = app
            .route("/dashboard", get(get_dashboard))
            .route("/dashboard/summary", get(get_dashboard_summary))
            .route("/dashboard/country/:code", get(get_dashboard_by_country))
            .route("/dashboard/source/:source", get(get_dashboard_by_source));
        info!("Dashboard enabled with external data sources");
    } else {
        info!("Dashboard disabled (set ACLED_EMAIL/ACLED_KEY for full functionality)");
    }

    let app = app.with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "Infrared is listening");
    info!("Privacy mode: ENABLED (no PII logging, no IP tracking)");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Create dashboard configuration from environment variables.
///
/// # Environment Variables
///
/// - `ACLED_EMAIL` - Email for ACLED API authentication (optional)
/// - `ACLED_KEY` - API key for ACLED API authentication (optional)
/// - `CLOUDFLARE_TOKEN` - Cloudflare API token for higher rate limits (optional)
/// - `DASHBOARD_APP_ID` - Application identifier for HDX/ReliefWeb (default: "infrared")
/// - `DASHBOARD_LOOKBACK_HOURS` - Hours to look back for issues (default: 24)
fn create_dashboard_if_configured() -> Option<Dashboard> {
    let config = DashboardConfig {
        acled_email: env::var("ACLED_EMAIL").ok(),
        acled_key: env::var("ACLED_KEY").ok(),
        cloudflare_token: env::var("CLOUDFLARE_TOKEN").ok(),
        app_identifier: env::var("DASHBOARD_APP_ID").unwrap_or_else(|_| "infrared".to_string()),
        monitored_countries: vec![], // Countries can be configured via API or extended config
        lookback_hours: env::var("DASHBOARD_LOOKBACK_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(24),
    };

    // Dashboard is always enabled, but ACLED data requires authentication
    Some(Dashboard::new(config))
}
