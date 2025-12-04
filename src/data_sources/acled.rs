//! ACLED (Armed Conflict Location & Event Data) API client.
//!
//! ACLED is the highest quality and most widely used real-time data source on
//! political violence and protest activity around the world.
//!
//! # Features
//!
//! - Conflict events (battles, explosions, violence against civilians)
//! - Protest and demonstration tracking
//! - Actor information (state forces, rebel groups, etc.)
//! - Fatality estimates
//! - Geographic data (admin levels, coordinates)
//!
//! # API Reference
//!
//! See: <https://acleddata.com/acled-api-documentation>
//!
//! # Authentication
//!
//! Requires registration at <https://acleddata.com/register/> to obtain API key.
//!
//! # Privacy
//!
//! All data is aggregate event-level statistics. No individual persons are tracked.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Base URL for the ACLED API.
const ACLED_API_BASE: &str = "https://api.acleddata.com/acled/read";

/// Client for querying the ACLED conflict data API.
#[derive(Clone)]
pub struct AcledClient {
    client: reqwest::Client,
    base_url: String,
    email: String,
    api_key: String,
}

impl AcledClient {
    /// Create a new ACLED client.
    ///
    /// # Arguments
    ///
    /// * `email` - Registered email address for ACLED access.
    /// * `api_key` - API key obtained from ACLED registration.
    pub fn new(email: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: ACLED_API_BASE.to_string(),
            email: email.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Create a client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str, email: &str, api_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            email: email.to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Build the authentication query string.
    fn auth_params(&self) -> String {
        format!("key={}&email={}", self.api_key, self.email)
    }

    /// Get conflict events for a specific country.
    ///
    /// # Arguments
    ///
    /// * `country` - Country name (e.g., "Ukraine", "Syria", "Afghanistan")
    /// * `limit` - Maximum number of events to return (default: 500)
    pub async fn get_events_by_country(
        &self,
        country: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&country={}&limit={}",
            self.base_url,
            self.auth_params(),
            urlencoding::encode(country),
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }

    /// Get conflict events for a date range.
    ///
    /// # Arguments
    ///
    /// * `country` - Country name
    /// * `event_date_start` - Start date (YYYY-MM-DD format)
    /// * `event_date_end` - End date (YYYY-MM-DD format)
    /// * `limit` - Maximum number of events to return
    pub async fn get_events_by_date_range(
        &self,
        country: &str,
        event_date_start: &str,
        event_date_end: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&country={}&event_date={}&event_date_where=BETWEEN&event_date={}&limit={}",
            self.base_url,
            self.auth_params(),
            urlencoding::encode(country),
            event_date_start,
            event_date_end,
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }

    /// Get events by event type.
    ///
    /// # Arguments
    ///
    /// * `country` - Country name
    /// * `event_type` - Event type (e.g., "Battles", "Explosions/Remote violence",
    ///                  "Violence against civilians", "Protests", "Riots", "Strategic developments")
    /// * `limit` - Maximum number of events to return
    pub async fn get_events_by_type(
        &self,
        country: &str,
        event_type: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&country={}&event_type={}&limit={}",
            self.base_url,
            self.auth_params(),
            urlencoding::encode(country),
            urlencoding::encode(event_type),
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }

    /// Get events with fatalities.
    ///
    /// # Arguments
    ///
    /// * `country` - Country name
    /// * `min_fatalities` - Minimum number of fatalities to filter by
    /// * `limit` - Maximum number of events to return
    pub async fn get_events_with_fatalities(
        &self,
        country: &str,
        min_fatalities: u32,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&country={}&fatalities={}&fatalities_where=>=&limit={}",
            self.base_url,
            self.auth_params(),
            urlencoding::encode(country),
            min_fatalities,
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }

    /// Get recent events (last N days).
    ///
    /// # Arguments
    ///
    /// * `country` - Country name
    /// * `days` - Number of days to look back
    /// * `limit` - Maximum number of events to return
    pub async fn get_recent_events(
        &self,
        country: &str,
        days: u32,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - chrono::Duration::days(days as i64);

        self.get_events_by_date_range(
            country,
            &start_date.format("%Y-%m-%d").to_string(),
            &end_date.format("%Y-%m-%d").to_string(),
            limit,
        )
        .await
    }

    /// Get events by region.
    ///
    /// # Arguments
    ///
    /// * `region` - ACLED region number:
    ///   - 1: Western Africa
    ///   - 2: Middle Africa
    ///   - 3: Eastern Africa
    ///   - 4: Southern Africa
    ///   - 5: Northern Africa
    ///   - 6: South Asia
    ///   - 7: Southeast Asia
    ///   - 8: Middle East
    ///   - 9: Europe
    ///   - 10: Caucasus and Central Asia
    ///   - 11: Central America
    ///   - 12: South America
    ///   - 13: Caribbean
    ///   - 14: East Asia
    ///   - 15: North America
    ///   - 16: Oceania
    /// * `limit` - Maximum number of events to return
    pub async fn get_events_by_region(
        &self,
        region: u32,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&region={}&limit={}",
            self.base_url,
            self.auth_params(),
            region,
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }

    /// Get events involving a specific actor.
    ///
    /// # Arguments
    ///
    /// * `country` - Country name
    /// * `actor` - Actor name or partial name (e.g., "Military", "Police", "Rebels")
    /// * `limit` - Maximum number of events to return
    pub async fn get_events_by_actor(
        &self,
        country: &str,
        actor: &str,
        limit: Option<u32>,
    ) -> anyhow::Result<AcledResponse> {
        let limit = limit.unwrap_or(500);
        let url = format!(
            "{}?{}&country={}&actor1={}&limit={}",
            self.base_url,
            self.auth_params(),
            urlencoding::encode(country),
            urlencoding::encode(actor),
            limit
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<AcledResponse>().await?;
        Ok(data)
    }
}

// ============================================================================
// Response types
// ============================================================================

/// Response from the ACLED API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcledResponse {
    /// Whether the request was successful.
    #[serde(default)]
    pub success: bool,

    /// Error message if request failed.
    #[serde(default)]
    pub error: Option<String>,

    /// Number of events returned.
    #[serde(default)]
    pub count: i64,

    /// List of conflict events.
    #[serde(default)]
    pub data: Vec<AcledEvent>,
}

impl AcledResponse {
    /// Get total fatalities across all events.
    pub fn total_fatalities(&self) -> i64 {
        self.data.iter().filter_map(|e| e.fatalities).sum()
    }

    /// Get count of events by type.
    pub fn events_by_type(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();
        for event in &self.data {
            *counts.entry(event.event_type.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Get the most recent event.
    pub fn most_recent(&self) -> Option<&AcledEvent> {
        self.data.iter().max_by_key(|e| &e.event_date)
    }

    /// Filter to only events with fatalities.
    pub fn with_fatalities(&self) -> Vec<&AcledEvent> {
        self.data
            .iter()
            .filter(|e| e.fatalities.map_or(false, |f| f > 0))
            .collect()
    }
}

/// A single ACLED conflict event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcledEvent {
    /// Unique event identifier.
    #[serde(default)]
    pub event_id_cnty: String,

    /// Event date (YYYY-MM-DD).
    #[serde(default)]
    pub event_date: String,

    /// Year of event.
    #[serde(default)]
    pub year: Option<i32>,

    /// Time precision level.
    #[serde(default)]
    pub time_precision: Option<i32>,

    /// Disorder type.
    #[serde(default)]
    pub disorder_type: String,

    /// Event type (Battles, Explosions/Remote violence, etc.).
    #[serde(default)]
    pub event_type: String,

    /// Sub-event type.
    #[serde(default)]
    pub sub_event_type: String,

    /// Primary actor name.
    #[serde(default)]
    pub actor1: String,

    /// Secondary actor name.
    #[serde(default)]
    pub actor2: String,

    /// Interaction type code.
    #[serde(default)]
    pub interaction: Option<i32>,

    /// Country name.
    #[serde(default)]
    pub country: String,

    /// ISO 3166-1 numeric country code.
    #[serde(default)]
    pub iso: Option<i32>,

    /// ISO 3166-1 alpha-3 country code.
    #[serde(default)]
    pub iso3: String,

    /// ACLED region number.
    #[serde(default)]
    pub region: Option<i32>,

    /// Admin level 1 (region/province).
    #[serde(default)]
    pub admin1: String,

    /// Admin level 2 (district).
    #[serde(default)]
    pub admin2: String,

    /// Admin level 3.
    #[serde(default)]
    pub admin3: String,

    /// Location name.
    #[serde(default)]
    pub location: String,

    /// Latitude.
    #[serde(default)]
    pub latitude: Option<f64>,

    /// Longitude.
    #[serde(default)]
    pub longitude: Option<f64>,

    /// Geographic precision level.
    #[serde(default)]
    pub geo_precision: Option<i32>,

    /// Data source.
    #[serde(default)]
    pub source: String,

    /// Source scale.
    #[serde(default)]
    pub source_scale: String,

    /// Notes about the event.
    #[serde(default)]
    pub notes: String,

    /// Number of fatalities.
    #[serde(default)]
    pub fatalities: Option<i64>,

    /// Timestamp of event record.
    #[serde(default)]
    pub timestamp: Option<i64>,
}

impl AcledEvent {
    /// Get the event date as a NaiveDate.
    pub fn date(&self) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(&self.event_date, "%Y-%m-%d").ok()
    }

    /// Get the event date as a DateTime (at midnight UTC).
    pub fn datetime(&self) -> Option<DateTime<Utc>> {
        self.date()
            .and_then(|d| d.and_hms_opt(0, 0, 0))
            .map(|dt| dt.and_utc())
    }

    /// Check if the event involved violence against civilians.
    pub fn is_civilian_targeting(&self) -> bool {
        self.event_type.contains("Violence against civilians")
    }

    /// Check if the event was a battle.
    pub fn is_battle(&self) -> bool {
        self.event_type.contains("Battles")
    }

    /// Check if the event was a protest.
    pub fn is_protest(&self) -> bool {
        self.event_type.contains("Protests")
    }

    /// Check if the event was a riot.
    pub fn is_riot(&self) -> bool {
        self.event_type.contains("Riots")
    }

    /// Check if the event involved explosions or remote violence.
    pub fn is_explosion(&self) -> bool {
        self.event_type.contains("Explosions") || self.event_type.contains("Remote violence")
    }

    /// Check if this was a lethal event.
    pub fn is_lethal(&self) -> bool {
        self.fatalities.map_or(false, |f| f > 0)
    }

    /// Get coordinates as a tuple.
    pub fn coordinates(&self) -> Option<(f64, f64)> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }
}

/// ACLED event types for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcledEventType {
    Battles,
    ExplosionsRemoteViolence,
    ViolenceAgainstCivilians,
    Protests,
    Riots,
    StrategicDevelopments,
}

impl AcledEventType {
    /// Get the API string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            AcledEventType::Battles => "Battles",
            AcledEventType::ExplosionsRemoteViolence => "Explosions/Remote violence",
            AcledEventType::ViolenceAgainstCivilians => "Violence against civilians",
            AcledEventType::Protests => "Protests",
            AcledEventType::Riots => "Riots",
            AcledEventType::StrategicDevelopments => "Strategic developments",
        }
    }
}

/// ACLED regions for filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcledRegion {
    WesternAfrica = 1,
    MiddleAfrica = 2,
    EasternAfrica = 3,
    SouthernAfrica = 4,
    NorthernAfrica = 5,
    SouthAsia = 6,
    SoutheastAsia = 7,
    MiddleEast = 8,
    Europe = 9,
    CaucasusCentralAsia = 10,
    CentralAmerica = 11,
    SouthAmerica = 12,
    Caribbean = 13,
    EastAsia = 14,
    NorthAmerica = 15,
    Oceania = 16,
}

impl AcledRegion {
    /// Get the region number for API calls.
    pub fn number(&self) -> u32 {
        *self as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn sample_event() -> AcledEvent {
        AcledEvent {
            event_id_cnty: "UKR12345".to_string(),
            event_date: "2024-01-15".to_string(),
            year: Some(2024),
            time_precision: Some(1),
            disorder_type: "Political violence".to_string(),
            event_type: "Battles".to_string(),
            sub_event_type: "Armed clash".to_string(),
            actor1: "Military Forces of Ukraine".to_string(),
            actor2: "Military Forces of Russia".to_string(),
            interaction: Some(11),
            country: "Ukraine".to_string(),
            iso: Some(804),
            iso3: "UKR".to_string(),
            region: Some(9),
            admin1: "Donetsk".to_string(),
            admin2: "".to_string(),
            admin3: "".to_string(),
            location: "Bakhmut".to_string(),
            latitude: Some(48.5953),
            longitude: Some(38.0003),
            geo_precision: Some(1),
            source: "Ukrainian Armed Forces".to_string(),
            source_scale: "National".to_string(),
            notes: "Clashes reported".to_string(),
            fatalities: Some(5),
            timestamp: Some(1705276800),
        }
    }

    #[test]
    fn test_event_date_parsing() {
        let event = sample_event();
        let date = event.date().unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_event_type_detection() {
        let battle = sample_event();
        assert!(battle.is_battle());
        assert!(!battle.is_protest());

        let protest = AcledEvent {
            event_type: "Protests".to_string(),
            ..sample_event()
        };
        assert!(protest.is_protest());
        assert!(!protest.is_battle());

        let civilian = AcledEvent {
            event_type: "Violence against civilians".to_string(),
            ..sample_event()
        };
        assert!(civilian.is_civilian_targeting());
    }

    #[test]
    fn test_response_statistics() {
        let response = AcledResponse {
            success: true,
            error: None,
            count: 3,
            data: vec![
                AcledEvent {
                    fatalities: Some(5),
                    event_type: "Battles".to_string(),
                    ..sample_event()
                },
                AcledEvent {
                    fatalities: Some(3),
                    event_type: "Battles".to_string(),
                    ..sample_event()
                },
                AcledEvent {
                    fatalities: Some(0),
                    event_type: "Protests".to_string(),
                    ..sample_event()
                },
            ],
        };

        assert_eq!(response.total_fatalities(), 8);
        assert_eq!(response.with_fatalities().len(), 2);

        let by_type = response.events_by_type();
        assert_eq!(by_type.get("Battles"), Some(&2));
        assert_eq!(by_type.get("Protests"), Some(&1));
    }

    #[test]
    fn test_event_coordinates() {
        let event = sample_event();
        let coords = event.coordinates().unwrap();
        assert!((coords.0 - 48.5953).abs() < 0.001);
        assert!((coords.1 - 38.0003).abs() < 0.001);
    }

    #[test]
    fn test_region_numbers() {
        assert_eq!(AcledRegion::MiddleEast.number(), 8);
        assert_eq!(AcledRegion::Europe.number(), 9);
    }
}
