//! Dashboard for aggregating issues from all data sources.
//!
//! This module provides a unified view of problems detected across all data sources:
//! - Internet outages (IODA, Cloudflare)
//! - Humanitarian crises (HDX HAPI, ReliefWeb)
//! - Conflict events (ACLED)
//!
//! # Usage
//!
//! ```ignore
//! let dashboard = Dashboard::new(config);
//! let issues = dashboard.get_all_issues().await?;
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::data_sources::{
    AcledClient, CloudflareRadarClient, HdxHapiClient, IodaClient, ReliefWebClient,
};

/// Dashboard configuration.
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    /// ACLED email (required for ACLED API).
    pub acled_email: Option<String>,

    /// ACLED API key (required for ACLED API).
    pub acled_key: Option<String>,

    /// Cloudflare API token (optional, for higher rate limits).
    pub cloudflare_token: Option<String>,

    /// Application identifier for HDX/ReliefWeb.
    pub app_identifier: String,

    /// Countries to monitor (ISO 3166-1 alpha-2 for IODA/Cloudflare, alpha-3 for others).
    pub monitored_countries: Vec<MonitoredCountry>,

    /// Hours to look back for recent issues.
    pub lookback_hours: u32,
}

/// A country to monitor with both code formats.
#[derive(Debug, Clone)]
pub struct MonitoredCountry {
    /// ISO 3166-1 alpha-2 code (e.g., "UA" for Ukraine).
    pub alpha2: String,

    /// ISO 3166-1 alpha-3 code (e.g., "UKR" for Ukraine).
    pub alpha3: String,

    /// Country name for display and ACLED queries.
    pub name: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            acled_email: None,
            acled_key: None,
            cloudflare_token: None,
            app_identifier: "infrared".to_string(),
            monitored_countries: vec![],
            lookback_hours: 24,
        }
    }
}

/// Issue severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// Informational - worth noting but not urgent.
    Info,
    /// Warning - potential problem developing.
    Warning,
    /// Critical - serious ongoing issue.
    Critical,
    /// Emergency - requires immediate attention.
    Emergency,
}

impl IssueSeverity {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            IssueSeverity::Info => "Info",
            IssueSeverity::Warning => "Warning",
            IssueSeverity::Critical => "Critical",
            IssueSeverity::Emergency => "Emergency",
        }
    }
}

/// The source of an issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSource {
    /// IODA Internet outage detection.
    Ioda,
    /// Cloudflare Radar traffic anomalies.
    CloudflareRadar,
    /// HDX HAPI humanitarian data.
    HdxHapi,
    /// ACLED conflict events.
    Acled,
    /// ReliefWeb disasters and reports.
    ReliefWeb,
}

impl IssueSource {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            IssueSource::Ioda => "IODA",
            IssueSource::CloudflareRadar => "Cloudflare Radar",
            IssueSource::HdxHapi => "HDX HAPI",
            IssueSource::Acled => "ACLED",
            IssueSource::ReliefWeb => "ReliefWeb",
        }
    }
}

/// Category of issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// Internet connectivity issues.
    InternetOutage,
    /// Traffic anomalies.
    TrafficAnomaly,
    /// Armed conflict events.
    Conflict,
    /// Food security crisis.
    FoodSecurity,
    /// Displaced populations.
    Displacement,
    /// Natural or man-made disaster.
    Disaster,
    /// Humanitarian emergency.
    HumanitarianEmergency,
}

impl IssueCategory {
    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            IssueCategory::InternetOutage => "Internet Outage",
            IssueCategory::TrafficAnomaly => "Traffic Anomaly",
            IssueCategory::Conflict => "Conflict",
            IssueCategory::FoodSecurity => "Food Security",
            IssueCategory::Displacement => "Displacement",
            IssueCategory::Disaster => "Disaster",
            IssueCategory::HumanitarianEmergency => "Humanitarian Emergency",
        }
    }
}

/// A single issue detected from any data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique issue identifier (source:type:location:timestamp).
    pub id: String,

    /// Source system that detected this issue.
    pub source: IssueSource,

    /// Category of the issue.
    pub category: IssueCategory,

    /// Severity level.
    pub severity: IssueSeverity,

    /// Country or region affected.
    pub location: String,

    /// ISO country code (alpha-2 or alpha-3 depending on source).
    pub location_code: String,

    /// Short title/summary.
    pub title: String,

    /// Detailed description.
    pub description: String,

    /// When the issue was detected or started.
    pub timestamp: DateTime<Utc>,

    /// When the issue ended (if applicable).
    pub end_timestamp: Option<DateTime<Utc>>,

    /// Whether the issue is ongoing.
    pub is_ongoing: bool,

    /// Numeric impact value (interpretation depends on category).
    pub impact_value: Option<f64>,

    /// Human-readable impact description.
    pub impact_label: Option<String>,

    /// URL for more information.
    pub url: Option<String>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl Issue {
    /// Create a new issue with basic fields.
    pub fn new(
        source: IssueSource,
        category: IssueCategory,
        severity: IssueSeverity,
        location: &str,
        location_code: &str,
        title: &str,
        description: &str,
        timestamp: DateTime<Utc>,
    ) -> Self {
        let id = format!(
            "{}:{}:{}:{}",
            source.label().to_lowercase().replace(' ', "_"),
            category.label().to_lowercase().replace(' ', "_"),
            location_code.to_lowercase(),
            timestamp.timestamp()
        );

        Self {
            id,
            source,
            category,
            severity,
            location: location.to_string(),
            location_code: location_code.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            timestamp,
            end_timestamp: None,
            is_ongoing: true,
            impact_value: None,
            impact_label: None,
            url: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the end timestamp and mark as not ongoing.
    pub fn with_end(mut self, end: DateTime<Utc>) -> Self {
        self.end_timestamp = Some(end);
        self.is_ongoing = false;
        self
    }

    /// Set the impact value and label.
    pub fn with_impact(mut self, value: f64, label: &str) -> Self {
        self.impact_value = Some(value);
        self.impact_label = Some(label.to_string());
        self
    }

    /// Set the URL.
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Dashboard for aggregating issues from all sources.
#[derive(Clone)]
pub struct Dashboard {
    config: Arc<DashboardConfig>,
    ioda: IodaClient,
    cloudflare: CloudflareRadarClient,
    hdx_hapi: HdxHapiClient,
    reliefweb: ReliefWebClient,
    acled: Option<AcledClient>,
}

impl Dashboard {
    /// Create a new dashboard with the given configuration.
    pub fn new(config: DashboardConfig) -> Self {
        let acled = match (&config.acled_email, &config.acled_key) {
            (Some(email), Some(key)) => Some(AcledClient::new(email, key)),
            _ => None,
        };

        Self {
            ioda: IodaClient::new(),
            cloudflare: CloudflareRadarClient::new(config.cloudflare_token.clone()),
            hdx_hapi: HdxHapiClient::new(&config.app_identifier),
            reliefweb: ReliefWebClient::new(&config.app_identifier),
            acled,
            config: Arc::new(config),
        }
    }

    /// Get all issues from all data sources.
    pub async fn get_all_issues(&self) -> anyhow::Result<DashboardResponse> {
        let mut all_issues = Vec::new();
        let mut errors = Vec::new();

        // Fetch from all sources concurrently
        let (ioda_result, cloudflare_result, hdx_result, reliefweb_result, acled_result) = tokio::join!(
            self.fetch_ioda_issues(),
            self.fetch_cloudflare_issues(),
            self.fetch_hdx_issues(),
            self.fetch_reliefweb_issues(),
            self.fetch_acled_issues(),
        );

        // Collect results
        match ioda_result {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => errors.push(SourceError {
                source: IssueSource::Ioda,
                message: e.to_string(),
            }),
        }

        match cloudflare_result {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => errors.push(SourceError {
                source: IssueSource::CloudflareRadar,
                message: e.to_string(),
            }),
        }

        match hdx_result {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => errors.push(SourceError {
                source: IssueSource::HdxHapi,
                message: e.to_string(),
            }),
        }

        match reliefweb_result {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => errors.push(SourceError {
                source: IssueSource::ReliefWeb,
                message: e.to_string(),
            }),
        }

        match acled_result {
            Ok(issues) => all_issues.extend(issues),
            Err(e) => errors.push(SourceError {
                source: IssueSource::Acled,
                message: e.to_string(),
            }),
        }

        // Sort by severity (highest first) then by timestamp (newest first)
        all_issues.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.timestamp.cmp(&a.timestamp))
        });

        // Compute summary
        let summary = DashboardSummary::from_issues(&all_issues);

        Ok(DashboardResponse {
            timestamp: Utc::now(),
            summary,
            issues: all_issues,
            errors,
        })
    }

    /// Get issues filtered by source.
    pub async fn get_issues_by_source(&self, source: IssueSource) -> anyhow::Result<Vec<Issue>> {
        match source {
            IssueSource::Ioda => self.fetch_ioda_issues().await,
            IssueSource::CloudflareRadar => self.fetch_cloudflare_issues().await,
            IssueSource::HdxHapi => self.fetch_hdx_issues().await,
            IssueSource::Acled => self.fetch_acled_issues().await,
            IssueSource::ReliefWeb => self.fetch_reliefweb_issues().await,
        }
    }

    /// Get issues filtered by country code.
    pub async fn get_issues_by_country(&self, country_code: &str) -> anyhow::Result<Vec<Issue>> {
        let all = self.get_all_issues().await?;
        Ok(all
            .issues
            .into_iter()
            .filter(|i| {
                i.location_code.eq_ignore_ascii_case(country_code)
                    || i.location.to_lowercase().contains(&country_code.to_lowercase())
            })
            .collect())
    }

    /// Fetch issues from IODA.
    async fn fetch_ioda_issues(&self) -> anyhow::Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let alerts = self.ioda.get_recent_alerts(self.config.lookback_hours).await?;

        for alert in alerts.data {
            let severity = match alert.level.as_str() {
                "critical" => IssueSeverity::Critical,
                "warning" => IssueSeverity::Warning,
                _ => IssueSeverity::Info,
            };

            let drop_pct = alert.drop_percentage();
            let timestamp = alert.timestamp().unwrap_or_else(Utc::now);

            let issue = Issue::new(
                IssueSource::Ioda,
                IssueCategory::InternetOutage,
                severity,
                &alert.entity_name,
                &alert.entity_code,
                &format!("Internet outage detected in {}", alert.entity_name),
                &format!(
                    "{} connectivity dropped by {:.1}% (from {} to {}) detected by {}",
                    alert.entity_name, drop_pct, alert.history_value, alert.value, alert.datasource
                ),
                timestamp,
            )
            .with_impact(drop_pct, &format!("{:.1}% drop from baseline", drop_pct))
            .with_metadata("datasource", &alert.datasource)
            .with_metadata("condition", &alert.condition);

            issues.push(issue);
        }

        Ok(issues)
    }

    /// Fetch issues from Cloudflare Radar.
    async fn fetch_cloudflare_issues(&self) -> anyhow::Result<Vec<Issue>> {
        let mut issues = Vec::new();
        let anomalies = self.cloudflare.get_traffic_anomalies(None, "7d").await?;

        if let Some(result) = anomalies.result {
            for anomaly in result.anomalies {
                let severity = if anomaly.verified {
                    IssueSeverity::Critical
                } else {
                    IssueSeverity::Warning
                };

                let timestamp = anomaly.start_datetime().unwrap_or_else(Utc::now);

                let mut issue = Issue::new(
                    IssueSource::CloudflareRadar,
                    IssueCategory::TrafficAnomaly,
                    severity,
                    &anomaly.location_name,
                    &anomaly.location,
                    &format!("Traffic anomaly in {}", anomaly.location_name),
                    &anomaly.description,
                    timestamp,
                )
                .with_metadata("anomaly_type", &anomaly.anomaly_type)
                .with_metadata("verified", &anomaly.verified.to_string());

                if let Some(end) = anomaly.end_datetime() {
                    issue = issue.with_end(end);
                }

                issues.push(issue);
            }
        }

        Ok(issues)
    }

    /// Fetch issues from HDX HAPI.
    async fn fetch_hdx_issues(&self) -> anyhow::Result<Vec<Issue>> {
        let mut issues = Vec::new();

        // Check national risk for all available countries
        let risk_response = self.hdx_hapi.get_national_risk(None).await?;

        for risk in risk_response.data {
            if risk.is_very_high_risk() {
                let timestamp = risk
                    .reference_period_start
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                let issue = Issue::new(
                    IssueSource::HdxHapi,
                    IssueCategory::HumanitarianEmergency,
                    IssueSeverity::Emergency,
                    &risk.location_name,
                    &risk.location_code,
                    &format!("Very high humanitarian risk in {}", risk.location_name),
                    &format!(
                        "National risk score: {:.1}/10. Hazard exposure: {:.1}, Vulnerability: {:.1}, Coping capacity: {:.1}",
                        risk.overall_risk.unwrap_or(0.0),
                        risk.hazard_exposure.unwrap_or(0.0),
                        risk.vulnerability.unwrap_or(0.0),
                        risk.coping_capacity.unwrap_or(0.0)
                    ),
                    timestamp,
                )
                .with_impact(
                    risk.overall_risk.unwrap_or(0.0),
                    &format!("{:.1}/10 risk score", risk.overall_risk.unwrap_or(0.0)),
                );

                issues.push(issue);
            } else if risk.is_high_risk() {
                let timestamp = risk
                    .reference_period_start
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(Utc::now);

                let issue = Issue::new(
                    IssueSource::HdxHapi,
                    IssueCategory::HumanitarianEmergency,
                    IssueSeverity::Critical,
                    &risk.location_name,
                    &risk.location_code,
                    &format!("High humanitarian risk in {}", risk.location_name),
                    &format!(
                        "National risk score: {:.1}/10. Hazard exposure: {:.1}, Vulnerability: {:.1}, Coping capacity: {:.1}",
                        risk.overall_risk.unwrap_or(0.0),
                        risk.hazard_exposure.unwrap_or(0.0),
                        risk.vulnerability.unwrap_or(0.0),
                        risk.coping_capacity.unwrap_or(0.0)
                    ),
                    timestamp,
                )
                .with_impact(
                    risk.overall_risk.unwrap_or(0.0),
                    &format!("{:.1}/10 risk score", risk.overall_risk.unwrap_or(0.0)),
                );

                issues.push(issue);
            }
        }

        Ok(issues)
    }

    /// Fetch issues from ACLED.
    async fn fetch_acled_issues(&self) -> anyhow::Result<Vec<Issue>> {
        let acled = match &self.acled {
            Some(client) => client,
            None => return Ok(Vec::new()), // ACLED not configured
        };

        let mut issues = Vec::new();

        // Fetch recent events with fatalities for monitored countries
        for country in &self.config.monitored_countries {
            let response = acled
                .get_events_with_fatalities(&country.name, 1, Some(100))
                .await?;

            // Group by event type and summarize
            let total_fatalities = response.total_fatalities();
            let event_count = response.count;

            if total_fatalities > 0 {
                let severity = if total_fatalities >= 100 {
                    IssueSeverity::Emergency
                } else if total_fatalities >= 50 {
                    IssueSeverity::Critical
                } else if total_fatalities >= 10 {
                    IssueSeverity::Warning
                } else {
                    IssueSeverity::Info
                };

                let most_recent = response.most_recent();
                let timestamp = most_recent
                    .and_then(|e| e.datetime())
                    .unwrap_or_else(Utc::now);

                let issue = Issue::new(
                    IssueSource::Acled,
                    IssueCategory::Conflict,
                    severity,
                    &country.name,
                    &country.alpha3,
                    &format!("Conflict activity in {}", country.name),
                    &format!(
                        "{} conflict events with {} fatalities in the last {} hours",
                        event_count, total_fatalities, self.config.lookback_hours
                    ),
                    timestamp,
                )
                .with_impact(
                    total_fatalities as f64,
                    &format!("{} fatalities", total_fatalities),
                )
                .with_metadata("event_count", &event_count.to_string());

                issues.push(issue);
            }
        }

        Ok(issues)
    }

    /// Fetch issues from ReliefWeb.
    async fn fetch_reliefweb_issues(&self) -> anyhow::Result<Vec<Issue>> {
        let mut issues = Vec::new();

        // Get ongoing disasters
        let disasters = self.reliefweb.get_ongoing_disasters(Some(50)).await?;

        for item in disasters.data {
            let disaster = &item.fields;

            let severity = match disaster.type_name() {
                Some("Complex Emergency") => IssueSeverity::Emergency,
                Some("Conflict") => IssueSeverity::Critical,
                Some("Epidemic") => IssueSeverity::Critical,
                Some("Flood") | Some("Earthquake") | Some("Tsunami") => IssueSeverity::Critical,
                _ => IssueSeverity::Warning,
            };

            let timestamp = disaster
                .date
                .as_ref()
                .and_then(|d| d.created_datetime())
                .unwrap_or_else(Utc::now);

            let country_name = disaster.country_name().unwrap_or("Unknown");
            let country_code = disaster
                .primary_country
                .as_ref()
                .map(|c| c.iso3.as_str())
                .unwrap_or("");

            let issue = Issue::new(
                IssueSource::ReliefWeb,
                IssueCategory::Disaster,
                severity,
                country_name,
                country_code,
                &disaster.name,
                &disaster.description,
                timestamp,
            )
            .with_url(&disaster.url)
            .with_metadata("disaster_type", disaster.type_name().unwrap_or("Unknown"))
            .with_metadata("glide", &disaster.glide)
            .with_metadata("status", &disaster.status);

            issues.push(issue);
        }

        Ok(issues)
    }
}

/// Dashboard API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardResponse {
    /// When this response was generated.
    pub timestamp: DateTime<Utc>,

    /// Summary statistics.
    pub summary: DashboardSummary,

    /// All issues, sorted by severity and timestamp.
    pub issues: Vec<Issue>,

    /// Errors encountered while fetching from sources.
    #[serde(default)]
    pub errors: Vec<SourceError>,
}

/// Summary statistics for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    /// Total number of issues.
    pub total_issues: usize,

    /// Number of emergency-level issues.
    pub emergency_count: usize,

    /// Number of critical-level issues.
    pub critical_count: usize,

    /// Number of warning-level issues.
    pub warning_count: usize,

    /// Number of info-level issues.
    pub info_count: usize,

    /// Issues by source.
    pub by_source: std::collections::HashMap<String, usize>,

    /// Issues by category.
    pub by_category: std::collections::HashMap<String, usize>,

    /// Countries with most issues.
    pub top_countries: Vec<CountryIssueCount>,
}

impl DashboardSummary {
    /// Compute summary from a list of issues.
    pub fn from_issues(issues: &[Issue]) -> Self {
        let mut by_source: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut by_category: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut by_country: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        let mut emergency_count = 0;
        let mut critical_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;

        for issue in issues {
            match issue.severity {
                IssueSeverity::Emergency => emergency_count += 1,
                IssueSeverity::Critical => critical_count += 1,
                IssueSeverity::Warning => warning_count += 1,
                IssueSeverity::Info => info_count += 1,
            }

            *by_source.entry(issue.source.label().to_string()).or_insert(0) += 1;
            *by_category
                .entry(issue.category.label().to_string())
                .or_insert(0) += 1;
            *by_country.entry(issue.location.clone()).or_insert(0) += 1;
        }

        // Get top 10 countries by issue count
        let mut country_counts: Vec<_> = by_country.into_iter().collect();
        country_counts.sort_by(|a, b| b.1.cmp(&a.1));
        let top_countries: Vec<CountryIssueCount> = country_counts
            .into_iter()
            .take(10)
            .map(|(country, count)| CountryIssueCount { country, count })
            .collect();

        Self {
            total_issues: issues.len(),
            emergency_count,
            critical_count,
            warning_count,
            info_count,
            by_source,
            by_category,
            top_countries,
        }
    }
}

/// Country with issue count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryIssueCount {
    pub country: String,
    pub count: usize,
}

/// Error from a data source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceError {
    pub source: IssueSource,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_creation() {
        let issue = Issue::new(
            IssueSource::Ioda,
            IssueCategory::InternetOutage,
            IssueSeverity::Critical,
            "Ukraine",
            "UA",
            "Internet outage in Ukraine",
            "BGP connectivity dropped by 50%",
            Utc::now(),
        );

        assert!(issue.id.starts_with("ioda:internet_outage:ua:"));
        assert!(issue.is_ongoing);
        assert_eq!(issue.severity, IssueSeverity::Critical);
    }

    #[test]
    fn test_issue_with_end() {
        let start = Utc::now();
        let end = start + chrono::Duration::hours(2);

        let issue = Issue::new(
            IssueSource::Ioda,
            IssueCategory::InternetOutage,
            IssueSeverity::Warning,
            "Germany",
            "DE",
            "Test",
            "Test",
            start,
        )
        .with_end(end);

        assert!(!issue.is_ongoing);
        assert_eq!(issue.end_timestamp, Some(end));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(IssueSeverity::Emergency > IssueSeverity::Critical);
        assert!(IssueSeverity::Critical > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn test_summary_from_issues() {
        let issues = vec![
            Issue::new(
                IssueSource::Ioda,
                IssueCategory::InternetOutage,
                IssueSeverity::Emergency,
                "Ukraine",
                "UA",
                "Test",
                "Test",
                Utc::now(),
            ),
            Issue::new(
                IssueSource::Acled,
                IssueCategory::Conflict,
                IssueSeverity::Critical,
                "Ukraine",
                "UA",
                "Test",
                "Test",
                Utc::now(),
            ),
            Issue::new(
                IssueSource::ReliefWeb,
                IssueCategory::Disaster,
                IssueSeverity::Warning,
                "Syria",
                "SY",
                "Test",
                "Test",
                Utc::now(),
            ),
        ];

        let summary = DashboardSummary::from_issues(&issues);

        assert_eq!(summary.total_issues, 3);
        assert_eq!(summary.emergency_count, 1);
        assert_eq!(summary.critical_count, 1);
        assert_eq!(summary.warning_count, 1);
        assert_eq!(summary.info_count, 0);
        assert_eq!(summary.by_source.get("IODA"), Some(&1));
        assert_eq!(summary.by_source.get("ACLED"), Some(&1));
        assert_eq!(summary.top_countries.len(), 2);
    }
}
