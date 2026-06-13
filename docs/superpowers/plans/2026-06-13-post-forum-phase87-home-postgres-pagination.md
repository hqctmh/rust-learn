# Post Forum Phase 87 Home Postgres Pagination

**Goal:** Make the Postgres-backed homepage respect `HomeQuery` pagination instead of always loading a fixed 50 posts.

## Scope

- Use normalized `home.query.page_size` as the Postgres homepage query limit.
- Use normalized `home.query.page` to calculate SQL offset.
- Preserve existing read-state personalization and sidebar loading.

## Tasks

- [x] Add RED Postgres runtime coverage for `page_size=1`.
- [x] Pass query-derived `limit` and `offset` into `PostgresPostRepository::list_published_summaries_for_user`.
- [x] Verify target test, homepage tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_respects_query_pagination -- --nocapture`: failed before implementation with `left: 50`, `right: 1`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_respects_query_pagination -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 168 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` list query requirement that homepage pagination parameters affect returned topics.
- Keeps the design稿 default of 12 topics while allowing page-size changes to be honored by the real Postgres data path.
