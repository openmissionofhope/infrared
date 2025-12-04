//! SQLite storage layer for Infrared.
//!
//! # Privacy Guarantees
//!
//! This module handles all database operations. The schema is intentionally minimal:
//!
//! - `bucket`: Coarse category identifier (no PII)
//! - `ts`: Unix timestamp in seconds (server-assigned)
//! - `weight`: Numeric intensity (anonymous)
//!
//! **No identifying information is ever stored in the database.**
//! If the entire database were leaked, no individual could be identified.

use chrono::{DateTime, TimeZone, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

use crate::model::LifeSignal;

/// Database connection pool wrapper.
#[derive(Clone)]
pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    /// Create a new storage instance and initialize the schema.
    ///
    /// # Arguments
    ///
    /// * `database_url` - SQLite connection string (e.g., "sqlite:infrared.db" or "sqlite::memory:")
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let storage = Self { pool };
        storage.initialize_schema().await?;

        Ok(storage)
    }

    /// Create the database schema if it doesn't exist.
    ///
    /// # Privacy Note
    ///
    /// The schema contains ONLY aggregate-safe columns:
    /// - No user IDs, IPs, device IDs, or any identifying fields
    /// - Only bucket (category), timestamp, and weight
    async fn initialize_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS life_signals (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bucket TEXT NOT NULL,
                ts INTEGER NOT NULL,
                weight INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Index for efficient time-range queries by bucket
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_life_signals_bucket_ts
            ON life_signals(bucket, ts)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Insert a new life signal into storage.
    ///
    /// # Privacy Note
    ///
    /// This function intentionally does NOT log or store:
    /// - Client IP addresses
    /// - Request headers
    /// - Any identifying information
    ///
    /// Only the bucket, server-assigned timestamp, and weight are recorded.
    pub async fn insert_life_signal(&self, signal: &LifeSignal) -> anyhow::Result<()> {
        let ts = signal.timestamp.timestamp();

        sqlx::query(
            r#"
            INSERT INTO life_signals (bucket, ts, weight)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(&signal.bucket)
        .bind(ts)
        .bind(signal.weight)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query the total weight of signals in a bucket within a time window.
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket to query
    /// * `window_minutes` - Size of the time window in minutes
    /// * `now` - The reference timestamp (typically current time)
    ///
    /// # Returns
    ///
    /// Sum of weights for signals in the window, or 0 if none found.
    pub async fn query_bucket_window(
        &self,
        bucket: &str,
        window_minutes: u32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<i64> {
        let window_seconds = i64::from(window_minutes) * 60;
        let now_ts = now.timestamp();
        let start_ts = now_ts - window_seconds;

        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(weight), 0) as total
            FROM life_signals
            WHERE bucket = ? AND ts >= ? AND ts <= ?
            "#,
        )
        .bind(bucket)
        .bind(start_ts)
        .bind(now_ts)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("total"))
    }

    /// Compute the average weight per window over recent history.
    ///
    /// # Arguments
    ///
    /// * `bucket` - The bucket to query
    /// * `window_minutes` - Size of each time window in minutes
    /// * `num_windows` - Number of historical windows to average
    /// * `now` - The reference timestamp
    ///
    /// # Returns
    ///
    /// Average weight per window. Returns 0.0 if no data exists.
    pub async fn compute_recent_average(
        &self,
        bucket: &str,
        window_minutes: u32,
        num_windows: u32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<f64> {
        let window_seconds = i64::from(window_minutes) * 60;
        let total_seconds = window_seconds * i64::from(num_windows);
        let now_ts = now.timestamp();
        // Start from one window ago (exclude current window)
        let end_ts = now_ts - window_seconds;
        let start_ts = end_ts - total_seconds;

        // Use SQL to bin signals into windows and compute average
        let row = sqlx::query(
            r#"
            SELECT COALESCE(AVG(window_total), 0.0) as avg_total
            FROM (
                SELECT (ts / ?) as window_id, SUM(weight) as window_total
                FROM life_signals
                WHERE bucket = ? AND ts >= ? AND ts < ?
                GROUP BY window_id
            )
            "#,
        )
        .bind(window_seconds)
        .bind(bucket)
        .bind(start_ts)
        .bind(end_ts)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("avg_total"))
    }

    /// Get the timestamp of the most recent signal for a bucket.
    ///
    /// # Returns
    ///
    /// The timestamp of the last signal, or None if no signals exist.
    pub async fn get_last_seen(&self, bucket: &str) -> anyhow::Result<Option<DateTime<Utc>>> {
        let row = sqlx::query(
            r#"
            SELECT MAX(ts) as last_ts
            FROM life_signals
            WHERE bucket = ?
            "#,
        )
        .bind(bucket)
        .fetch_one(&self.pool)
        .await?;

        let last_ts: Option<i64> = row.get("last_ts");
        Ok(last_ts.map(|ts| Utc.timestamp_opt(ts, 0).unwrap()))
    }

    /// Get all distinct buckets that have signals within a time range.
    ///
    /// # Arguments
    ///
    /// * `minutes` - Lookback window in minutes
    /// * `now` - The reference timestamp
    pub async fn get_active_buckets(
        &self,
        minutes: u32,
        now: DateTime<Utc>,
    ) -> anyhow::Result<Vec<String>> {
        let window_seconds = i64::from(minutes) * 60;
        let now_ts = now.timestamp();
        let start_ts = now_ts - window_seconds;

        let rows = sqlx::query(
            r#"
            SELECT DISTINCT bucket
            FROM life_signals
            WHERE ts >= ?
            "#,
        )
        .bind(start_ts)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| r.get("bucket")).collect())
    }

    /// Get all buckets that have ever had signals (for alert checking).
    pub async fn get_all_known_buckets(&self) -> anyhow::Result<Vec<String>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT bucket FROM life_signals
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| r.get("bucket")).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_insert_and_query() {
        let storage = Storage::new("sqlite::memory:").await.unwrap();

        let now = Utc::now();
        let signal = LifeSignal {
            bucket: "test-bucket".to_string(),
            timestamp: now,
            weight: 5,
        };

        storage.insert_life_signal(&signal).await.unwrap();

        let total = storage
            .query_bucket_window("test-bucket", 10, now + chrono::Duration::seconds(1))
            .await
            .unwrap();

        assert_eq!(total, 5);
    }

    #[tokio::test]
    async fn test_multiple_signals() {
        let storage = Storage::new("sqlite::memory:").await.unwrap();

        let now = Utc::now();

        for i in 0..5 {
            let signal = LifeSignal {
                bucket: "test-bucket".to_string(),
                timestamp: now - chrono::Duration::minutes(i),
                weight: 10,
            };
            storage.insert_life_signal(&signal).await.unwrap();
        }

        let total = storage
            .query_bucket_window("test-bucket", 10, now + chrono::Duration::seconds(1))
            .await
            .unwrap();

        assert_eq!(total, 50);
    }

    #[tokio::test]
    async fn test_get_last_seen() {
        let storage = Storage::new("sqlite::memory:").await.unwrap();

        // No signals yet
        let last = storage.get_last_seen("test-bucket").await.unwrap();
        assert!(last.is_none());

        let now = Utc::now();
        let signal = LifeSignal {
            bucket: "test-bucket".to_string(),
            timestamp: now,
            weight: 1,
        };
        storage.insert_life_signal(&signal).await.unwrap();

        let last = storage.get_last_seen("test-bucket").await.unwrap();
        assert!(last.is_some());
    }
}
