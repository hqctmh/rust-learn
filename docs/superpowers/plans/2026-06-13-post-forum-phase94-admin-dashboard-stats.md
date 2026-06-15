# Post Forum Phase 94 Admin Dashboard Stats

**Goal:** Back the admin dashboard statistics required by PRD 5.10 with Postgres aggregate data instead of partial static counts.

## Scope

- Add a dedicated `PostgresAdminStatsRepository`.
- Aggregate total users, today's new users, total posts, today's new posts, total comments, today's new comments, and total likes.
- Surface current online connection count from runtime WebSocket state.
- Surface hot post and hot tag summary cards from Postgres data.
- Keep dashboard UI data shape unchanged through `AdminStat`.

## Tasks

- [x] Add RED runtime coverage for all PRD 5.10 dashboard stat labels.
- [x] Add RED source coverage requiring a dedicated admin stats repository.
- [x] Implement SQLx checked aggregate queries for dashboard summary, hot post, and hot tag.
- [x] Wire `AppState::admin_dashboard` to the repository and runtime online connection state.
- [x] Verify target tests, dashboard tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_dashboard_stats_are_loaded_from_dedicated_repository -- --nocapture`: failed before implementation because `repositories::admin_stats` did not exist.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_admin_dashboard_aggregates_postgres_runtime_data -- --nocapture`: failed before implementation because dashboard stats did not include `今日新增用户` and the rest of PRD 5.10 stat labels.
- GREEN:
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_dashboard_stats_are_loaded_from_dedicated_repository -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_admin_dashboard_aggregates_postgres_runtime_data -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_dashboard_ -- --nocapture`: PASS, 3 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 178 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps selected columns to struct fields by name and validates missing or unused fields at compile time.
- Aggregate expressions use explicit checked aliases such as `"user_count!"` and `"hot_score!"` so SQLx treats them as non-null.
- Optional dashboard widgets use `fetch_optional` for hot post and hot tag rows.

## PRD Coverage

- Supports `5.10` admin dashboard data statistics.
- Moves dashboard statistics from partial `state.rs` list lengths to dedicated Postgres aggregate queries.
