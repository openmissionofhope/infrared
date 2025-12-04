//! External data sources for real-time country-level connectivity and crisis monitoring.
//!
//! This module provides clients for fetching near real-time internet connectivity,
//! traffic data, conflict events, and humanitarian information from public APIs.
//! These can be used to detect large-scale outages, humanitarian crises, and
//! "everyone suddenly offline" scenarios.
//!
//! # Data Sources
//!
//! ## Internet Connectivity
//!
//! - [`ioda`]: IODA (Internet Outage Detection and Analysis) - specialized for outage detection
//! - [`cloudflare`]: Cloudflare Radar - traffic volume and anomaly data
//!
//! ## Humanitarian Data
//!
//! - [`hdx_hapi`]: HDX HAPI (OCHA) - humanitarian indicators, refugees, IDPs, food security
//! - [`reliefweb`]: ReliefWeb - disasters, reports, humanitarian updates
//!
//! ## Conflict Data
//!
//! - [`acled`]: ACLED - armed conflict events, protests, violence against civilians
//!
//! # Privacy
//!
//! These data sources provide only aggregate, country-level statistics.
//! No individual user data is collected or processed.

pub mod acled;
pub mod cloudflare;
pub mod hdx_hapi;
pub mod ioda;
pub mod reliefweb;

pub use acled::AcledClient;
pub use cloudflare::CloudflareRadarClient;
pub use hdx_hapi::HdxHapiClient;
pub use ioda::IodaClient;
pub use reliefweb::ReliefWebClient;
