//! External data sources for real-time country-level connectivity monitoring.
//!
//! This module provides clients for fetching near real-time internet connectivity
//! and traffic data from public APIs. These can be used to detect large-scale
//! outages or "everyone suddenly offline" scenarios.
//!
//! # Data Sources
//!
//! - [`ioda`]: IODA (Internet Outage Detection and Analysis) - specialized for outage detection
//! - [`cloudflare`]: Cloudflare Radar - traffic volume and anomaly data
//!
//! # Privacy
//!
//! These data sources provide only aggregate, country-level statistics.
//! No individual user data is collected or processed.

pub mod cloudflare;
pub mod ioda;

pub use cloudflare::CloudflareRadarClient;
pub use ioda::IodaClient;
