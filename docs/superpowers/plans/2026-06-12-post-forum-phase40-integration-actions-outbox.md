# Post Forum Phase40 Integration Actions Outbox Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable infrastructure action boundary for PRD cache invalidation, NATS event publication, and Elasticsearch search-index synchronization.

**Architecture:** Add `domain::integrations` as a typed outbox/action contract that describes Redis cache invalidations, NATS event publications, and Elasticsearch index mutations without coupling request handlers to external clients. `ForumStore` records these actions when posts are published, comments change post counters, and announcements are published. PostgreSQL runtime paths persist the same action shape into `integration_outbox`, so later workers can drain it into Redis, NATS, and Elasticsearch.

**Tech Stack:** Rust 2024, Serde DTOs, existing in-memory `ForumStore`, PRD infra services declared in Docker Compose.

---

### Task 1: Integration Action Domain and Store Hooks

**Task Status:** Completed and verified on 2026-06-12.

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/src/domain/integrations.rs`
- Modify: `post/src/domain/mod.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Write the failing test**

Add `infrastructure_integration_actions_cover_cache_events_and_search_index` to `post/tests/phase1_contract.rs`. It asserts:

```rust
let integration_source = std::fs::read_to_string("src/domain/integrations.rs")
    .expect("integration action domain should exist");
let state_source = std::fs::read_to_string("src/state.rs").expect("read state source");

for required in [
    "pub enum IntegrationAction",
    "CacheInvalidate(CacheInvalidation)",
    "NatsPublish(IntegrationEvent)",
    "SearchIndex(SearchIndexMutation)",
    "pub struct SearchIndexDocument",
    "post_published_actions",
    "post_comment_changed_actions",
    "announcement_published_actions",
] {
    assert!(integration_source.contains(required));
}

for required in [
    "integration_actions: Vec<IntegrationAction>",
    "pub fn integration_actions(&self) -> Vec<IntegrationAction>",
    "post_published_actions(&detail)",
    "post_comment_changed_actions(",
    "&post_snapshot",
    "comment.comment_id",
    "announcement_published_actions(&published)",
] {
    assert!(state_source.contains(required));
}
```

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml infrastructure_integration_actions_cover_cache_events_and_search_index
```

Observed: FAIL because `src/domain/integrations.rs` did not exist.

- [x] **Step 3: Write minimal implementation**

Create `post/src/domain/integrations.rs` with:

```rust
pub enum IntegrationAction {
    CacheInvalidate(CacheInvalidation),
    NatsPublish(IntegrationEvent),
    SearchIndex(SearchIndexMutation),
}
```

Add typed payloads for `CacheInvalidation`, `IntegrationEvent`, `SearchIndexMutation`, and `SearchIndexDocument`. Add helper constructors:

```rust
pub fn post_published_actions(post: &PostDetail) -> Vec<IntegrationAction>;
pub fn post_comment_changed_actions(post: &PostDetail, comment_id: Uuid) -> Vec<IntegrationAction>;
pub fn announcement_published_actions(announcement: &AnnouncementItem) -> Vec<IntegrationAction>;
```

Add `pub mod integrations;` to `post/src/domain/mod.rs`.

Add `integration_actions: Vec<IntegrationAction>` to `ForumData`, initialize it in `ForumStore::seeded`, expose `ForumStore::integration_actions`, and record actions from:

- `ForumStore::create_post` when `publish == true`
- `ForumStore::add_comment` after the post comment count changes
- `ForumStore::publish_announcement` after announcement publish succeeds

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml infrastructure_integration_actions_cover_cache_events_and_search_index
```

Observed: PASS, 1 passed.

- [x] **Step 5: Run full verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 104 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

---

### Task 2: PostgreSQL Integration Outbox Persistence

**Task Status:** Completed and verified on 2026-06-12.

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/migrations/202606120002_integration_outbox.sql`
- Create: `post/src/repositories/integrations.rs`
- Modify: `post/src/repositories/mod.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Write the failing test**

Added `postgres_integration_outbox_persists_runtime_actions_with_sqlx_macros` to `post/tests/phase1_contract.rs`. It asserts:

- `integration_outbox` migration exists with pending status, payload, timestamps, and pending index.
- `PostgresIntegrationRepository` exists.
- Repository uses `sqlx::query_as!` / `sqlx::query!`.
- Repository supports `insert_actions`, `list_pending`, and `mark_processed`.
- `AppState` PostgreSQL branches call `PostgresIntegrationRepository::insert_actions`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml postgres_integration_outbox_persists_runtime_actions_with_sqlx_macros
```

Observed: FAIL because `migrations/202606120002_integration_outbox.sql` did not exist.

- [x] **Step 3: Write minimal implementation**

Added `integration_outbox` with:

- `outbox_id`
- `action_kind`
- `subject`
- `aggregate_id`
- `payload`
- `status`
- retry/error metadata
- created/processed timestamps
- pending index

Added `PostgresIntegrationRepository` with SQLx macro-backed:

- `insert_actions`
- `list_pending`
- `mark_processed`

Connected PostgreSQL runtime hooks:

- Published posts persist `post_published_actions`.
- New comments reread the updated post snapshot and persist `post_comment_changed_actions`.
- Published announcements persist `announcement_published_actions`.

- [x] **Step 4: Apply migration to local PostgreSQL for SQLx macro verification**

Run:

```bash
docker exec -i post-postgres psql -U post -d post < post/migrations/202606120002_integration_outbox.sql
```

Observed:

- `CREATE TABLE`
- `CREATE INDEX`

- [x] **Step 5: Run targeted test to verify it passes**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml postgres_integration_outbox_persists_runtime_actions_with_sqlx_macros
```

Observed:

- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml postgres_integration_outbox_persists_runtime_actions_with_sqlx_macros`: PASS, 1 passed.

- [x] **Step 6: Run full verification**

Run:

```bash
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo test --manifest-path post/Cargo.toml`: PASS, 105 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

---

### Task 3: Integration Outbox Drain Worker and Failure Retry State

**Task Status:** Completed and verified on 2026-06-12.

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/repositories/integrations.rs`
- Create: `post/src/integration_worker.rs`
- Modify: `post/src/lib.rs`

- [x] **Step 1: Write the failing test**

Added `integration_outbox_worker_drains_pending_rows_and_records_failures` to `post/tests/phase1_contract.rs`. It asserts:

- `PostgresIntegrationRepository::mark_failed` exists.
- Failed rows increment `attempts`.
- Failed rows record `last_error`.
- Failed rows remain `pending` until `max_attempts`, then move to `failed`.
- `IntegrationOutboxWorker` drains pending rows through an `IntegrationActionHandler`.
- Successful rows call `mark_processed`; failed rows call `mark_failed`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml integration_outbox_worker_drains_pending_rows_and_records_failures
```

Observed: FAIL because `src/integration_worker.rs` did not exist.

- [x] **Step 3: Write minimal implementation**

Added:

- `PostgresIntegrationRepository::mark_failed`
- `IntegrationActionHandler`
- `IntegrationOutboxWorker`
- `IntegrationDrainReport`
- SSR module export from `lib.rs`

The worker loads pending rows, calls the handler, marks successes processed, and records failures with retry metadata.

- [x] **Step 4: Verify worker behavior against PostgreSQL**

Added `integration_outbox_worker_processes_successes_and_records_retriable_failures`.

The test inserts success and failure actions into PostgreSQL, drains once with a test handler, then verifies:

- successful action is no longer pending
- failed action remains pending
- failed action has `attempts = 1`
- failed action stores `last_error = "handler failed"`

During full-suite verification, the first version assumed only 2 rows would be scanned. That failed when other parallel PostgreSQL tests inserted pending outbox rows concurrently. Root cause was test isolation, not worker behavior. The assertion was corrected to check the exact state of the test-owned rows while allowing the worker to scan additional pending rows.

- [x] **Step 5: Run full verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml integration_outbox_worker_processes_successes_and_records_retriable_failures
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml integration_outbox_worker_processes_successes_and_records_retriable_failures`: PASS, 1 passed.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 107 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

---

### Task 4: Runtime Redis, NATS, and Elasticsearch Handler Boundary

**Task Status:** Completed and verified on 2026-06-12.

**Files:**
- Modify: `post/Cargo.toml`
- Modify: `post/Cargo.lock`
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/repositories/integrations.rs`
- Create: `post/src/integration_handler.rs`
- Modify: `post/src/lib.rs`

- [x] **Step 1: Confirm third-party API usage**

Checked current docs and crate metadata:

- `redis` 1.2.2: async usage through `Client::open(...)`, `get_multiplexed_async_connection().await`, and `redis::cmd(...).query_async(...)`; async Tokio support uses the `tokio-comp` feature.
- `async-nats` 0.49.1: connect with `async_nats::connect(url).await`, publish with `client.publish(subject, payload.into()).await`, and flush with `client.flush().await`.
- `elasticsearch` 9.1.0-alpha.1: create a client with `Transport::single_node(...)` and `Elasticsearch::new(...)`; index/delete use client request builders.
- `serde_json`: use `serde_json::json!` for compact JSON payload construction and `serde_json::from_str` for parsing.

Local toolchain evidence:

- `rustc --version`: `rustc 1.96.0`, which satisfies `redis` and `async-nats` MSRV requirements observed from `cargo info`.

- [x] **Step 2: Write the failing test**

Added `runtime_integration_handler_uses_real_redis_nats_and_elasticsearch_clients` to `post/tests/phase1_contract.rs`. It asserts:

- Manifest includes `redis`, `async-nats`, `elasticsearch`, and `serde_json` under SSR dependencies.
- Outbox payloads are generated as JSON, not ad-hoc text.
- Runtime handler initializes Redis, NATS, and Elasticsearch clients.
- Runtime handler implements `IntegrationActionHandler`.
- Runtime handler dispatches cache invalidation, NATS publish, and search index upsert/delete.

- [x] **Step 3: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml runtime_integration_handler_uses_real_redis_nats_and_elasticsearch_clients
```

Observed: FAIL because `src/integration_handler.rs` did not exist.

- [x] **Step 4: Write minimal implementation**

Added SSR-only dependencies:

- `redis = { version = "1.2.2", features = ["tokio-comp"], optional = true }`
- `async-nats = { version = "0.49.1", optional = true }`
- `elasticsearch = { version = "9.1.0-alpha.1", optional = true }`
- `serde_json = { version = "1.0", optional = true }`

Updated `PostgresIntegrationRepository` payload construction to write compact JSON for:

- `cache_invalidate`
- `nats_publish`
- `search_upsert`
- `search_delete`

Added `RuntimeIntegrationHandler`:

- Connects Redis/NATS/Elasticsearch from `RuntimeConfig`.
- Deletes Redis cache keys, including wildcard keys via `KEYS` then `DEL`.
- Publishes NATS events with the outbox subject and JSON payload.
- Upserts Elasticsearch documents for post search fields.
- Deletes Elasticsearch documents for delete mutations.
- Implements the existing `IntegrationActionHandler` trait so it can be used by `IntegrationOutboxWorker`.

- [x] **Step 5: Run targeted verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml runtime_integration_handler_uses_real_redis_nats_and_elasticsearch_clients
```

Observed:

- Initial compile succeeded, but the source contract failed because `rustfmt` split `get_multiplexed_async_connection().await` across lines.
- Contract was adjusted to check API fragments without depending on one-line formatting.
- `cargo test --manifest-path post/Cargo.toml runtime_integration_handler_uses_real_redis_nats_and_elasticsearch_clients`: PASS, 1 passed.

- [x] **Step 6: Run full verification**

Run:

```bash
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo test --manifest-path post/Cargo.toml`: PASS, 108 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- First sandboxed `cargo leptos build` failed because cargo could not unpack `linux-raw-sys` into `~/.cargo/registry` under sandbox permissions.
- Retried `cargo leptos build` with approved non-sandbox execution: PASS.

---

### Task 5: SSR Startup Scheduling for the Integration Outbox Worker

**Task Status:** Completed and verified on 2026-06-12.

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/state.rs`
- Create: `post/src/integration_runtime.rs`
- Modify: `post/src/lib.rs`
- Modify: `post/src/main.rs`

- [x] **Step 1: Confirm runtime scheduling API**

Checked Tokio docs:

- `tokio::spawn` starts a background future immediately and returns `JoinHandle`; it must be called inside a Tokio runtime.
- `tokio::time::interval(Duration)` is the recommended API for periodic work because it measures ticks against the schedule instead of simply sleeping after each loop.

- [x] **Step 2: Write the failing test**

Added `integration_outbox_worker_runtime_starts_from_main_when_enabled` to `post/tests/phase1_contract.rs`. It asserts:

- `RuntimeConfig` exposes:
  - `integration_worker_enabled`
  - `integration_worker_batch_size`
  - `integration_worker_max_attempts`
  - `integration_worker_interval_millis`
- Environment variables are supported:
  - `INTEGRATION_WORKER_ENABLED`
  - `INTEGRATION_WORKER_BATCH_SIZE`
  - `INTEGRATION_WORKER_MAX_ATTEMPTS`
  - `INTEGRATION_WORKER_INTERVAL_MILLIS`
- `integration_runtime` uses `RuntimeIntegrationHandler`, `IntegrationOutboxWorker`, `tokio::spawn`, `tokio::time::interval`, and `worker.drain_once`.
- `main.rs` builds `RuntimeConfig::from_env()` and calls `spawn_integration_outbox_worker(...)`.

- [x] **Step 3: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml integration_outbox_worker_runtime_starts_from_main_when_enabled
```

Observed: FAIL because `src/integration_runtime.rs` did not exist.

- [x] **Step 4: Write minimal implementation**

Added worker config to `RuntimeConfig`:

- disabled by default with `INTEGRATION_WORKER_ENABLED=false`
- default batch size: `50`
- default max attempts: `3`
- default interval: `1000ms`

Added `spawn_integration_outbox_worker(pool, config)`:

- returns `None` when worker is disabled
- returns `None` with an error log when enabled but no PostgreSQL pool exists
- creates `RuntimeIntegrationHandler`
- creates `IntegrationOutboxWorker`
- runs a periodic Tokio interval loop and drains pending outbox rows

Updated `main.rs` to build `RuntimeConfig::from_env()`, create the database pool from that config, and call `spawn_integration_outbox_worker(db.clone(), runtime_config)`.

- [x] **Step 5: Run targeted verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml integration_outbox_worker_runtime_starts_from_main_when_enabled
```

Observed:

- Initial source-contract assertion was too strict about `Duration::from_millis(...)` being on one line after `rustfmt`; the test was adjusted to check the actual API fragments.
- `cargo test --manifest-path post/Cargo.toml integration_outbox_worker_runtime_starts_from_main_when_enabled`: PASS, 1 passed.

- [x] **Step 6: Run full verification**

Run:

```bash
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo test --manifest-path post/Cargo.toml`: PASS, 109 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

---

### Task 6: Live External-Service E2E Test Entry for Redis, NATS, and Elasticsearch

**Task Status:** Completed as a runnable ignored test and script on 2026-06-12. Live execution is pending Docker image availability.

**Files:**
- Modify: `post/Cargo.toml`
- Modify: `post/Cargo.lock`
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/tests/integration_live.rs`
- Create: `post/scripts/run-integration-live.sh`

- [x] **Step 1: Write the failing contract test**

Added `integration_outbox_live_e2e_test_is_available_for_external_services` to `post/tests/phase1_contract.rs`. It asserts:

- `tests/integration_live.rs` exists.
- The live test is `#[ignore]`.
- The live test seeds Redis with `SET` and verifies deletion with `EXISTS`.
- The live test subscribes to NATS and waits for a published message.
- The live test inserts outbox rows through `PostgresIntegrationRepository::insert_actions`.
- The live test drains rows with `IntegrationOutboxWorker` and `RuntimeIntegrationHandler`.
- The live test searches Elasticsearch with `SearchParts::Index`.
- The live test includes delete cleanup with `DeleteParts::IndexId`.
- `scripts/run-integration-live.sh` starts required Docker Compose services and runs the ignored test.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml integration_outbox_live_e2e_test_is_available_for_external_services
```

Observed:

- Initial run failed before reaching the new contract because SQLx macros could not connect to PostgreSQL while Docker/OrbStack was down.
- `orb start` failed in sandbox with `chmod /Users/mah2/.orbstack/run: operation not permitted`.
- Retried `orb start` outside sandbox; it timed out, but `orb status` later reported `Running` and Docker API recovered.
- Postgres and RustFS containers came back healthy.

- [x] **Step 3: Attempt to start external services**

Run:

```bash
docker compose -f post/docker-compose.yml up -d redis nats elasticsearch
```

Observed:

- Both sandboxed and non-sandboxed attempts failed while pulling images.
- Docker registry DNS inside the VM timed out:
  `lookup registry-1.docker.io on 0.250.250.200:53: i/o timeout`.

- [x] **Step 4: Add live e2e test and script**

Added `post/tests/integration_live.rs`:

- Connects to PostgreSQL, Redis, NATS, and Elasticsearch.
- Seeds a Redis key and verifies the handler deletes it.
- Subscribes to a unique NATS subject and verifies the handler publishes a message.
- Inserts a search upsert outbox row and verifies the document becomes searchable in Elasticsearch.
- Inserts a search delete row and drains it for cleanup.
- Is marked `#[ignore]` so default test runs are not blocked by Docker services.

Added `post/scripts/run-integration-live.sh`:

- Starts `postgres`, `redis`, `nats`, and `elasticsearch`.
- Waits for Docker health checks.
- Applies the integration outbox migration.
- Runs `cargo test --test integration_live -- --ignored --nocapture`.

- [x] **Step 5: Run targeted and default verification**

Run:

```bash
cargo test --manifest-path post/Cargo.toml integration_outbox_live_e2e_test_is_available_for_external_services
cargo test --manifest-path post/Cargo.toml integration_outbox_worker_processes_successes_and_records_retriable_failures
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Observed:

- `cargo test --manifest-path post/Cargo.toml integration_outbox_live_e2e_test_is_available_for_external_services`: PASS, 1 passed.
- `cargo test --manifest-path post/Cargo.toml integration_outbox_worker_processes_successes_and_records_retriable_failures`: PASS, 1 passed.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 110 passed, 1 ignored.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

Live ignored test was not executed because Redis/NATS/Elasticsearch images could not be pulled in the current Docker environment.

## Self-Review

- Covers PRD requirements that content changes trigger cache invalidation or asynchronous refresh.
- Covers PRD requirements that post changes update the search index with title, summary, body, tags, category, and author fields.
- Covers NATS event boundary for post publish, comment creation, and announcement publish without blocking request handlers on external brokers.
- Adds PostgreSQL-backed outbox persistence for AppState runtime paths.
- Adds a reusable outbox drain worker with success and failure retry state.
- Adds a concrete Redis/NATS/Elasticsearch runtime handler behind the outbox worker boundary.
- Adds optional SSR startup scheduling for the outbox worker.
- Adds a live Redis/NATS/Elasticsearch ignored e2e test and script for external-service verification.
- Leaves actual live e2e execution pending until Docker registry DNS/image pulls are available.
