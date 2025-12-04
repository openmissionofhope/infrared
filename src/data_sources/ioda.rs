//! IODA (Internet Outage Detection and Analysis) client.
//!
//! IODA monitors the Internet in near real-time to identify macroscopic Internet
//! outages affecting the edge of the network on a country, regional, or ASN level.
//!
//! # Data Sources Used by IODA
//!
//! - **BGP**: Routing data from RouteViews and RIPE RIS (~500 monitors)
//! - **Active Probing**: Continuous ping monitoring for normal vs abnormal signals
//! - **Darknet/Telescope**: Unsolicited traffic from UCSD Network Telescope
//!
//! # API Reference
//!
//! See: <https://github.com/CAIDA/ioda-api/wiki/API-Specification>
//!
//! # Privacy
//!
//! All data is aggregate network-level statistics. No individual users are tracked.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base URL for the IODA API.
const IODA_API_BASE: &str = "https://api.ioda.inetintel.cc.gatech.edu/v2";

/// Client for querying IODA's Internet outage detection API.
#[derive(Clone)]
pub struct IodaClient {
    client: reqwest::Client,
    base_url: String,
}

impl Default for IodaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl IodaClient {
    /// Create a new IODA client with default settings.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: IODA_API_BASE.to_string(),
        }
    }

    /// Create a new IODA client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    /// Fetch outage alerts for a specific country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-2 country code (e.g., "US", "DE", "JP")
    /// * `from` - Start of time range (Unix timestamp)
    /// * `until` - End of time range (Unix timestamp)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = IodaClient::new();
    /// let now = Utc::now().timestamp();
    /// let one_day_ago = now - 86400;
    /// let alerts = client.get_country_alerts("US", one_day_ago, now).await?;
    /// ```
    pub async fn get_country_alerts(
        &self,
        country_code: &str,
        from: i64,
        until: i64,
    ) -> anyhow::Result<IodaAlertsResponse> {
        let url = format!(
            "{}/outages/alerts/country/{}?from={}&until={}",
            self.base_url,
            country_code.to_uppercase(),
            from,
            until
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<IodaAlertsResponse>().await?;
        Ok(data)
    }

    /// Fetch outage alerts for all countries in a time range.
    ///
    /// # Arguments
    ///
    /// * `from` - Start of time range (Unix timestamp)
    /// * `until` - End of time range (Unix timestamp)
    pub async fn get_all_country_alerts(
        &self,
        from: i64,
        until: i64,
    ) -> anyhow::Result<IodaAlertsResponse> {
        let url = format!(
            "{}/outages/alerts/country?from={}&until={}",
            self.base_url, from, until
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<IodaAlertsResponse>().await?;
        Ok(data)
    }

    /// Fetch outage events (aggregated alerts) for countries.
    ///
    /// Events are aggregated from multiple alerts and include severity scores.
    ///
    /// # Arguments
    ///
    /// * `from` - Start of time range (Unix timestamp)
    /// * `until` - End of time range (Unix timestamp)
    pub async fn get_country_events(
        &self,
        from: i64,
        until: i64,
    ) -> anyhow::Result<IodaEventsResponse> {
        let url = format!(
            "{}/outages/events/country?from={}&until={}&format=codf",
            self.base_url, from, until
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<IodaEventsResponse>().await?;
        Ok(data)
    }

    /// Fetch raw signal time series for a country.
    ///
    /// Returns normalized connectivity scores from BGP, active probing, and darknet.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-2 country code
    /// * `from` - Start of time range (Unix timestamp)
    /// * `until` - End of time range (Unix timestamp)
    pub async fn get_country_signals(
        &self,
        country_code: &str,
        from: i64,
        until: i64,
    ) -> anyhow::Result<IodaSignalsResponse> {
        let url = format!(
            "{}/signals/raw/country/{}?from={}&until={}",
            self.base_url,
            country_code.to_uppercase(),
            from,
            until
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<IodaSignalsResponse>().await?;
        Ok(data)
    }

    /// Get a summary of outage scores for all countries.
    ///
    /// Returns overall scores plus per-datasource breakdowns.
    ///
    /// # Arguments
    ///
    /// * `from` - Start of time range (Unix timestamp)
    /// * `until` - End of time range (Unix timestamp)
    pub async fn get_country_summary(
        &self,
        from: i64,
        until: i64,
    ) -> anyhow::Result<IodaSummaryResponse> {
        let url = format!(
            "{}/outages/summary/country?from={}&until={}",
            self.base_url, from, until
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<IodaSummaryResponse>().await?;
        Ok(data)
    }

    /// Convenience method: get alerts from the last N hours for all countries.
    pub async fn get_recent_alerts(&self, hours: u32) -> anyhow::Result<IodaAlertsResponse> {
        let now = Utc::now().timestamp();
        let from = now - (hours as i64 * 3600);
        self.get_all_country_alerts(from, now).await
    }

    /// Convenience method: get alerts from the last N hours for a specific country.
    pub async fn get_recent_country_alerts(
        &self,
        country_code: &str,
        hours: u32,
    ) -> anyhow::Result<IodaAlertsResponse> {
        let now = Utc::now().timestamp();
        let from = now - (hours as i64 * 3600);
        self.get_country_alerts(country_code, from, now).await
    }
}

// ============================================================================
// Response types
// ============================================================================

/// Response from the IODA alerts endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaAlertsResponse {
    /// List of outage alerts.
    #[serde(default)]
    pub data: Vec<IodaAlert>,
}

/// A single outage alert from IODA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaAlert {
    /// Data source that detected the alert (e.g., "bgp", "ping-slash24", "ucsd-nt").
    #[serde(default)]
    pub datasource: String,

    /// Entity type (e.g., "country", "asn", "region").
    #[serde(default, rename = "entityType")]
    pub entity_type: String,

    /// Entity code (e.g., "US", "DE" for countries).
    #[serde(default, rename = "entityCode")]
    pub entity_code: String,

    /// Human-readable entity name.
    #[serde(default, rename = "entityName")]
    pub entity_name: String,

    /// Unix timestamp when the alert was detected.
    #[serde(default)]
    pub time: i64,

    /// Alert severity level.
    #[serde(default)]
    pub level: String,

    /// Alert condition (e.g., "down", "normal").
    #[serde(default)]
    pub condition: String,

    /// Current value at time of alert.
    #[serde(default)]
    pub value: f64,

    /// Historical baseline value.
    #[serde(default, rename = "historyValue")]
    pub history_value: f64,
}

impl IodaAlert {
    /// Get the timestamp as a DateTime.
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.time, 0)
    }

    /// Calculate the drop percentage from historical baseline.
    pub fn drop_percentage(&self) -> f64 {
        if self.history_value > 0.0 {
            ((self.history_value - self.value) / self.history_value) * 100.0
        } else {
            0.0
        }
    }
}

/// Response from the IODA events endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaEventsResponse {
    /// List of outage events.
    #[serde(default)]
    pub data: Vec<IodaEvent>,
}

/// An outage event (aggregated from multiple alerts).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaEvent {
    /// Entity type.
    #[serde(default, rename = "entityType")]
    pub entity_type: String,

    /// Entity code.
    #[serde(default, rename = "entityCode")]
    pub entity_code: String,

    /// Entity name.
    #[serde(default, rename = "entityName")]
    pub entity_name: String,

    /// Event start time (Unix timestamp).
    #[serde(default)]
    pub from: i64,

    /// Event end time (Unix timestamp).
    #[serde(default)]
    pub until: i64,

    /// Overall severity score.
    #[serde(default)]
    pub score: f64,
}

impl IodaEvent {
    /// Get the event duration in seconds.
    pub fn duration_seconds(&self) -> i64 {
        self.until - self.from
    }

    /// Get start time as DateTime.
    pub fn start_time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.from, 0)
    }

    /// Get end time as DateTime.
    pub fn end_time(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.until, 0)
    }
}

/// Response from the IODA signals endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaSignalsResponse {
    /// Time series data per data source.
    #[serde(default)]
    pub data: Vec<IodaSignalSeries>,
}

/// Time series data from a single data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaSignalSeries {
    /// Data source name.
    #[serde(default)]
    pub datasource: String,

    /// Entity code.
    #[serde(default, rename = "entityCode")]
    pub entity_code: String,

    /// Time series values: each entry is [timestamp, value].
    #[serde(default)]
    pub values: Vec<Vec<f64>>,
}

impl IodaSignalSeries {
    /// Get the latest value from the time series.
    pub fn latest_value(&self) -> Option<f64> {
        self.values.last().and_then(|v| v.get(1).copied())
    }

    /// Get the latest timestamp from the time series.
    pub fn latest_timestamp(&self) -> Option<i64> {
        self.values.last().and_then(|v| v.first().map(|t| *t as i64))
    }
}

/// Response from the IODA summary endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaSummaryResponse {
    /// Summary data for each entity.
    #[serde(default)]
    pub data: Vec<IodaSummary>,
}

/// Summary of outage scores for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IodaSummary {
    /// Entity type.
    #[serde(default, rename = "entityType")]
    pub entity_type: String,

    /// Entity code.
    #[serde(default, rename = "entityCode")]
    pub entity_code: String,

    /// Entity name.
    #[serde(default, rename = "entityName")]
    pub entity_name: String,

    /// Overall outage score (higher = more severe).
    #[serde(default)]
    pub score: f64,

    /// Scores broken down by data source.
    #[serde(default)]
    pub scores: IodaScores,
}

/// Per-datasource outage scores.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IodaScores {
    /// Overall combined score.
    #[serde(default)]
    pub overall: f64,

    /// BGP routing score.
    #[serde(default)]
    pub bgp: f64,

    /// Active probing (ping) score.
    #[serde(default, rename = "ping-slash24")]
    pub ping_slash24: f64,

    /// Darknet/telescope score.
    #[serde(default, rename = "ucsd-nt")]
    pub ucsd_nt: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_drop_percentage() {
        let alert = IodaAlert {
            datasource: "bgp".to_string(),
            entity_type: "country".to_string(),
            entity_code: "US".to_string(),
            entity_name: "United States".to_string(),
            time: 1701500000,
            level: "critical".to_string(),
            condition: "down".to_string(),
            value: 20.0,
            history_value: 100.0,
        };

        assert!((alert.drop_percentage() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_event_duration() {
        let event = IodaEvent {
            entity_type: "country".to_string(),
            entity_code: "DE".to_string(),
            entity_name: "Germany".to_string(),
            from: 1701500000,
            until: 1701503600,
            score: 50.0,
        };

        assert_eq!(event.duration_seconds(), 3600);
    }
}
