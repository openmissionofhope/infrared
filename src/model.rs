//! Data models for Infrared.
//!
//! # Privacy Guarantees
//!
//! All types in this module are designed to be **privacy-safe by construction**.
//! They intentionally exclude any fields that could identify individuals:
//!
//! - No usernames, emails, or account IDs
//! - No IP addresses or device identifiers
//! - No GPS coordinates or precise locations
//! - No biometric data or personal attributes
//! - No content or message bodies
//!
//! If the database or logs containing these types were to leak publicly,
//! **no individual could be identified, tracked, or reconstructed**.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single "life signal" event.
///
/// Represents anonymous evidence that "something is alive" in a given bucket.
/// This is the core data type that Infrared collects and aggregates.
///
/// # Privacy
///
/// This struct intentionally contains **only** aggregate-safe fields:
/// - `bucket`: A coarse category defined by system configuration, not users
/// - `timestamp`: When the signal was recorded (server-side, not client-provided)
/// - `weight`: A numeric intensity, with no identifying characteristics
///
/// No field in this struct can be used to identify, locate, or profile any individual.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifeSignal {
    /// A coarse bucket identifier such as "region:north", "city:A", "cluster:web-01".
    ///
    /// This is defined by configuration/integration, never by end-users.
    /// Buckets should be broad enough that individual signals cannot be
    /// correlated back to specific people.
    pub bucket: String,

    /// Server-side timestamp when the signal was recorded (UTC).
    ///
    /// This is always set by the server, never provided by clients,
    /// to prevent timing-based identification attacks.
    pub timestamp: DateTime<Utc>,

    /// Optional weight (e.g., number of entities represented by this signal).
    ///
    /// Default = 1. This allows batching multiple life signals into one event
    /// for efficiency, while maintaining aggregate-only semantics.
    pub weight: i32,
}

/// Request body for POST /signal endpoint.
///
/// # Privacy
///
/// Clients provide only the bucket and optional weight.
/// The timestamp is set server-side to prevent timing attacks.
/// No identifying information is accepted or stored.
#[derive(Debug, Clone, Deserialize)]
pub struct SignalRequest {
    /// The bucket to record the signal in.
    pub bucket: String,

    /// Optional weight for this signal (defaults to 1).
    #[serde(default = "default_weight")]
    pub weight: i32,
}

fn default_weight() -> i32 {
    1
}

/// The health status of a bucket based on its warmth index.
///
/// Status is determined by comparing current activity to recent historical averages.
/// This provides early warning of population-level changes without tracking individuals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WarmthStatus {
    /// Current activity is at or above 80% of recent average.
    /// Life signals are flowing normally.
    Alive,

    /// Current activity is between 20% and 80% of recent average.
    /// Noticeable decline but not critical.
    Stressed,

    /// Current activity is below 20% of recent average but non-zero.
    /// Severe decline indicating potential crisis.
    Collapsing,

    /// No activity in current window while recent average was positive.
    /// Complete cessation of life signals.
    Dead,
}

impl WarmthStatus {
    /// Determine status based on current vs recent average activity.
    ///
    /// # Thresholds
    ///
    /// - `alive`: current >= 0.8 * recent_average
    /// - `stressed`: 0.2 * recent_average <= current < 0.8 * recent_average
    /// - `collapsing`: 0 < current < 0.2 * recent_average
    /// - `dead`: current == 0 && recent_average > 0
    ///
    /// If recent_average is 0, we return `Alive` (no baseline to compare against).
    pub fn from_activity(current: i64, recent_average: f64) -> Self {
        if recent_average <= 0.0 {
            // No historical baseline; assume alive
            return WarmthStatus::Alive;
        }

        let ratio = current as f64 / recent_average;

        if current == 0 {
            WarmthStatus::Dead
        } else if ratio < 0.2 {
            WarmthStatus::Collapsing
        } else if ratio < 0.8 {
            WarmthStatus::Stressed
        } else {
            WarmthStatus::Alive
        }
    }
}

/// Response for GET /warmth endpoint.
///
/// Provides the warmth index and status for a specific bucket.
#[derive(Debug, Clone, Serialize)]
pub struct WarmthResponse {
    /// The bucket being queried.
    pub bucket: String,

    /// The time window in minutes used for the current measurement.
    pub window_minutes: u32,

    /// Total weight of signals in the current window.
    pub current_window_total: i64,

    /// Average weight per window over recent history.
    pub recent_average: f64,

    /// Health status derived from current vs recent activity.
    pub status: WarmthStatus,
}

/// A single alert for a bucket in distress.
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    /// The bucket experiencing the issue.
    pub bucket: String,

    /// Current status of the bucket.
    pub status: WarmthStatus,

    /// When the last signal was received (if any).
    pub last_seen_timestamp: Option<DateTime<Utc>>,

    /// Historical average for context.
    pub recent_average: f64,

    /// Human-readable description of the alert.
    pub message: String,
}

/// Response for GET /alerts/recent endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AlertsResponse {
    /// List of buckets currently in distress.
    pub alerts: Vec<Alert>,

    /// The lookback window in minutes that was used.
    pub lookback_minutes: u32,
}

/// Query parameters for GET /warmth endpoint.
#[derive(Debug, Deserialize)]
pub struct WarmthQuery {
    /// The bucket to query.
    pub bucket: String,

    /// Time window in minutes (default: 10).
    #[serde(default = "default_window_minutes")]
    pub window_minutes: u32,
}

fn default_window_minutes() -> u32 {
    10
}

/// Query parameters for GET /alerts/recent endpoint.
#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    /// Lookback window in minutes (default: 60).
    #[serde(default = "default_lookback_minutes")]
    pub minutes: u32,
}

fn default_lookback_minutes() -> u32 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmth_status_alive() {
        // Current >= 80% of average
        assert_eq!(WarmthStatus::from_activity(100, 100.0), WarmthStatus::Alive);
        assert_eq!(WarmthStatus::from_activity(80, 100.0), WarmthStatus::Alive);
        assert_eq!(WarmthStatus::from_activity(120, 100.0), WarmthStatus::Alive);
    }

    #[test]
    fn test_warmth_status_stressed() {
        // 20% <= current < 80% of average
        assert_eq!(
            WarmthStatus::from_activity(79, 100.0),
            WarmthStatus::Stressed
        );
        assert_eq!(
            WarmthStatus::from_activity(50, 100.0),
            WarmthStatus::Stressed
        );
        assert_eq!(
            WarmthStatus::from_activity(20, 100.0),
            WarmthStatus::Stressed
        );
    }

    #[test]
    fn test_warmth_status_collapsing() {
        // 0 < current < 20% of average
        assert_eq!(
            WarmthStatus::from_activity(19, 100.0),
            WarmthStatus::Collapsing
        );
        assert_eq!(
            WarmthStatus::from_activity(1, 100.0),
            WarmthStatus::Collapsing
        );
    }

    #[test]
    fn test_warmth_status_dead() {
        // current == 0 while average > 0
        assert_eq!(WarmthStatus::from_activity(0, 100.0), WarmthStatus::Dead);
        assert_eq!(WarmthStatus::from_activity(0, 1.0), WarmthStatus::Dead);
    }

    #[test]
    fn test_warmth_status_no_baseline() {
        // No historical data; assume alive
        assert_eq!(WarmthStatus::from_activity(0, 0.0), WarmthStatus::Alive);
        assert_eq!(WarmthStatus::from_activity(10, 0.0), WarmthStatus::Alive);
    }
}
