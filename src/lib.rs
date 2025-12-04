//! Infrared - A privacy-preserving system for detecting signs of life at scale.
//!
//! # Overview
//!
//! Infrared tracks aggregate "warmth" (activity/presence) at a bucket/region level
//! without tracking individuals. It measures population-level life signals and
//! detects large-scale drops or disappearances.
//!
//! # Privacy Guarantees
//!
//! Infrared is designed to be **privacy-safe by construction**:
//!
//! - No identity tracking (no usernames, emails, account IDs)
//! - No location tracking (no GPS, IP addresses)
//! - No device tracking (no device IDs, fingerprints)
//! - No behavioral profiling (no cross-session linking)
//!
//! If the entire database were leaked publicly, **no individual could be
//! identified, located, or profiled**.
//!
//! # Modules
//!
//! - [`model`]: Data types for life signals, warmth responses, and alerts
//! - [`storage`]: SQLite storage layer
//! - [`aggregation`]: Logic for computing warmth indices
//! - [`api`]: HTTP API handlers

pub mod aggregation;
pub mod api;
pub mod model;
pub mod storage;
