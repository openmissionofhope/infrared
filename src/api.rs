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
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use tracing::{info, instrument, warn};

use crate::aggregation::{compute_warmth, generate_alerts};
use crate::model::{
    AlertsQuery, AlertsResponse, LifeSignal, SignalRequest, WarmthQuery, WarmthResponse,
};
use crate::storage::Storage;

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
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
