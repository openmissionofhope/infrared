# Infrared
*A privacy-preserving system for detecting signs of life at scale.*

Infrared is a minimal-data framework designed to detect **population-level life signals** without tracking, identifying, or profiling any individual.
Just as living creatures emit **infrared warmth** that reveals presence without revealing identity, Infrared measures **aggregate activity patterns**—not people.

Infrared ensures that **large-scale disappearance, collapse, or catastrophic harm** cannot occur silently, while keeping every person's privacy, dignity, and legal protections intact.

---

## Quick Start

### Build and Run

```bash
# Build the project
cargo build --release

# Run with default settings (port 3000, SQLite database)
cargo run --release

# Or with custom configuration
INFRARED_PORT=8080 INFRARED_DATABASE_URL="sqlite:mydata.db?mode=rwc" cargo run --release
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `INFRARED_PORT` | `3000` | HTTP server port |
| `INFRARED_DATABASE_URL` | `sqlite:infrared.db?mode=rwc` | SQLite database URL |

---

## API Reference

### POST /signal

Record a life signal for a bucket.

**Request:**
```bash
curl -X POST http://localhost:3000/signal \
  -H "Content-Type: application/json" \
  -d '{"bucket": "zone-a", "weight": 1}'
```

**Body:**
```json
{
  "bucket": "zone-a",
  "weight": 1
}
```

- `bucket` (required): Coarse category identifier (e.g., "region:north", "cluster:web-01")
- `weight` (optional, default: 1): Signal intensity

**Response:** `202 Accepted`

---

### GET /warmth

Query the warmth index for a specific bucket.

**Request:**
```bash
curl "http://localhost:3000/warmth?bucket=zone-a&window_minutes=10"
```

**Query Parameters:**
- `bucket` (required): The bucket to query
- `window_minutes` (optional, default: 10): Time window in minutes

**Response:**
```json
{
  "bucket": "zone-a",
  "window_minutes": 10,
  "current_window_total": 42,
  "recent_average": 50.5,
  "status": "alive"
}
```

**Status Values:**
| Status | Condition |
|--------|-----------|
| `alive` | Current >= 80% of recent average |
| `stressed` | Current is 20-80% of recent average |
| `collapsing` | Current is >0 but <20% of recent average |
| `dead` | Current is 0 while recent average > 0 |

---

### GET /alerts/recent

Get alerts for all buckets currently in distress.

**Request:**
```bash
curl "http://localhost:3000/alerts/recent?minutes=60"
```

**Query Parameters:**
- `minutes` (optional, default: 60): Lookback window in minutes

**Response:**
```json
{
  "alerts": [
    {
      "bucket": "zone-a",
      "status": "dead",
      "last_seen_timestamp": "2024-01-15T10:30:00Z",
      "recent_average": 50.0,
      "message": "CRITICAL: Bucket 'zone-a' has gone completely silent..."
    }
  ],
  "lookback_minutes": 60
}
```

---

### GET /health

Simple health check endpoint.

**Request:**
```bash
curl http://localhost:3000/health
```

**Response:** `200 OK`

---

## Purpose

**Infrared exists to answer one question:**

> **Is life still present here?**

It monitors continuity of existence through *aggregate warmth signals*:

- overall activity levels
- event density
- communication volume
- temporal presence patterns
- system "pulses" and "heartbeats"

By tracking only **signals of life**, not the identities generating them, Infrared can detect:

- sudden population drops
- vanishing activity in a region or system
- abnormal declines
- catastrophic events
- infrastructure failures
- natural disasters
- concentrated disappearance patterns

All **without ever touching personal data**.

---

## Privacy and Legal Safety

Infrared is built to be inherently safe:

- **No identity tracking**
- **No location tracking**
- **No personal identifiers**
- **No behavioral profiling**
- **No cross-session linking**
- **No user consent requirements**
- **No GDPR/CCPA exposure**
- **No surveillance value**

Even if all Infrared data were leaked publicly, **no individual could be found, inferred, or reconstructed**.

Infrared observes only **population heat**, never individuals.

### What Infrared stores:

| Field | Description |
|-------|-------------|
| `bucket` | Coarse category (e.g., "region:north") |
| `timestamp` | Server-assigned UTC timestamp |
| `weight` | Numeric intensity |

### What Infrared NEVER stores:

- Usernames or emails
- IP addresses
- GPS coordinates
- Device IDs
- Biometrics
- Personal attributes
- Identifiable content

---

## External Data Sources

Infrared integrates with multiple public APIs to detect large-scale crises:

- **Internet connectivity**: Detect "everyone suddenly offline" scenarios via IODA and Cloudflare Radar
- **Humanitarian data**: Track disasters, displacement, and food security via ReliefWeb and HDX HAPI
- **Conflict monitoring**: Monitor violence, protests, and unrest via ACLED

### IODA (Internet Outage Detection and Analysis)

IODA monitors the Internet in near real-time to identify macroscopic outages at the country, regional, or ASN level. Data updates every ~5 minutes.

```rust
use infrared::data_sources::IodaClient;

let client = IodaClient::new();

// Get outage alerts from the last 24 hours for all countries
let alerts = client.get_recent_alerts(24).await?;

// Get alerts for a specific country (ISO 3166-1 alpha-2 code)
let us_alerts = client.get_recent_country_alerts("US", 6).await?;

// Get raw connectivity signals (BGP, active probing, darknet)
let now = chrono::Utc::now().timestamp();
let one_day_ago = now - 86400;
let signals = client.get_country_signals("DE", one_day_ago, now).await?;

// Check for significant drops
for alert in alerts.data {
    println!("{}: {}% drop", alert.entity_name, alert.drop_percentage());
}
```

### Cloudflare Radar

Cloudflare Radar provides traffic volume data from Cloudflare's global network (330+ cities, 120+ countries). Data available at 15-minute granularity.

```rust
use infrared::data_sources::CloudflareRadarClient;

// API token required for best results (free tier available)
let client = CloudflareRadarClient::new(Some("your-api-token".into()));

// Get last 24 hours of traffic for a country
let traffic = client.get_daily_traffic("US").await?;

// Get last 7 days
let weekly = client.get_weekly_traffic("JP").await?;

// Compare multiple countries
let comparison = client.compare_countries(&["US", "DE", "JP"], "7d").await?;

// Detect significant traffic drops
if let Some(result) = traffic.result {
    for series in result.series {
        if series.has_significant_drop(0.5) {
            println!("Traffic in {} dropped below 50% of average!", series.name);
        }
    }
}

// Get verified traffic anomalies
let anomalies = client.get_traffic_anomalies(Some("US"), "7d").await?;
```

### ReliefWeb

ReliefWeb is OCHA's humanitarian information service, aggregating reports, disasters, and updates from 4,000+ sources worldwide. Useful for detecting humanitarian crises, natural disasters, and conflict situations.

```rust
use infrared::data_sources::ReliefWebClient;

let client = ReliefWebClient::new("my-app-name");

// Get ongoing disasters
let disasters = client.get_ongoing_disasters(Some(50)).await?;

// Get disasters for a specific country
let syria_disasters = client.get_disasters(Some("Syria"), Some("ongoing"), None).await?;

// Get disasters by type
let floods = client.get_disasters_by_type("Flood", Some(20)).await?;

// Get recent humanitarian reports
let reports = client.get_reports(Some("Ukraine"), None, Some(10)).await?;

// Search reports by keyword
let conflict_reports = client.search_reports("displacement crisis", Some(25)).await?;

// Get country information
let country = client.get_country("AFG").await?;

// Check disaster status
for item in disasters.data {
    if item.fields.is_ongoing() {
        println!("{}: {} ({})",
            item.fields.name,
            item.fields.type_name().unwrap_or("Unknown"),
            item.fields.country_name().unwrap_or("Unknown")
        );
    }
}
```

### HDX HAPI (Humanitarian Data Exchange)

HDX HAPI provides standardized humanitarian indicators from OCHA, including population displacement, food security, conflict events, and national risk assessments.

```rust
use infrared::data_sources::HdxHapiClient;

let client = HdxHapiClient::new("my-app-name");

// Get humanitarian needs for a country (ISO alpha-3 code)
let needs = client.get_humanitarian_needs("AFG").await?;

// Get refugee statistics
let refugees = client.get_refugees(Some("SYR"), None).await?;

// Get internally displaced persons (IDP) data
let idps = client.get_idps("UKR").await?;

// Get food security (IPC phase) data
let food_security = client.get_food_security("SOM").await?;

// Check for famine conditions
for record in food_security.data {
    if record.is_famine() {
        println!("CRITICAL: Famine conditions in {}", record.location_name);
    } else if record.is_emergency_level() {
        println!("Emergency food insecurity in {}", record.location_name);
    }
}

// Get conflict events (via ACLED data)
let conflicts = client.get_conflict_events("ETH").await?;

// Get national risk indicators
let risks = client.get_national_risk(None).await?;
for risk in risks.data {
    if risk.is_very_high_risk() {
        println!("Very high risk: {} (score: {:?})",
            risk.location_name, risk.overall_risk);
    }
}

// Get food prices for market monitoring
let prices = client.get_food_prices("YEM").await?;

// Get operational presence (who does what where)
let presence = client.get_operational_presence("SDN").await?;
```

### ACLED (Armed Conflict Location & Event Data)

ACLED provides real-time data on political violence and protest activity worldwide. Requires free registration for API access.

```rust
use infrared::data_sources::AcledClient;

// API key required - register at https://acleddata.com/register/
let client = AcledClient::new("your-email@example.com", "your-api-key");

// Get recent events for a country
let events = client.get_events_by_country("Ukraine", Some(100)).await?;

// Get events from last 30 days
let recent = client.get_recent_events("Myanmar", 30, Some(200)).await?;

// Get events by type
let protests = client.get_events_by_type("Sudan", "Protests", Some(100)).await?;
let battles = client.get_events_by_type("Syria", "Battles", Some(100)).await?;

// Get events with fatalities
let lethal = client.get_events_with_fatalities("Afghanistan", 10, Some(100)).await?;

// Get events by date range
let events = client.get_events_by_date_range(
    "Ethiopia",
    "2024-01-01",
    "2024-06-30",
    Some(500)
).await?;

// Get events by region (e.g., Middle East = 8)
let middle_east = client.get_events_by_region(8, Some(200)).await?;

// Analyze response
println!("Total events: {}", events.count);
println!("Total fatalities: {}", events.total_fatalities());

let by_type = events.events_by_type();
for (event_type, count) in by_type {
    println!("{}: {} events", event_type, count);
}

// Filter specific event types
for event in events.data {
    if event.is_civilian_targeting() {
        println!("Civilian targeting: {} in {}", event.location, event.event_date);
    }
}
```

### Data Source Comparison

| Source | Update Frequency | Auth Required | Best For |
|--------|-----------------|---------------|----------|
| IODA   | ~5 minutes      | No            | Internet outage detection, BGP analysis |
| Cloudflare Radar | ~15 minutes | Yes (free API token) | Traffic volume trends |
| ReliefWeb | Near real-time | No (app name only) | Disaster tracking, humanitarian reports |
| HDX HAPI | Daily/Weekly | No (app identifier only) | Humanitarian indicators, food security, displacement |
| ACLED | Daily | Yes (free registration) | Conflict events, protests, violence tracking |

All sources provide **aggregate data only**—no individual tracking.

---

## Architecture

```
src/
├── main.rs          # Entry point, server setup
├── lib.rs           # Library exports
├── model.rs         # Data types (LifeSignal, WarmthStatus, etc.)
├── storage.rs       # SQLite operations
├── aggregation.rs   # Warmth index calculations
├── api.rs           # HTTP handlers
└── data_sources/    # External data source clients
    ├── mod.rs       # Module exports
    ├── ioda.rs      # IODA outage detection client
    ├── cloudflare.rs # Cloudflare Radar traffic client
    ├── reliefweb.rs # ReliefWeb humanitarian data client
    ├── hdx_hapi.rs  # HDX HAPI humanitarian indicators client
    └── acled.rs     # ACLED conflict events client
```

---

## Use Cases

Infrared helps detect:

- ecosystem outages
- natural disasters
- infrastructure failures
- forced displacement
- population collapse
- silent catastrophic events
- unrest or mass evacuation
- system-level mortality signatures

All **without identifying or tracking any human being**.

---

## Philosophy

Infrared is built on two principles:

### **1. Every life has dignity.**
Systems should protect the existence of people without exposing their private actions.

### **2. No population should vanish silently.**
Disappearance—whether caused by disaster, violence, disease, or infrastructure collapse—must be detectable **without** violating privacy.

Infrared shines a faint, gentle beam of truth:
enough to see **life**, never enough to see **individuals**.

---

## License

Apache License 2.0

---

## Contributing

Infrared welcomes contributors who care about:

- privacy-preserving analytics
- ethical technology
- humanitarian safety systems
- resilience and early-warning design
- open-source work grounded in compassion

Please open an issue or submit a PR.

---

## Acknowledgments

Inspired by biological infrared:
warmth is visible, individuals are not.
