Purrfect, Rust it is. 🐈‍⬛🦀

Here’s a **ready-to-paste Claude prompt** that’s already Rust-specific and tuned for the Infrared idea we just nailed down.

---

### Prompt to Claude (Rust MVP for Infrared)

You are a senior Rust backend engineer.
Help me build an MVP of a system called **Infrared**.

---

## 1. Concept (do not change)

Infrared is a **privacy-preserving life-signal tracker**.

**Core idea:**

> Infrared tracks *signs of life* at large scale **without tracking individuals**.
> It measures **aggregate “warmth”** (activity / presence) and detects large-scale drops or disappearances, but **never stores identity, location, or other personal data**.

Think: “Is there still life here?” — at a **bucket / region / system** level, not per person.

Infrared must be:

* **Aggregate-only**: no per-user tracking, no identifiers.
* **Privacy-first**: no PII, no IPs, no device IDs, no GPS.
* **Legally boring**: if the DB leaks, it reveals nothing about any specific human.
* **Simple**: a single Rust binary, easy to run.

---

## 2. MVP requirements

Build a single Rust crate (binary) that:

### 2.1. Data model

Define a core event type, e.g.:

```rust
/// A single "life signal" event.
///
/// Represents anonymous evidence that "something is alive" in a given bucket.
pub struct LifeSignal {
    /// A coarse bucket identifier such as "region:north", "city:A", "cluster:web-01".
    /// This is defined by configuration/integration, never by end-users.
    pub bucket: String,

    /// Server-side timestamp when the signal was recorded (UTC).
    pub timestamp: DateTime<Utc>,

    /// Optional weight (e.g. number of entities represented by this signal).
    /// Default = 1.
    pub weight: i32,
}
```

Strict constraints:

* No usernames, IPs, device IDs, emails, GPS, etc.
* Only `bucket`, `timestamp`, `weight`.

### 2.2. HTTP API (axum preferred)

Use **axum** (with tokio) to expose:

1. `POST /signal`

   * Body JSON: `{ "bucket": "zone-a", "weight": 1 }` (weight optional, defaults to 1).
   * Server sets `timestamp = now()`.
   * Append to storage and update any in-memory cache if needed.
   * Return `202 Accepted` or `200 OK`.

2. `GET /warmth`
   Query params:

   * `bucket=<bucket>` (required)
   * `window_minutes=<n>` (optional, default e.g. 10)

   Response JSON should include:

   * `bucket`
   * `window_minutes`
   * `current_window_total` (sum of weights in the latest window)
   * `recent_average` (average of previous N windows)
   * `status`: `"alive" | "stressed" | "collapsing" | "dead"`

   Use simple heuristic thresholds, e.g.:

   * `alive`: current ≥ 0.8 × recent_average
   * `stressed`: 0.2–0.8 × recent_average
   * `collapsing`: 0 < current < 0.2 × recent_average
   * `dead`: current == 0 while recent_average > 0

3. `GET /alerts/recent`
   Query params:

   * `minutes=<m>` (lookback window)

   For each bucket, detect if it appears to be in `"collapsing"` or `"dead"` state over the last `m` minutes. Return a list of objects with:

   * `bucket`
   * `status`
   * `last_seen_timestamp`
   * `recent_average`
   * `message` (human-readable summary)

No fancy auth for MVP; you can add a simple shared-secret header (like `X-Infrared-Key`) for `POST /signal` if you want, but keep it minimal.

### 2.3. Storage

Use **SQLite** with `sqlx` (or `sea-query + sqlx`, your choice, but keep it simple).

Schema (MVP):

* `life_signals(bucket TEXT NOT NULL, ts INTEGER NOT NULL, weight INTEGER NOT NULL)`

Where `ts` is a Unix timestamp (seconds or milliseconds).

Requirements:

* Append-only writes.
* Simple queries by bucket + time range.
* No migrations complexity; just create the table if it doesn’t exist.

You may optionally implement a simple “aggregation” query that bins by time windows using SQL (e.g. floor(ts / window_size)).

### 2.4. Privacy guarantees baked into code

Enforce privacy at design level:

* The HTTP handler for `POST /signal` must **not** log client IPs or headers.
  Use structured logging but only for:

  * bucket
  * weight
  * maybe high-level diagnostics (latency, success/fail)

* No fields anywhere that carry identity / IP / device / GPS.

Add Rust doc comments and inline comments that clearly say:

* why certain fields are forbidden,
* what makes Infrared safe if logs/DB leak.

---

## 3. Project structure

Propose and implement a structure like:

* `src/main.rs`

  * CLI args or env-based config (port, DB path).
  * Start tokio runtime.
  * Initialize DB connection pool.
  * Build axum router and serve.

* `src/model.rs`

  * `LifeSignal`
  * `WarmthStatus` enum
  * Data structs for API responses.

* `src/storage.rs`

  * SQLite setup.
  * Functions:

    * `insert_life_signal(...)`
    * `query_bucket_window(bucket, window_minutes, now)`
    * `compute_recent_average(bucket, window_minutes, num_windows, now)`

* `src/api.rs`

  * Axum handlers:

    * `post_signal`
    * `get_warmth`
    * `get_recent_alerts`

* `src/aggregation.rs`

  * Logic for computing `WarmthIndex` + status.
  * Reusable functions used by `api.rs`.

Use `anyhow::Result` or a small custom error enum for simplicity.

---

## 4. Dependencies

Set up `Cargo.toml` with (you can adjust versions):

* `tokio` (full features)
* `axum`
* `serde`, `serde_json`
* `serde_derive` or `serde` with `derive` feature
* `chrono`
* `sqlx` with `sqlite` + `runtime-tokio` features
* `tracing`, `tracing-subscriber`
* `thiserror` or `anyhow` for errors

---

## 5. Tests

Add:

1. **Unit tests** for aggregation logic (`WarmthStatus`):

   * e.g. simulate windows and verify `"alive" | "stressed" | "collapsing" | "dead"` classification.

2. **Integration test** (optional but nice):

   * Spin up the app using an in-memory SQLite DB.
   * POST some signals for a bucket.
   * Call `GET /warmth` and assert the response matches expectations.

---

## 6. README

Generate a `README.md` that:

* Briefly explains what Infrared is (as above: population-level life signals, no identity).
* Documents the API:

  * `POST /signal`
  * `GET /warmth`
  * `GET /alerts/recent`
* Shows basic usage with `curl`.
* States the privacy constraints explicitly.

---

## 7. How to respond

1. Start by summarizing your understanding of Infrared in 3–4 sentences.
2. Propose the module structure and key types.
3. Then generate:

   * `Cargo.toml`
   * `src/main.rs`
   * `src/model.rs`
   * `src/storage.rs`
   * `src/aggregation.rs`
   * `src/api.rs`
   * basic tests
   * and a draft `README.md`.

You can provide the code in sections, clearly labeled per file.

Please prioritize **clarity**, **simplicity**, and **privacy guarantees** over micro-optimizations.

---

That’s the full spec. Now act as if you’re implementing this Rust MVP for Infrared from scratch.

---

If you’d like, I can next help you:

* turn this into an actual Rust repo layout,
* or tighten this prompt even more for a “one-shot Claude codegen” run.
