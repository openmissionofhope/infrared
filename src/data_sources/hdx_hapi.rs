//! HDX HAPI (Humanitarian API) client.
//!
//! HDX HAPI provides access to standardized humanitarian indicators from multiple sources
//! including OCHA (UN Office for the Coordination of Humanitarian Affairs).
//!
//! # Features
//!
//! - Affected population data
//! - Food security and nutrition indicators
//! - Conflict events data
//! - Humanitarian needs and response
//! - Refugee and IDP statistics
//!
//! # API Reference
//!
//! See: <https://hdx-hapi.readthedocs.io/en/latest/>
//!
//! # Privacy
//!
//! All data is aggregate humanitarian statistics. No individual persons are tracked.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Base URL for the HDX HAPI.
const HDX_HAPI_BASE: &str = "https://hapi.humdata.org/api/v1";

/// Client for querying the HDX Humanitarian API.
#[derive(Clone)]
pub struct HdxHapiClient {
    client: reqwest::Client,
    base_url: String,
    app_identifier: String,
}

impl Default for HdxHapiClient {
    fn default() -> Self {
        Self::new("infrared")
    }
}

impl HdxHapiClient {
    /// Create a new HDX HAPI client.
    ///
    /// # Arguments
    ///
    /// * `app_identifier` - Application identifier for API tracking (required by HDX).
    pub fn new(app_identifier: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: HDX_HAPI_BASE.to_string(),
            app_identifier: app_identifier.to_string(),
        }
    }

    /// Create a client with a custom base URL (for testing).
    pub fn with_base_url(base_url: &str, app_identifier: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            app_identifier: app_identifier.to_string(),
        }
    }

    /// Get humanitarian needs data for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code (e.g., "AFG", "UKR", "SYR")
    pub async fn get_humanitarian_needs(
        &self,
        country_code: &str,
    ) -> anyhow::Result<HdxHumanitarianNeedsResponse> {
        let url = format!(
            "{}/affected-people/humanitarian-needs?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxHumanitarianNeedsResponse>().await?;
        Ok(data)
    }

    /// Get refugee statistics for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    /// * `asylum_country` - Optional asylum country code to filter by
    pub async fn get_refugees(
        &self,
        country_code: Option<&str>,
        asylum_country: Option<&str>,
    ) -> anyhow::Result<HdxRefugeesResponse> {
        let mut url = format!(
            "{}/affected-people/refugees?app_identifier={}",
            self.base_url, self.app_identifier
        );

        if let Some(code) = country_code {
            url.push_str(&format!("&origin_location_code={}", code.to_uppercase()));
        }
        if let Some(asylum) = asylum_country {
            url.push_str(&format!("&asylum_location_code={}", asylum.to_uppercase()));
        }

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxRefugeesResponse>().await?;
        Ok(data)
    }

    /// Get internally displaced persons (IDP) data.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_idps(&self, country_code: &str) -> anyhow::Result<HdxIdpsResponse> {
        let url = format!(
            "{}/affected-people/idps?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxIdpsResponse>().await?;
        Ok(data)
    }

    /// Get food security (IPC/CH) data for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_food_security(
        &self,
        country_code: &str,
    ) -> anyhow::Result<HdxFoodSecurityResponse> {
        let url = format!(
            "{}/food/food-security?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxFoodSecurityResponse>().await?;
        Ok(data)
    }

    /// Get food prices for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_food_prices(
        &self,
        country_code: &str,
    ) -> anyhow::Result<HdxFoodPricesResponse> {
        let url = format!(
            "{}/food/food-price?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxFoodPricesResponse>().await?;
        Ok(data)
    }

    /// Get conflict events for a country.
    ///
    /// Note: This returns ACLED data via HDX HAPI.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_conflict_events(
        &self,
        country_code: &str,
    ) -> anyhow::Result<HdxConflictEventsResponse> {
        let url = format!(
            "{}/coordination-context/conflict-event?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxConflictEventsResponse>().await?;
        Ok(data)
    }

    /// Get operational presence (3W: Who does What Where) data.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_operational_presence(
        &self,
        country_code: &str,
    ) -> anyhow::Result<HdxOperationalPresenceResponse> {
        let url = format!(
            "{}/coordination-context/operational-presence?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxOperationalPresenceResponse>().await?;
        Ok(data)
    }

    /// Get country-level population statistics.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_population(&self, country_code: &str) -> anyhow::Result<HdxPopulationResponse> {
        let url = format!(
            "{}/population-social/population?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxPopulationResponse>().await?;
        Ok(data)
    }

    /// Get poverty indicators for a country.
    ///
    /// # Arguments
    ///
    /// * `country_code` - ISO 3166-1 alpha-3 country code
    pub async fn get_poverty(&self, country_code: &str) -> anyhow::Result<HdxPovertyResponse> {
        let url = format!(
            "{}/population-social/poverty-rate?location_code={}&app_identifier={}",
            self.base_url,
            country_code.to_uppercase(),
            self.app_identifier
        );

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxPovertyResponse>().await?;
        Ok(data)
    }

    /// Get national risk indicators.
    ///
    /// # Arguments
    ///
    /// * `country_code` - Optional ISO 3166-1 alpha-3 country code (returns all if None)
    pub async fn get_national_risk(
        &self,
        country_code: Option<&str>,
    ) -> anyhow::Result<HdxNationalRiskResponse> {
        let mut url = format!(
            "{}/coordination-context/national-risk?app_identifier={}",
            self.base_url, self.app_identifier
        );

        if let Some(code) = country_code {
            url.push_str(&format!("&location_code={}", code.to_uppercase()));
        }

        let response = self.client.get(&url).send().await?;
        let data = response.json::<HdxNationalRiskResponse>().await?;
        Ok(data)
    }
}

// ============================================================================
// Response types
// ============================================================================

/// Generic paginated response wrapper from HDX HAPI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdxPaginatedResponse<T> {
    /// List of data items.
    #[serde(default)]
    pub data: Vec<T>,
}

/// Humanitarian needs response.
pub type HdxHumanitarianNeedsResponse = HdxPaginatedResponse<HdxHumanitarianNeed>;

/// A single humanitarian needs record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxHumanitarianNeed {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Population group.
    #[serde(default)]
    pub population_group: String,

    /// Population status.
    #[serde(default)]
    pub population_status: String,

    /// Gender category.
    #[serde(default)]
    pub gender: String,

    /// Age range.
    #[serde(default)]
    pub age_range: String,

    /// Population count in need.
    #[serde(default)]
    pub population: Option<i64>,
}

impl HdxHumanitarianNeed {
    /// Get reference period start as DateTime.
    pub fn start_date(&self) -> Option<DateTime<Utc>> {
        self.reference_period_start.as_ref().and_then(|s| {
            DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
        })
    }
}

/// Refugees response.
pub type HdxRefugeesResponse = HdxPaginatedResponse<HdxRefugee>;

/// A single refugee statistics record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxRefugee {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Origin country code.
    #[serde(default)]
    pub origin_location_code: String,

    /// Origin country name.
    #[serde(default)]
    pub origin_location_name: String,

    /// Asylum country code.
    #[serde(default)]
    pub asylum_location_code: String,

    /// Asylum country name.
    #[serde(default)]
    pub asylum_location_name: String,

    /// Population group.
    #[serde(default)]
    pub population_group: String,

    /// Gender category.
    #[serde(default)]
    pub gender: String,

    /// Age range.
    #[serde(default)]
    pub age_range: String,

    /// Number of refugees.
    #[serde(default)]
    pub population: Option<i64>,
}

/// IDPs response.
pub type HdxIdpsResponse = HdxPaginatedResponse<HdxIdp>;

/// A single IDP statistics record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxIdp {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Number of IDPs.
    #[serde(default)]
    pub population: Option<i64>,
}

/// Food security response.
pub type HdxFoodSecurityResponse = HdxPaginatedResponse<HdxFoodSecurity>;

/// A single food security (IPC/CH) record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxFoodSecurity {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// IPC phase classification (1-5).
    #[serde(default)]
    pub ipc_phase: Option<i32>,

    /// IPC type (current, projected).
    #[serde(default)]
    pub ipc_type: String,

    /// Population in this phase.
    #[serde(default)]
    pub population_in_phase: Option<i64>,

    /// Fraction of total population.
    #[serde(default)]
    pub population_fraction_in_phase: Option<f64>,
}

impl HdxFoodSecurity {
    /// Check if this is a crisis-level food insecurity (IPC Phase 3+).
    pub fn is_crisis_level(&self) -> bool {
        self.ipc_phase.map_or(false, |p| p >= 3)
    }

    /// Check if this is emergency-level food insecurity (IPC Phase 4+).
    pub fn is_emergency_level(&self) -> bool {
        self.ipc_phase.map_or(false, |p| p >= 4)
    }

    /// Check if this is famine (IPC Phase 5).
    pub fn is_famine(&self) -> bool {
        self.ipc_phase == Some(5)
    }
}

/// Food prices response.
pub type HdxFoodPricesResponse = HdxPaginatedResponse<HdxFoodPrice>;

/// A single food price record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxFoodPrice {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Market name.
    #[serde(default)]
    pub market_name: String,

    /// Commodity category.
    #[serde(default)]
    pub commodity_category: String,

    /// Commodity name.
    #[serde(default)]
    pub commodity_name: String,

    /// Currency code.
    #[serde(default)]
    pub currency_code: String,

    /// Price value.
    #[serde(default)]
    pub price: Option<f64>,

    /// Price type (retail, wholesale).
    #[serde(default)]
    pub price_type: String,

    /// Unit of measurement.
    #[serde(default)]
    pub unit: String,
}

/// Conflict events response.
pub type HdxConflictEventsResponse = HdxPaginatedResponse<HdxConflictEvent>;

/// A single conflict event record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxConflictEvent {
    /// Reference period start (event date).
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Admin level 1 (region/province).
    #[serde(default)]
    pub admin1_name: String,

    /// Admin level 2 (district).
    #[serde(default)]
    pub admin2_name: String,

    /// Event type.
    #[serde(default)]
    pub event_type: String,

    /// Number of events.
    #[serde(default)]
    pub events: Option<i64>,

    /// Number of fatalities.
    #[serde(default)]
    pub fatalities: Option<i64>,
}

impl HdxConflictEvent {
    /// Check if there were any fatalities.
    pub fn has_fatalities(&self) -> bool {
        self.fatalities.map_or(false, |f| f > 0)
    }
}

/// Operational presence response.
pub type HdxOperationalPresenceResponse = HdxPaginatedResponse<HdxOperationalPresence>;

/// A single operational presence (3W) record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxOperationalPresence {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Organization name.
    #[serde(default)]
    pub org_name: String,

    /// Organization type.
    #[serde(default)]
    pub org_type_description: String,

    /// Sector/cluster.
    #[serde(default)]
    pub sector_name: String,
}

/// Population response.
pub type HdxPopulationResponse = HdxPaginatedResponse<HdxPopulation>;

/// A single population record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxPopulation {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Gender category.
    #[serde(default)]
    pub gender: String,

    /// Age range.
    #[serde(default)]
    pub age_range: String,

    /// Population count.
    #[serde(default)]
    pub population: Option<i64>,
}

/// Poverty response.
pub type HdxPovertyResponse = HdxPaginatedResponse<HdxPoverty>;

/// A single poverty indicator record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxPoverty {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// MPI (Multidimensional Poverty Index).
    #[serde(default)]
    pub mpi: Option<f64>,

    /// Headcount ratio (percentage in poverty).
    #[serde(default)]
    pub headcount_ratio: Option<f64>,

    /// Intensity of poverty.
    #[serde(default)]
    pub intensity_of_deprivation: Option<f64>,

    /// Vulnerable to poverty (percentage).
    #[serde(default)]
    pub vulnerable_to_poverty: Option<f64>,

    /// In severe poverty (percentage).
    #[serde(default)]
    pub in_severe_poverty: Option<f64>,
}

/// National risk response.
pub type HdxNationalRiskResponse = HdxPaginatedResponse<HdxNationalRisk>;

/// A single national risk indicator record.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HdxNationalRisk {
    /// Reference period start.
    #[serde(default)]
    pub reference_period_start: Option<String>,

    /// Reference period end.
    #[serde(default)]
    pub reference_period_end: Option<String>,

    /// Location code.
    #[serde(default)]
    pub location_code: String,

    /// Location name.
    #[serde(default)]
    pub location_name: String,

    /// Overall risk score.
    #[serde(default)]
    pub overall_risk: Option<f64>,

    /// Hazard & exposure score.
    #[serde(default)]
    pub hazard_exposure: Option<f64>,

    /// Vulnerability score.
    #[serde(default)]
    pub vulnerability: Option<f64>,

    /// Coping capacity score.
    #[serde(default)]
    pub coping_capacity: Option<f64>,
}

impl HdxNationalRisk {
    /// Check if the country is at high risk (score >= 5.0 on typical 0-10 scale).
    pub fn is_high_risk(&self) -> bool {
        self.overall_risk.map_or(false, |r| r >= 5.0)
    }

    /// Check if the country is at very high risk (score >= 7.0).
    pub fn is_very_high_risk(&self) -> bool {
        self.overall_risk.map_or(false, |r| r >= 7.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_food_security_levels() {
        let crisis = HdxFoodSecurity {
            reference_period_start: None,
            reference_period_end: None,
            location_code: "AFG".to_string(),
            location_name: "Afghanistan".to_string(),
            ipc_phase: Some(3),
            ipc_type: "current".to_string(),
            population_in_phase: Some(1_000_000),
            population_fraction_in_phase: Some(0.1),
        };

        assert!(crisis.is_crisis_level());
        assert!(!crisis.is_emergency_level());
        assert!(!crisis.is_famine());

        let famine = HdxFoodSecurity {
            ipc_phase: Some(5),
            ..crisis.clone()
        };

        assert!(famine.is_crisis_level());
        assert!(famine.is_emergency_level());
        assert!(famine.is_famine());
    }

    #[test]
    fn test_conflict_event_fatalities() {
        let event_with_fatalities = HdxConflictEvent {
            reference_period_start: None,
            reference_period_end: None,
            location_code: "UKR".to_string(),
            location_name: "Ukraine".to_string(),
            admin1_name: "Kyiv".to_string(),
            admin2_name: "".to_string(),
            event_type: "battles".to_string(),
            events: Some(10),
            fatalities: Some(5),
        };

        let event_without_fatalities = HdxConflictEvent {
            fatalities: Some(0),
            ..event_with_fatalities.clone()
        };

        assert!(event_with_fatalities.has_fatalities());
        assert!(!event_without_fatalities.has_fatalities());
    }

    #[test]
    fn test_national_risk_levels() {
        let high_risk = HdxNationalRisk {
            reference_period_start: None,
            reference_period_end: None,
            location_code: "SOM".to_string(),
            location_name: "Somalia".to_string(),
            overall_risk: Some(6.5),
            hazard_exposure: Some(7.0),
            vulnerability: Some(8.0),
            coping_capacity: Some(4.0),
        };

        assert!(high_risk.is_high_risk());
        assert!(!high_risk.is_very_high_risk());

        let very_high_risk = HdxNationalRisk {
            overall_risk: Some(8.0),
            ..high_risk.clone()
        };

        assert!(very_high_risk.is_very_high_risk());
    }
}
