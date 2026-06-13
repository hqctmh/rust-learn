# Post Forum Phase 84 Home Active Authors Disabled Filter

**Goal:** Keep disabled users out of the homepage active author sidebar.

## Scope

- Require active author SQL to filter `users.status = 'active'`.
- Apply the same filter to the documented SQL string and the SQLx runtime query.
- Preserve existing reply-count ordering and label formatting.

## Tasks

- [x] Add RED contract coverage for active author SQL status filtering.
- [x] Add `where u.status = 'active'` to active author queries.
- [x] Verify target test, homepage tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows -- --nocapture`: failed before implementation with missing `u.status = 'active'`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 165 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` management support requirement that disabled users should not appear in the active author leaderboard.
