# Post Forum Phase 104 Live Integration E2E

**Goal:** Bring the local runtime dependencies online and prove the ignored live integration tests pass against real PostgreSQL, Redis, NATS, RustFS, and Elasticsearch services.

## Scope

- Start the missing Docker services for Redis, NATS, and Elasticsearch.
- Confirm PostgreSQL migrations are applied.
- Run the ignored live e2e tests.
- Fix the RustFS S3 client configuration so uploads work with the local RustFS endpoint.

## Tasks

- [x] Start Redis, NATS, RustFS, PostgreSQL, and Elasticsearch through Docker Compose.
- [x] Confirm Elasticsearch and NATS health endpoints are reachable.
- [x] Apply SQLx migrations against the local PostgreSQL database.
- [x] Run ignored live e2e tests.
- [x] Add a regression contract requiring explicit S3 region, endpoint, and credentials configuration.
- [x] Configure the `aws-sdk-s3` client from local `.env`/environment values.
- [x] Re-run full ignored live e2e coverage.

## Failure

- `rustfs_object_store_uploads_to_live_rustfs` initially failed with:
  - `A region must be set when sending requests to S3.`
- Root cause:
  - `RustfsObjectStore::from_config` used `aws_config::load_defaults(...)` without explicitly setting a region. The local RustFS path depends on `.env` values such as `AWS_REGION`, `AWS_ENDPOINT_URL`, `AWS_ACCESS_KEY_ID`, and `AWS_SECRET_ACCESS_KEY`.

## Fix

- `RustfsObjectStore::from_config` now uses:
  - `aws_config::defaults(aws_config::BehaviorVersion::latest())`.
  - `Region::new(...)` from `aws_sdk_s3::config`.
  - `.endpoint_url(...)`.
  - `Credentials::new(...)` from `aws_sdk_s3::config`.
- Defaults remain compatible with the local Docker Compose setup:
  - region: `us-east-1`
  - endpoint: `http://127.0.0.1:9000`
  - access key: `rustfsadmin`
  - secret key: `rustfsadmin`

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract rustfs_object_store_adapter_contract_uses_s3_put_object -- --nocapture`: failed before implementation because `src/object_store.rs` did not contain `config::{Credentials, Region}` or the explicit S3 config builder fragments.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract rustfs_object_store_adapter_contract_uses_s3_put_object -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test integration_live rustfs_object_store_uploads_to_live_rustfs -- --ignored --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test integration_live -- --ignored --nocapture`: PASS, 2 passed.

## PRD Coverage

- Proves the PRD runtime integration requirements for real Redis, NATS, RustFS, Elasticsearch, and PostgreSQL wiring.
- Confirms uploaded binary assets can be written to object storage instead of only being validated in memory.
