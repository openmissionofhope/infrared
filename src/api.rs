//! HTTP API handlers for Infrared.
//!
//! # Privacy Guarantees
//!
//! These handlers are designed with privacy as a core principle:
//!
//! - **POST /signal**: Does NOT log client IPs, headers, or any identifying information.
//!   Only the bucket and weight are recorded.
//!
//! - **GET /warmth**: Returns aggregate statistics only. No individual signals are exposed.
//!
//! - **GET /alerts/recent**: Reports bucket-level status. No user data is revealed.
//!
//! All logging uses structured tracing that explicitly excludes:
//! - IP addresses
//! - User agents
//! - Session identifiers
//! - Any personally identifiable information

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::Deserialize;
use tracing::{info, instrument, warn};

use crate::aggregation::{compute_warmth, generate_alerts};
use crate::dashboard::{Dashboard, DashboardResponse, IssueSource};
use crate::model::{
    AlertsQuery, AlertsResponse, LifeSignal, SignalRequest, WarmthQuery, WarmthResponse,
};
use crate::storage::Storage;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub dashboard: Option<Dashboard>,
}

/// POST /signal - Record a life signal.
///
/// # Privacy Note
///
/// This handler intentionally does NOT:
/// - Log the client's IP address
/// - Store any request headers
/// - Record any identifying information
///
/// Only the bucket and weight are stored, with a server-assigned timestamp.
///
/// # Request Body
///
/// ```json
/// {
///     "bucket": "zone-a",
///     "weight": 1
/// }
/// ```
///
/// Weight is optional and defaults to 1.
///
/// # Response
///
/// Returns `202 Accepted` on success.
#[instrument(skip(state), fields(bucket, weight))]
pub async fn post_signal(
    State(state): State<AppState>,
    Json(request): Json<SignalRequest>,
) -> impl IntoResponse {
    // Log only non-identifying information
    // PRIVACY: We explicitly do NOT log client IP, headers, or any PII
    tracing::Span::current().record("bucket", &request.bucket);
    tracing::Span::current().record("weight", request.weight);

    let signal = LifeSignal {
        bucket: request.bucket.clone(),
        timestamp: Utc::now(), // Server-assigned timestamp
        weight: request.weight,
    };

    match state.storage.insert_life_signal(&signal).await {
        Ok(()) => {
            info!(
                bucket = %signal.bucket,
                weight = signal.weight,
                "Life signal recorded"
            );
            StatusCode::ACCEPTED
        }
        Err(e) => {
            warn!(
                bucket = %signal.bucket,
                error = %e,
                "Failed to record life signal"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// GET /warmth - Query the warmth index for a bucket.
///
/// # Query Parameters
///
/// - `bucket` (required): The bucket to query
/// - `window_minutes` (optional): Time window in minutes (default: 10)
///
/// # Response
///
/// ```json
/// {
///     "bucket": "zone-a",
///     "window_minutes": 10,
///     "current_window_total": 42,
///     "recent_average": 50.5,
///     "status": "alive"
/// }
/// ```
///
/// Status can be: "alive", "stressed", "collapsing", or "dead"
#[instrument(skip(state))]
pub async fn get_warmth(
    State(state): State<AppState>,
    Query(query): Query<WarmthQuery>,
) -> Result<Json<WarmthResponse>, StatusCode> {
    let now = Utc::now();

    match compute_warmth(&state.storage, &query.bucket, query.window_minutes, now).await {
        Ok(response) => {
            info!(
                bucket = %response.bucket,
                status = ?response.status,
                current = response.current_window_total,
                average = %response.recent_average,
                "Warmth queried"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(
                bucket = %query.bucket,
                error = %e,
                "Failed to compute warmth"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /alerts/recent - Get recent alerts for buckets in distress.
///
/// # Query Parameters
///
/// - `minutes` (optional): Lookback window in minutes (default: 60)
///
/// # Response
///
/// ```json
/// {
///     "alerts": [
///         {
///             "bucket": "zone-a",
///             "status": "dead",
///             "last_seen_timestamp": "2024-01-15T10:30:00Z",
///             "recent_average": 50.0,
///             "message": "CRITICAL: Bucket 'zone-a' has gone completely silent..."
///         }
///     ],
///     "lookback_minutes": 60
/// }
/// ```
#[instrument(skip(state))]
pub async fn get_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> Result<Json<AlertsResponse>, StatusCode> {
    let now = Utc::now();

    match generate_alerts(&state.storage, query.minutes, now).await {
        Ok(response) => {
            info!(
                alert_count = response.alerts.len(),
                lookback_minutes = query.minutes,
                "Alerts queried"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(
                lookback_minutes = query.minutes,
                error = %e,
                "Failed to generate alerts"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /health - Simple health check endpoint.
pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

// ============================================================================
// Dashboard API handlers
// ============================================================================

/// Query parameters for the dashboard endpoint.
#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    /// Filter by source (ioda, cloudflare_radar, hdx_hapi, acled, reliefweb).
    pub source: Option<String>,
    /// Filter by country code.
    pub country: Option<String>,
}

/// GET /dashboard - Get aggregated issues from all data sources.
///
/// # Query Parameters
///
/// - `source` (optional): Filter by source (ioda, cloudflare_radar, hdx_hapi, acled, reliefweb)
/// - `country` (optional): Filter by country code
///
/// # Response
///
/// Returns a JSON object with:
/// - `timestamp`: When the response was generated
/// - `summary`: Summary statistics (counts by severity, source, category)
/// - `issues`: List of issues sorted by severity and timestamp
/// - `errors`: Any errors encountered while fetching from sources
#[instrument(skip(state))]
pub async fn get_dashboard(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, StatusCode> {
    let dashboard = state.dashboard.as_ref().ok_or_else(|| {
        warn!("Dashboard not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    // Filter by country if specified
    if let Some(country) = &query.country {
        match dashboard.get_issues_by_country(country).await {
            Ok(issues) => {
                let summary = crate::dashboard::DashboardSummary::from_issues(&issues);
                let response = DashboardResponse {
                    timestamp: Utc::now(),
                    summary,
                    issues,
                    errors: vec![],
                };
                info!(
                    country = %country,
                    issue_count = response.issues.len(),
                    "Dashboard queried by country"
                );
                return Ok(Json(response));
            }
            Err(e) => {
                warn!(country = %country, error = %e, "Failed to fetch dashboard by country");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Filter by source if specified
    if let Some(source_str) = &query.source {
        let source = match source_str.as_str() {
            "ioda" => IssueSource::Ioda,
            "cloudflare_radar" | "cloudflare" => IssueSource::CloudflareRadar,
            "hdx_hapi" | "hdx" | "hapi" => IssueSource::HdxHapi,
            "acled" => IssueSource::Acled,
            "reliefweb" => IssueSource::ReliefWeb,
            _ => {
                warn!(source = %source_str, "Invalid source filter");
                return Err(StatusCode::BAD_REQUEST);
            }
        };

        match dashboard.get_issues_by_source(source).await {
            Ok(issues) => {
                let summary = crate::dashboard::DashboardSummary::from_issues(&issues);
                let response = DashboardResponse {
                    timestamp: Utc::now(),
                    summary,
                    issues,
                    errors: vec![],
                };
                info!(
                    source = %source_str,
                    issue_count = response.issues.len(),
                    "Dashboard queried by source"
                );
                return Ok(Json(response));
            }
            Err(e) => {
                warn!(source = %source_str, error = %e, "Failed to fetch dashboard by source");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    }

    // Get all issues
    match dashboard.get_all_issues().await {
        Ok(response) => {
            info!(
                issue_count = response.issues.len(),
                error_count = response.errors.len(),
                "Dashboard queried"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch dashboard");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /dashboard/summary - Get just the summary statistics.
#[instrument(skip(state))]
pub async fn get_dashboard_summary(
    State(state): State<AppState>,
) -> Result<Json<crate::dashboard::DashboardSummary>, StatusCode> {
    let dashboard = state.dashboard.as_ref().ok_or_else(|| {
        warn!("Dashboard not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    match dashboard.get_all_issues().await {
        Ok(response) => {
            info!(
                total_issues = response.summary.total_issues,
                emergency_count = response.summary.emergency_count,
                critical_count = response.summary.critical_count,
                "Dashboard summary queried"
            );
            Ok(Json(response.summary))
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch dashboard summary");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /dashboard/country/:code - Get issues for a specific country.
#[instrument(skip(state))]
pub async fn get_dashboard_by_country(
    State(state): State<AppState>,
    Path(country_code): Path<String>,
) -> Result<Json<DashboardResponse>, StatusCode> {
    let dashboard = state.dashboard.as_ref().ok_or_else(|| {
        warn!("Dashboard not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    match dashboard.get_issues_by_country(&country_code).await {
        Ok(issues) => {
            let summary = crate::dashboard::DashboardSummary::from_issues(&issues);
            let response = DashboardResponse {
                timestamp: Utc::now(),
                summary,
                issues,
                errors: vec![],
            };
            info!(
                country = %country_code,
                issue_count = response.issues.len(),
                "Dashboard queried by country"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(country = %country_code, error = %e, "Failed to fetch dashboard by country");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /dashboard/source/:source - Get issues from a specific source.
#[instrument(skip(state))]
pub async fn get_dashboard_by_source(
    State(state): State<AppState>,
    Path(source_str): Path<String>,
) -> Result<Json<DashboardResponse>, StatusCode> {
    let dashboard = state.dashboard.as_ref().ok_or_else(|| {
        warn!("Dashboard not configured");
        StatusCode::SERVICE_UNAVAILABLE
    })?;

    let source = match source_str.as_str() {
        "ioda" => IssueSource::Ioda,
        "cloudflare_radar" | "cloudflare" => IssueSource::CloudflareRadar,
        "hdx_hapi" | "hdx" | "hapi" => IssueSource::HdxHapi,
        "acled" => IssueSource::Acled,
        "reliefweb" => IssueSource::ReliefWeb,
        _ => {
            warn!(source = %source_str, "Invalid source");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    match dashboard.get_issues_by_source(source).await {
        Ok(issues) => {
            let summary = crate::dashboard::DashboardSummary::from_issues(&issues);
            let response = DashboardResponse {
                timestamp: Utc::now(),
                summary,
                issues,
                errors: vec![],
            };
            info!(
                source = %source_str,
                issue_count = response.issues.len(),
                "Dashboard queried by source"
            );
            Ok(Json(response))
        }
        Err(e) => {
            warn!(source = %source_str, error = %e, "Failed to fetch dashboard by source");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
