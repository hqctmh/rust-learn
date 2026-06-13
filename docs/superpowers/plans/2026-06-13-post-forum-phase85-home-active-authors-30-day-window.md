# Post Forum Phase 85 Home Active Authors 30 Day Window

**Goal:** Rank homepage active authors by recent reply activity instead of all-time comment totals.

## Scope

- Restrict active author SQL to visible comments created within the last 30 days.
- Keep disabled-user filtering from phase84.
- Preserve current reply count labels and ordering.

## Tasks

- [x] Add RED contract coverage for the 30 day active author window.
- [x] Add `c.created_at >= now() - interval '30 days'` to active author queries.
- [x] Verify target test, homepage tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows -- --nocapture`: failed before implementation with missing `c.created_at >= now() - interval '30 days'`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 165 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` requirement that active authors are ranked by recent reply activity over roughly the last 30 days.
