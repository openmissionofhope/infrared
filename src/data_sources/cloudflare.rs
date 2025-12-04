//! Cloudflare Radar API client.
//!
//! Cloudflare Radar provides near real-time insights into global Internet traffic
//! patterns, powered by Cloudflare's network which spans 330+ cities in 120+ countries.
//!
//! # Features
//!
//! - Traffic volume time series by country
//! - Traffic anomaly detection
//! - HTTP request statistics
//!
//! # API Reference
//!
//! See: <https://developers.cloudflare.com/radar/>
//!
//! # License
//!
//! Data is available under CC BY-NC 4.0 license.
//!
//! # Privacy
//!
//! All data is aggregate traffic statistics. No individual users are tracked.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base URL for the Cloudflare Radar API.
const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4/radar";

/// Client for querying Cloudflare Radar's traffic data API.
#[derive(Clone)]
pub struct CloudflareRadarClient {
    client: reqwest::Client,
    base_url: String,
    api_token: Option<String>,
}

impl Default for CloudflareRadarClient {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CloudflareRadarClient {
    /// Create a new Cloudflare Radar client.
    ///
    /// # Arguments
    ///
    /// * `api_token` - Optional API token for authenticated requests.
    ///                 Some endpoints work without authentication but may have rate limits.
    pub fn new(api_token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: CLOUDFLARE_API_BASE.to_string(),
            api_token,
        }
    }

    /// Create a client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str, api_token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            api_token,
        }
    }

    /// Build a request with optional authentication.
    fn build_request(&self, url: &str) -> reqwest::RequestBuilder {
        let req = self.client.get(url);
        if let Some(token) = &self.api_token {
            req.header("Authorization", format!("Bearer {}", token))
        } else {
            req
        }
    }

    /// Get network traffic time series for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-2 country code (e.g., "US", "DE")
    /// * `date_range` - Time range (e.g., "1d", "7d", "14d", "28d")
    /// * `agg_interval` - Aggregation interval (e.g., "15m", "1h", "1d")
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = CloudflareRadarClient::new(Some("your-api-token".to_string()));
    /// let traffic = client.get_traffic_timeseries("US", "7d", Some("1h")).await?;
    /// ```
    pub async fn get_traffic_timeseries(
        &self,
        country_code: &str,
        date_range: &str,
        agg_interval: Option<&str>,
    ) -> anyhow::Result<CloudflareTimeseriesResponse> {
        let mut url = format!(
            "{}/netflows/timeseries?location={}&dateRange={}&format=json",
            self.base_url,
            country_code.to_uppercase(),
            date_range
        );

        if let Some(interval) = agg_interval {
            url.push_str(&format!("&aggInterval={}", interval));
        }

        let response = self.build_request(&url).send().await?;
        let data = response.json::<CloudflareTimeseriesResponse>().await?;
        Ok(data)
    }

    /// Get HTTP request time series for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-2 country code
    /// * `date_range` - Time range (e.g., "1d", "7d")
    pub async fn get_http_timeseries(
        &self,
        country_code: &str,
        date_range: &str,
    ) -> anyhow::Result<CloudflareTimeseriesResponse> {
        let url = format!(
            "{}/http/timeseries?location={}&dateRange={}&format=json",
            self.base_url,
            country_code.to_uppercase(),
            date_range
        );

        let response = self.build_request(&url).send().await?;
        let data = response.json::<CloudflareTimeseriesResponse>().await?;
        Ok(data)
    }

    /// Compare traffic between multiple countries.
    ///
    /// # Arguments
    ///
    /// * `country_codes` - List of ISO 3166-1 alpha-2 country codes
    /// * `date_range` - Time range
    pub async fn compare_countries(
        &self,
        country_codes: &[&str],
        date_range: &str,
    ) -> anyhow::Result<CloudflareTimeseriesResponse> {
        // Build URL with multiple location params
        let locations: Vec<String> = country_codes
            .iter()
            .map(|code| {
                format!(
                    "name={}_data&location={}&dateRange={}",
                    code.to_lowercase(),
                    code.to_uppercase(),
                    date_range
                )
            })
            .collect();

        let url = format!(
            "{}/netflows/timeseries?{}&format=json",
            self.base_url,
            locations.join("&")
        );

        let response = self.build_request(&url).send().await?;
        let data = response.json::<CloudflareTimeseriesResponse>().await?;
        Ok(data)
    }

    /// Get traffic anomalies for a location.
    ///
    /// # Arguments
    ///
    /// * `country_code` - Optional country code; if None, returns global anomalies
    /// * `date_range` - Time range (e.g., "7d", "14d")
    pub async fn get_traffic_anomalies(
        &self,
        country_code: Option<&str>,
        date_range: &str,
    ) -> anyhow::Result<CloudflareAnomaliesResponse> {
        let mut url = format!(
            "{}/traffic_anomalies?dateRange={}&format=json",
            self.base_url, date_range
        );

        if let Some(code) = country_code {
            url.push_str(&format!("&location={}", code.to_uppercase()));
        }

        let response = self.build_request(&url).send().await?;
        let data = response.json::<CloudflareAnomaliesResponse>().await?;
        Ok(data)
    }

    /// Get the current traffic summary for a country.
    ///
    /// Returns the most recent traffic data point.
    pub async fn get_current_traffic(
        &self,
        country_code: &str,
    ) -> anyhow::Result<Option<CloudflareDataPoint>> {
        let response = self.get_traffic_timeseries(country_code, "1d", Some("15m")).await?;

        Ok(response
            .result
            .and_then(|r| r.series.into_iter().next())
            .and_then(|s| {
                let timestamps = s.timestamps;
                let values = s.values;
                timestamps
                    .into_iter()
                    .zip(values.into_iter())
                    .last()
                    .map(|(ts, val)| CloudflareDataPoint {
                        timestamp: ts,
                        value: val,
                    })
            }))
    }

    /// Convenience method: get last 24 hours of traffic for a country.
    pub async fn get_daily_traffic(
        &self,
        country_code: &str,
    ) -> anyhow::Result<CloudflareTimeseriesResponse> {
        self.get_traffic_timeseries(country_code, "1d", Some("1h"))
            .await
    }

    /// Convenience method: get last 7 days of traffic for a country.
    pub async fn get_weekly_traffic(
        &self,
        country_code: &str,
    ) -> anyhow::Result<CloudflareTimeseriesResponse> {
        self.get_traffic_timeseries(country_code, "7d", Some("1h"))
            .await
    }
}

// ============================================================================
// Response types
// ============================================================================

/// Response wrapper for Cloudflare API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareTimeseriesResponse {
    /// Whether the request was successful.
    #[serde(default)]
    pub success: bool,

    /// Error messages if any.
    #[serde(default)]
    pub errors: Vec<CloudflareError>,

    /// The actual data.
    pub result: Option<CloudflareTimeseriesResult>,
}

/// Cloudflare API error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareError {
    /// Error code.
    #[serde(default)]
    pub code: i32,

    /// Error message.
    #[serde(default)]
    pub message: String,
}

/// Timeseries result data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareTimeseriesResult {
    /// Time series data.
    #[serde(default)]
    pub series: Vec<CloudflareSeries>,

    /// Metadata about the query.
    #[serde(default)]
    pub meta: CloudflareMeta,
}

/// A single time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareSeries {
    /// Name/label for this series (e.g., "us_data").
    #[serde(default)]
    pub name: String,

    /// ISO 8601 timestamps.
    #[serde(default)]
    pub timestamps: Vec<String>,

    /// Corresponding values (normalized, 0-1 range typically).
    #[serde(default)]
    pub values: Vec<f64>,
}

impl CloudflareSeries {
    /// Get the latest value.
    pub fn latest_value(&self) -> Option<f64> {
        self.values.last().copied()
    }

    /// Get the latest timestamp as DateTime.
    pub fn latest_timestamp(&self) -> Option<DateTime<Utc>> {
        self.timestamps
            .last()
            .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Calculate the average value.
    pub fn average(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }

    /// Calculate the minimum value.
    pub fn min(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::min)
    }

    /// Calculate the maximum value.
    pub fn max(&self) -> Option<f64> {
        self.values.iter().copied().reduce(f64::max)
    }

    /// Detect if there's a significant drop in the recent values.
    ///
    /// Returns true if the latest value is below `threshold` fraction of the average.
    pub fn has_significant_drop(&self, threshold: f64) -> bool {
        if let (Some(latest), avg) = (self.latest_value(), self.average()) {
            if avg > 0.0 {
                return latest < avg * threshold;
            }
        }
        false
    }
}

/// Metadata about a Cloudflare query.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudflareMeta {
    /// Date range queried.
    #[serde(default, rename = "dateRange")]
    pub date_range: Vec<CloudflareDateRange>,

    /// Aggregation interval used.
    #[serde(default, rename = "aggInterval")]
    pub agg_interval: String,

    /// Normalization info.
    #[serde(default, rename = "normalization")]
    pub normalization: String,
}

/// Date range specification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudflareDateRange {
    /// Start time.
    #[serde(default, rename = "startTime")]
    pub start_time: String,

    /// End time.
    #[serde(default, rename = "endTime")]
    pub end_time: String,
}

/// Response for traffic anomalies endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAnomaliesResponse {
    /// Whether the request was successful.
    #[serde(default)]
    pub success: bool,

    /// Error messages if any.
    #[serde(default)]
    pub errors: Vec<CloudflareError>,

    /// The actual data.
    pub result: Option<CloudflareAnomaliesResult>,
}

/// Traffic anomalies result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAnomaliesResult {
    /// List of detected anomalies.
    #[serde(default)]
    pub anomalies: Vec<CloudflareAnomaly>,
}

/// A detected traffic anomaly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAnomaly {
    /// Anomaly ID.
    #[serde(default)]
    pub id: String,

    /// Location code (country or ASN).
    #[serde(default)]
    pub location: String,

    /// Location name.
    #[serde(default, rename = "locationName")]
    pub location_name: String,

    /// Anomaly type (e.g., "OUTAGE", "UNUSUAL_TRAFFIC").
    #[serde(default, rename = "type")]
    pub anomaly_type: String,

    /// Start time of the anomaly.
    #[serde(default, rename = "startTime")]
    pub start_time: String,

    /// End time of the anomaly (empty if ongoing).
    #[serde(default, rename = "endTime")]
    pub end_time: String,

    /// Whether the anomaly has been verified by Cloudflare.
    #[serde(default)]
    pub verified: bool,

    /// Description of the anomaly.
    #[serde(default)]
    pub description: String,
}

impl CloudflareAnomaly {
    /// Check if the anomaly is currently ongoing.
    pub fn is_ongoing(&self) -> bool {
        self.end_time.is_empty()
    }

    /// Get start time as DateTime.
    pub fn start_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.start_time)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Get end time as DateTime (if ended).
    pub fn end_datetime(&self) -> Option<DateTime<Utc>> {
        if self.end_time.is_empty() {
            None
        } else {
            DateTime::parse_from_rfc3339(&self.end_time)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        }
    }
}

/// A single data point from the time series.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareDataPoint {
    /// ISO 8601 timestamp.
    pub timestamp: String,

    /// Traffic value.
    pub value: f64,
}

impl CloudflareDataPoint {
    /// Get timestamp as DateTime.
    pub fn datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.timestamp)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_statistics() {
        let series = CloudflareSeries {
            name: "test".to_string(),
            timestamps: vec![
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T01:00:00Z".to_string(),
                "2024-01-01T02:00:00Z".to_string(),
            ],
            values: vec![0.8, 1.0, 0.6],
        };

        assert_eq!(series.latest_value(), Some(0.6));
        assert!((series.average() - 0.8).abs() < 0.01);
        assert_eq!(series.min(), Some(0.6));
        assert_eq!(series.max(), Some(1.0));
    }

    #[test]
    fn test_significant_drop_detection() {
        let series = CloudflareSeries {
            name: "test".to_string(),
            timestamps: vec![
                "2024-01-01T00:00:00Z".to_string(),
                "2024-01-01T01:00:00Z".to_string(),
                "2024-01-01T02:00:00Z".to_string(),
                "2024-01-01T03:00:00Z".to_string(),
            ],
            values: vec![1.0, 1.0, 1.0, 0.2], // Sudden drop to 20%
        };

        // Average is 0.8, latest is 0.2
        // 0.2 < 0.8 * 0.5 = 0.4, so should detect drop
        assert!(series.has_significant_drop(0.5));

        // 0.2 < 0.8 * 0.2 = 0.16 is false
        assert!(!series.has_significant_drop(0.2));
    }

    #[test]
    fn test_anomaly_ongoing() {
        let ongoing = CloudflareAnomaly {
            id: "1".to_string(),
            location: "US".to_string(),
            location_name: "United States".to_string(),
            anomaly_type: "OUTAGE".to_string(),
            start_time: "2024-01-01T00:00:00Z".to_string(),
            end_time: "".to_string(),
            verified: true,
            description: "Traffic anomaly".to_string(),
        };

        let ended = CloudflareAnomaly {
            end_time: "2024-01-01T02:00:00Z".to_string(),
            ..ongoing.clone()
        };

        assert!(ongoing.is_ongoing());
        assert!(!ended.is_ongoing());
    }
}
