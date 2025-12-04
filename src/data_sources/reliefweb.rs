//! ReliefWeb API client.
//!
//! ReliefWeb is OCHA's humanitarian information service, providing access to
//! reports, disasters, countries, jobs, and training from 4,000+ sources.
//!
//! # Features
//!
//! - Disaster information with GLIDE numbers
//! - Humanitarian reports and updates
//! - Country profiles and statistics
//! - Job listings and training opportunities
//!
//! # API Reference
//!
//! See: <https://apidoc.reliefweb.int/>
//!
//! # Rate Limits
//!
//! - Maximum 1,000 calls per day
//! - Maximum 1,000 entries per response
//!
//! # Privacy
//!
//! All data is publicly curated humanitarian information. No individual persons are tracked.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base URL for the ReliefWeb API.
const RELIEFWEB_API_BASE: &str = "https://api.reliefweb.int/v1";

/// Client for querying the ReliefWeb humanitarian data API.
#[derive(Clone)]
pub struct ReliefWebClient {
    client: reqwest::Client,
    base_url: String,
    app_name: String,
}

impl Default for ReliefWebClient {
    fn default() -> Self {
        Self::new("infrared")
    }
}

impl ReliefWebClient {
    /// Create a new ReliefWeb client.
    ///
    /// # Arguments
    ///
    /// * `app_name` - Application name for API identification (required as of Nov 2025).
    pub fn new(app_name: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: RELIEFWEB_API_BASE.to_string(),
            app_name: app_name.to_string(),
        }
    }

    /// Create a client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str, app_name: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            app_name: app_name.to_string(),
        }
    }

    /// Get disasters list, optionally filtered by country or status.
    ///
    /// # Arguments
    ///
    /// * `country` - Optional country name to filter by
    /// * `status` - Optional status filter ("ongoing", "past", "alert")
    /// * `limit` - Maximum number of results (default: 50, max: 1000)
    pub async fn get_disasters(
        &self,
        country: Option<&str>,
        status: Option<&str>,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebDisastersResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let mut url = format!(
            "{}/disasters?appname={}&limit={}&preset=latest",
            self.base_url, self.app_name, limit
        );

        if let Some(c) = country {
            url.push_str(&format!(
                "&filter[field]=country.name&filter[value]={}",
                urlencoding::encode(c)
            ));
        }
        if let Some(s) = status {
            url.push_str(&format!(
                "&filter[field]=status&filter[value]={}",
                urlencoding::encode(s)
            ));
        }

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebDisastersResponse>().await?;
        Ok(data)
    }

    /// Get a specific disaster by ID.
    pub async fn get_disaster(&self, id: u64) -> anyhow::Result<ReliefWebDisasterResponse> {
        let url = format!(
            "{}/disasters/{}?appname={}",
            self.base_url, id, self.app_name
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebDisasterResponse>().await?;
        Ok(data)
    }

    /// Get recent reports, optionally filtered.
    ///
    /// # Arguments
    ///
    /// * `country` - Optional country name to filter by
    /// * `disaster` - Optional disaster name to filter by
    /// * `limit` - Maximum number of results
    pub async fn get_reports(
        &self,
        country: Option<&str>,
        disaster: Option<&str>,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebReportsResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let mut url = format!(
            "{}/reports?appname={}&limit={}&preset=latest",
            self.base_url, self.app_name, limit
        );

        if let Some(c) = country {
            url.push_str(&format!(
                "&filter[field]=country.name&filter[value]={}",
                urlencoding::encode(c)
            ));
        }
        if let Some(d) = disaster {
            url.push_str(&format!(
                "&filter[field]=disaster.name&filter[value]={}",
                urlencoding::encode(d)
            ));
        }

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebReportsResponse>().await?;
        Ok(data)
    }

    /// Get a specific report by ID.
    pub async fn get_report(&self, id: u64) -> anyhow::Result<ReliefWebReportResponse> {
        let url = format!("{}/reports/{}?appname={}", self.base_url, id, self.app_name);

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebReportResponse>().await?;
        Ok(data)
    }

    /// Get country information.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    pub async fn get_countries(&self, limit: Option<u32>) -> anyhow::Result<ReliefWebCountriesResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let url = format!(
            "{}/countries?appname={}&limit={}",
            self.base_url, self.app_name, limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebCountriesResponse>().await?;
        Ok(data)
    }

    /// Get a specific country by ID or ISO code.
    pub async fn get_country(&self, id: &str) -> anyhow::Result<ReliefWebCountryResponse> {
        let url = format!(
            "{}/countries/{}?appname={}",
            self.base_url, id, self.app_name
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebCountryResponse>().await?;
        Ok(data)
    }

    /// Get humanitarian job listings.
    ///
    /// # Arguments
    ///
    /// * `country` - Optional country name to filter by
    /// * `limit` - Maximum number of results
    pub async fn get_jobs(
        &self,
        country: Option<&str>,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebJobsResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let mut url = format!(
            "{}/jobs?appname={}&limit={}&preset=latest",
            self.base_url, self.app_name, limit
        );

        if let Some(c) = country {
            url.push_str(&format!(
                "&filter[field]=country.name&filter[value]={}",
                urlencoding::encode(c)
            ));
        }

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebJobsResponse>().await?;
        Ok(data)
    }

    /// Get training opportunities.
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of results
    pub async fn get_training(&self, limit: Option<u32>) -> anyhow::Result<ReliefWebTrainingResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let url = format!(
            "{}/training?appname={}&limit={}&preset=latest",
            self.base_url, self.app_name, limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebTrainingResponse>().await?;
        Ok(data)
    }

    /// Get information sources.
    pub async fn get_sources(&self, limit: Option<u32>) -> anyhow::Result<ReliefWebSourcesResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let url = format!(
            "{}/sources?appname={}&limit={}",
            self.base_url, self.app_name, limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebSourcesResponse>().await?;
        Ok(data)
    }

    /// Search reports with a text query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query text
    /// * `limit` - Maximum number of results
    pub async fn search_reports(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebReportsResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let url = format!(
            "{}/reports?appname={}&limit={}&query[value]={}&query[operator]=AND",
            self.base_url,
            self.app_name,
            limit,
            urlencoding::encode(query)
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebReportsResponse>().await?;
        Ok(data)
    }

    /// Get ongoing disasters.
    pub async fn get_ongoing_disasters(
        &self,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebDisastersResponse> {
        self.get_disasters(None, Some("ongoing"), limit).await
    }

    /// Get disasters by type.
    ///
    /// # Arguments
    ///
    /// * `disaster_type` - Type of disaster (e.g., "Flood", "Earthquake", "Conflict")
    /// * `limit` - Maximum number of results
    pub async fn get_disasters_by_type(
        &self,
        disaster_type: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<ReliefWebDisastersResponse> {
        let limit = limit.unwrap_or(50).min(1000);
        let url = format!(
            "{}/disasters?appname={}&limit={}&filter[field]=type.name&filter[value]={}",
            self.base_url,
            self.app_name,
            limit,
            urlencoding::encode(disaster_type)
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<ReliefWebDisastersResponse>().await?;
        Ok(data)
    }
}

// ============================================================================
// Response types
// ============================================================================

/// Generic ReliefWeb list response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebListResponse<T> {
    /// Total number of matching items.
    #[serde(default, rename = "totalCount")]
    pub total_count: i64,

    /// Number of items returned.
    #[serde(default)]
    pub count: i64,

    /// List of data items.
    #[serde(default)]
    pub data: Vec<ReliefWebItem<T>>,
}

/// A single item wrapper in ReliefWeb responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebItem<T> {
    /// Item ID.
    #[serde(default)]
    pub id: String,

    /// Item score (relevance).
    #[serde(default)]
    pub score: f64,

    /// Item fields.
    #[serde(default)]
    pub fields: T,

    /// Direct URL to the item.
    #[serde(default)]
    pub href: String,
}

/// Generic single item response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebSingleResponse<T> {
    /// The data item.
    #[serde(default)]
    pub data: Vec<ReliefWebItem<T>>,
}

// Disaster types

/// Disasters list response.
pub type ReliefWebDisastersResponse = ReliefWebListResponse<ReliefWebDisasterFields>;

/// Single disaster response.
pub type ReliefWebDisasterResponse = ReliefWebSingleResponse<ReliefWebDisasterFields>;

/// Disaster record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebDisasterFields {
    /// Disaster name.
    #[serde(default)]
    pub name: String,

    /// Disaster description.
    #[serde(default)]
    pub description: String,

    /// GLIDE number (Global IDentifier).
    #[serde(default)]
    pub glide: String,

    /// Disaster status.
    #[serde(default)]
    pub status: String,

    /// Primary disaster type.
    #[serde(default, rename = "type")]
    pub disaster_type: Vec<ReliefWebTerm>,

    /// Primary affected country.
    #[serde(default)]
    pub primary_country: Option<ReliefWebCountryRef>,

    /// All affected countries.
    #[serde(default)]
    pub country: Vec<ReliefWebCountryRef>,

    /// Date the disaster occurred.
    #[serde(default)]
    pub date: Option<ReliefWebDate>,

    /// Date created in ReliefWeb.
    #[serde(default, rename = "date.created")]
    pub date_created: Option<String>,

    /// URL to the disaster page.
    #[serde(default)]
    pub url: String,

    /// Current situation.
    #[serde(default)]
    pub current: String,
}

impl ReliefWebDisasterFields {
    /// Check if the disaster is ongoing.
    pub fn is_ongoing(&self) -> bool {
        self.status.to_lowercase() == "ongoing"
    }

    /// Get the primary type name.
    pub fn type_name(&self) -> Option<&str> {
        self.disaster_type.first().map(|t| t.name.as_str())
    }

    /// Get the primary country name.
    pub fn country_name(&self) -> Option<&str> {
        self.primary_country.as_ref().map(|c| c.name.as_str())
    }
}

// Report types

/// Reports list response.
pub type ReliefWebReportsResponse = ReliefWebListResponse<ReliefWebReportFields>;

/// Single report response.
pub type ReliefWebReportResponse = ReliefWebSingleResponse<ReliefWebReportFields>;

/// Report record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebReportFields {
    /// Report title.
    #[serde(default)]
    pub title: String,

    /// Report body/content.
    #[serde(default)]
    pub body: String,

    /// Report summary.
    #[serde(default, rename = "body-html")]
    pub body_html: String,

    /// Report status.
    #[serde(default)]
    pub status: String,

    /// Report format (e.g., "Situation Report", "News").
    #[serde(default)]
    pub format: Vec<ReliefWebTerm>,

    /// Report source organization.
    #[serde(default)]
    pub source: Vec<ReliefWebSource>,

    /// Primary affected country.
    #[serde(default)]
    pub primary_country: Option<ReliefWebCountryRef>,

    /// All mentioned countries.
    #[serde(default)]
    pub country: Vec<ReliefWebCountryRef>,

    /// Associated disasters.
    #[serde(default)]
    pub disaster: Vec<ReliefWebDisasterRef>,

    /// Report themes.
    #[serde(default)]
    pub theme: Vec<ReliefWebTerm>,

    /// OCHA product type.
    #[serde(default)]
    pub ocha_product: Vec<ReliefWebTerm>,

    /// Report language.
    #[serde(default)]
    pub language: Vec<ReliefWebTerm>,

    /// Publication date.
    #[serde(default)]
    pub date: Option<ReliefWebDate>,

    /// URL to the report.
    #[serde(default)]
    pub url: String,

    /// URL alias.
    #[serde(default)]
    pub url_alias: String,

    /// Origin (original source URL).
    #[serde(default)]
    pub origin: String,
}

impl ReliefWebReportFields {
    /// Get the primary source name.
    pub fn source_name(&self) -> Option<&str> {
        self.source.first().map(|s| s.name.as_str())
    }

    /// Get the primary format.
    pub fn format_name(&self) -> Option<&str> {
        self.format.first().map(|f| f.name.as_str())
    }

    /// Get the primary country name.
    pub fn country_name(&self) -> Option<&str> {
        self.primary_country.as_ref().map(|c| c.name.as_str())
    }
}

// Country types

/// Countries list response.
pub type ReliefWebCountriesResponse = ReliefWebListResponse<ReliefWebCountryFields>;

/// Single country response.
pub type ReliefWebCountryResponse = ReliefWebSingleResponse<ReliefWebCountryFields>;

/// Country record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebCountryFields {
    /// Country name.
    #[serde(default)]
    pub name: String,

    /// Country description.
    #[serde(default)]
    pub description: String,

    /// ISO 3166-1 alpha-3 code.
    #[serde(default)]
    pub iso3: String,

    /// Country status in ReliefWeb.
    #[serde(default)]
    pub status: String,

    /// Country profile URL.
    #[serde(default)]
    pub url: String,

    /// Current situation overview.
    #[serde(default)]
    pub current: String,
}

// Job types

/// Jobs list response.
pub type ReliefWebJobsResponse = ReliefWebListResponse<ReliefWebJobFields>;

/// Job record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebJobFields {
    /// Job title.
    #[serde(default)]
    pub title: String,

    /// Job description.
    #[serde(default)]
    pub body: String,

    /// Posting organization.
    #[serde(default)]
    pub source: Vec<ReliefWebSource>,

    /// Job type (e.g., "Permanent", "Temporary").
    #[serde(default, rename = "type")]
    pub job_type: Vec<ReliefWebTerm>,

    /// Job experience level.
    #[serde(default)]
    pub experience: Vec<ReliefWebTerm>,

    /// Career category.
    #[serde(default)]
    pub career_categories: Vec<ReliefWebTerm>,

    /// Job location country.
    #[serde(default)]
    pub country: Vec<ReliefWebCountryRef>,

    /// Job city.
    #[serde(default)]
    pub city: Vec<ReliefWebTerm>,

    /// Application closing date.
    #[serde(default)]
    pub date: Option<ReliefWebDate>,

    /// Job URL.
    #[serde(default)]
    pub url: String,

    /// How to apply.
    #[serde(default)]
    pub how_to_apply: String,
}

// Training types

/// Training list response.
pub type ReliefWebTrainingResponse = ReliefWebListResponse<ReliefWebTrainingFields>;

/// Training record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebTrainingFields {
    /// Training title.
    #[serde(default)]
    pub title: String,

    /// Training description.
    #[serde(default)]
    pub body: String,

    /// Training provider.
    #[serde(default)]
    pub source: Vec<ReliefWebSource>,

    /// Training type.
    #[serde(default, rename = "type")]
    pub training_type: Vec<ReliefWebTerm>,

    /// Training format (online, in-person).
    #[serde(default)]
    pub format: Vec<ReliefWebTerm>,

    /// Training language.
    #[serde(default)]
    pub language: Vec<ReliefWebTerm>,

    /// Training theme.
    #[serde(default)]
    pub theme: Vec<ReliefWebTerm>,

    /// Training country.
    #[serde(default)]
    pub country: Vec<ReliefWebCountryRef>,

    /// Training dates.
    #[serde(default)]
    pub date: Option<ReliefWebDate>,

    /// Training URL.
    #[serde(default)]
    pub url: String,

    /// Cost information.
    #[serde(default)]
    pub cost: String,

    /// Registration URL.
    #[serde(default)]
    pub registration: String,
}

// Source types

/// Sources list response.
pub type ReliefWebSourcesResponse = ReliefWebListResponse<ReliefWebSourceFields>;

/// Source record fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebSourceFields {
    /// Source name.
    #[serde(default)]
    pub name: String,

    /// Source description.
    #[serde(default)]
    pub description: String,

    /// Source type.
    #[serde(default, rename = "type")]
    pub source_type: Vec<ReliefWebTerm>,

    /// Source homepage.
    #[serde(default)]
    pub homepage: String,

    /// Source URL in ReliefWeb.
    #[serde(default)]
    pub url: String,
}

// Common reference types

/// A term/taxonomy reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebTerm {
    /// Term ID.
    #[serde(default)]
    pub id: i64,

    /// Term name.
    #[serde(default)]
    pub name: String,
}

/// A source organization reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebSource {
    /// Source ID.
    #[serde(default)]
    pub id: i64,

    /// Source name.
    #[serde(default)]
    pub name: String,

    /// Source shortname.
    #[serde(default)]
    pub shortname: String,

    /// Source homepage.
    #[serde(default)]
    pub homepage: String,
}

/// A country reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebCountryRef {
    /// Country ID.
    #[serde(default)]
    pub id: i64,

    /// Country name.
    #[serde(default)]
    pub name: String,

    /// ISO 3166-1 alpha-3 code.
    #[serde(default)]
    pub iso3: String,

    /// Primary flag.
    #[serde(default)]
    pub primary: bool,
}

/// A disaster reference.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebDisasterRef {
    /// Disaster ID.
    #[serde(default)]
    pub id: i64,

    /// Disaster name.
    #[serde(default)]
    pub name: String,

    /// GLIDE number.
    #[serde(default)]
    pub glide: String,
}

/// A date object in ReliefWeb responses.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReliefWebDate {
    /// Original date string.
    #[serde(default)]
    pub original: String,

    /// ISO timestamp.
    #[serde(default)]
    pub created: String,

    /// Changed timestamp.
    #[serde(default)]
    pub changed: String,
}

impl ReliefWebDate {
    /// Parse the created date as DateTime.
    pub fn created_datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.created)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }
}

/// Common disaster types for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReliefWebDisasterType {
    Flood,
    Earthquake,
    Cyclone,
    Drought,
    Epidemic,
    Conflict,
    Fire,
    Landslide,
    Volcano,
    ColdWave,
    HeatWave,
    Storm,
    Tsunami,
    InsectInfestation,
    ComplexEmergency,
}

impl ReliefWebDisasterType {
    /// Get the API string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            ReliefWebDisasterType::Flood => "Flood",
            ReliefWebDisasterType::Earthquake => "Earthquake",
            ReliefWebDisasterType::Cyclone => "Tropical Cyclone",
            ReliefWebDisasterType::Drought => "Drought",
            ReliefWebDisasterType::Epidemic => "Epidemic",
            ReliefWebDisasterType::Conflict => "Conflict",
            ReliefWebDisasterType::Fire => "Fire",
            ReliefWebDisasterType::Landslide => "Landslide",
            ReliefWebDisasterType::Volcano => "Volcano",
            ReliefWebDisasterType::ColdWave => "Cold Wave",
            ReliefWebDisasterType::HeatWave => "Heat Wave",
            ReliefWebDisasterType::Storm => "Storm",
            ReliefWebDisasterType::Tsunami => "Tsunami",
            ReliefWebDisasterType::InsectInfestation => "Insect Infestation",
            ReliefWebDisasterType::ComplexEmergency => "Complex Emergency",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disaster_status() {
        let ongoing = ReliefWebDisasterFields {
            name: "Test Disaster".to_string(),
            status: "ongoing".to_string(),
            ..Default::default()
        };

        let past = ReliefWebDisasterFields {
            status: "past".to_string(),
            ..ongoing.clone()
        };

        assert!(ongoing.is_ongoing());
        assert!(!past.is_ongoing());
    }

    #[test]
    fn test_disaster_type_strings() {
        assert_eq!(ReliefWebDisasterType::Flood.as_str(), "Flood");
        assert_eq!(ReliefWebDisasterType::Cyclone.as_str(), "Tropical Cyclone");
        assert_eq!(
            ReliefWebDisasterType::ComplexEmergency.as_str(),
            "Complex Emergency"
        );
    }

    #[test]
    fn test_disaster_type_name() {
        let disaster = ReliefWebDisasterFields {
            disaster_type: vec![ReliefWebTerm {
                id: 1,
                name: "Earthquake".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(disaster.type_name(), Some("Earthquake"));
    }

    #[test]
    fn test_report_source() {
        let report = ReliefWebReportFields {
            source: vec![ReliefWebSource {
                id: 1,
                name: "OCHA".to_string(),
                shortname: "OCHA".to_string(),
                homepage: "https://www.unocha.org".to_string(),
            }],
            ..Default::default()
        };

        assert_eq!(report.source_name(), Some("OCHA"));
    }
}
