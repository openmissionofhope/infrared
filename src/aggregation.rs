//! Aggregation logic for computing warmth indices and alerts.
//!
//! # Privacy Guarantees
//!
//! All computations in this module operate on **aggregate data only**.
//! No individual signals can be traced back to specific users or entities.
//! The warmth index reflects population-level activity, not individual behavior.

use chrono::{DateTime, Utc};

use crate::model::{Alert, AlertsResponse, WarmthResponse, WarmthStatus};
use crate::storage::Storage;

/// Number of historical windows to use when computing the recent average.
const NUM_HISTORICAL_WINDOWS: u32 = 6;

/// Compute the warmth index for a specific bucket.
///
/// This function queries the storage layer to get:
/// 1. Current window total (sum of weights in the latest time window)
/// 2. Recent average (average of the previous N windows)
///
/// It then derives the `WarmthStatus` based on the ratio of current to average.
///
/// # Arguments
///
/// * `storage` - Database connection
/// * `bucket` - The bucket to analyze
/// * `window_minutes` - Size of time windows in minutes
/// * `now` - Reference timestamp (typically current time)
///
/// # Returns
///
/// A `WarmthResponse` containing the bucket's current warmth index and status.
pub async fn compute_warmth(
    storage: &Storage,
    bucket: &str,
    window_minutes: u32,
    now: DateTime<Utc>,
) -> anyhow::Result<WarmthResponse> {
    // Get current window total
    let current_window_total = storage
        .query_bucket_window(bucket, window_minutes, now)
        .await?;

    // Get recent average (excluding current window)
    let recent_average = storage
        .compute_recent_average(bucket, window_minutes, NUM_HISTORICAL_WINDOWS, now)
        .await?;

    // Derive status
    let status = WarmthStatus::from_activity(current_window_total, recent_average);

    Ok(WarmthResponse {
        bucket: bucket.to_string(),
        window_minutes,
        current_window_total,
        recent_average,
        status,
    })
}

/// Generate alerts for all buckets in distress.
///
/// Scans all known buckets and identifies those with `Collapsing` or `Dead` status.
/// Returns a list of alerts with human-readable messages.
///
/// # Arguments
///
/// * `storage` - Database connection
/// * `lookback_minutes` - How far back to look for historical data
/// * `now` - Reference timestamp
///
/// # Returns
///
/// An `AlertsResponse` containing all current alerts.
pub async fn generate_alerts(
    storage: &Storage,
    lookback_minutes: u32,
    now: DateTime<Utc>,
) -> anyhow::Result<AlertsResponse> {
    // Use a reasonable window size for alert checking
    let window_minutes = lookback_minutes.min(10);

    // Get all buckets that have ever had signals
    let buckets = storage.get_all_known_buckets().await?;

    let mut alerts = Vec::new();

    for bucket in buckets {
        let warmth = compute_warmth(storage, &bucket, window_minutes, now).await?;

        // Only alert on collapsing or dead buckets
        if matches!(warmth.status, WarmthStatus::Collapsing | WarmthStatus::Dead) {
            let last_seen = storage.get_last_seen(&bucket).await?;

            let message = generate_alert_message(&bucket, warmth.status, &warmth);

            alerts.push(Alert {
                bucket: bucket.clone(),
                status: warmth.status,
                last_seen_timestamp: last_seen,
                recent_average: warmth.recent_average,
                message,
            });
        }
    }

    Ok(AlertsResponse {
        alerts,
        lookback_minutes,
    })
}

/// Generate a human-readable alert message.
fn generate_alert_message(bucket: &str, status: WarmthStatus, warmth: &WarmthResponse) -> String {
    match status {
        WarmthStatus::Dead => {
            format!(
                "CRITICAL: Bucket '{}' has gone completely silent. \
                 No signals received in the current window. \
                 Historical average was {:.1} signals per window.",
                bucket, warmth.recent_average
            )
        }
        WarmthStatus::Collapsing => {
            let percentage = if warmth.recent_average > 0.0 {
                (warmth.current_window_total as f64 / warmth.recent_average * 100.0) as i32
            } else {
                0
            };
            format!(
                "WARNING: Bucket '{}' is collapsing. \
                 Current activity ({}) is only {}% of recent average ({:.1}).",
                bucket, warmth.current_window_total, percentage, warmth.recent_average
            )
        }
        _ => format!("Bucket '{}' status: {:?}", bucket, status),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LifeSignal;

    async fn setup_test_storage() -> Storage {
        Storage::new("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_compute_warmth_no_data() {
        let storage = setup_test_storage().await;
        let now = Utc::now();

        let warmth = compute_warmth(&storage, "empty-bucket", 10, now)
            .await
            .unwrap();

        assert_eq!(warmth.bucket, "empty-bucket");
        assert_eq!(warmth.current_window_total, 0);
        assert_eq!(warmth.recent_average, 0.0);
        // With no baseline, status should be Alive
        assert_eq!(warmth.status, WarmthStatus::Alive);
    }

    #[tokio::test]
    async fn test_compute_warmth_alive() {
        let storage = setup_test_storage().await;
        let now = Utc::now();

        // Insert signals in historical windows
        for i in 1..=6 {
            let signal = LifeSignal {
                bucket: "test-bucket".to_string(),
                timestamp: now - chrono::Duration::minutes(i64::from(i) * 10 + 5),
                weight: 100,
            };
            storage.insert_life_signal(&signal).await.unwrap();
        }

        // Insert signals in current window
        let current_signal = LifeSignal {
            bucket: "test-bucket".to_string(),
            timestamp: now - chrono::Duration::minutes(5),
            weight: 100,
        };
        storage.insert_life_signal(&current_signal).await.unwrap();

        let warmth = compute_warmth(&storage, "test-bucket", 10, now)
            .await
            .unwrap();

        assert_eq!(warmth.status, WarmthStatus::Alive);
    }

    #[tokio::test]
    async fn test_generate_alerts_empty() {
        let storage = setup_test_storage().await;
        let now = Utc::now();

        let alerts = generate_alerts(&storage, 60, now).await.unwrap();

        assert!(alerts.alerts.is_empty());
    }

    #[tokio::test]
    async fn test_alert_message_dead() {
        let warmth = WarmthResponse {
            bucket: "zone-a".to_string(),
            window_minutes: 10,
            current_window_total: 0,
            recent_average: 50.0,
            status: WarmthStatus::Dead,
        };

        let message = generate_alert_message("zone-a", WarmthStatus::Dead, &warmth);

        assert!(message.contains("CRITICAL"));
        assert!(message.contains("zone-a"));
        assert!(message.contains("silent"));
    }

    #[tokio::test]
    async fn test_alert_message_collapsing() {
        let warmth = WarmthResponse {
            bucket: "zone-b".to_string(),
            window_minutes: 10,
            current_window_total: 5,
            recent_average: 100.0,
            status: WarmthStatus::Collapsing,
        };

        let message = generate_alert_message("zone-b", WarmthStatus::Collapsing, &warmth);

        assert!(message.contains("WARNING"));
        assert!(message.contains("zone-b"));
        assert!(message.contains("collapsing"));
    }
}
