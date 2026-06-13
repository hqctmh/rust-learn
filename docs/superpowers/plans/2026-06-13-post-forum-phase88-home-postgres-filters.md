# Post Forum Phase 88 Home Postgres Filters

**Goal:** Make the Postgres-backed homepage apply the normalized `HomeQuery` filters and sort options used by the homepage design.

## Scope

- Apply category and tag filters in the real Postgres homepage list.
- Apply `Unanswered` and `Following` tab semantics in the real Postgres homepage list.
- Apply time-window filtering for today, week, and month.
- Apply homepage sort options while preserving pinned posts first.
- Preserve session-based read markers from `post_reads`.

## Tasks

- [x] Use Context7 to verify SQLx `query_as!` macro behavior for checked row mapping.
- [x] Add RED Postgres runtime coverage for category, tag, unanswered, time, and following filters.
- [x] Add a query-aware `PostgresPostRepository::list_homepage_summaries` method.
- [x] Route `AppState::home_page` through the query-aware Postgres homepage method.
- [x] Verify target tests, homepage tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_applies_category_tag_unanswered_and_sort_filters -- --nocapture`: failed before implementation because unrelated categories were returned.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_filters_following_tab_by_followed_authors -- --nocapture`: failed before implementation because non-followed authors were returned.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_applies_category_tag_unanswered_and_sort_filters -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_applies_time_filter -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_filters_following_tab_by_followed_authors -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 171 passed, 2 ignored.
  - `cargo leptos build`: PASS.
  - `git diff --check`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps returned columns to struct fields by name and rejects unused or missing fields.
- Multi-row checked queries use `fetch_all`.
- Nullable output must map to `Option<T>` unless aliases such as `"field!"` force non-null inference.

## PRD Coverage

- Supports the homepage design requirement that visible topic rows respond to tab, category, tag, time, sort, and pagination controls.
- Keeps `Following` tab gated by the current session while anonymous users still receive a login-required homepage state.
- Keeps pinned topics above sorted results and preserves read/unread markers for logged-in users.
