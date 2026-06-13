# Post Forum Phase 89 Home Postgres Pagination Totals

**Goal:** Make the Postgres-backed homepage return pagination totals from the same filters used by the topic list.

## Scope

- Count published homepage topics with the same category, tag, tab, following, and time filters as the list query.
- Replace seed pagination constants in the Postgres homepage path.
- Keep the demo fallback pagination unchanged for the supplied design seed data.
- Keep sidebar cache behavior isolated from list pagination metadata.

## Tasks

- [x] Add RED Postgres runtime coverage for filtered `pagination.total`, `total_pages`, and label.
- [x] Add `PostgresPostRepository::count_homepage_summaries` using SQLx checked macros.
- [x] Recalculate `HomePagination` in `AppState::home_page` for Postgres mode.
- [x] Verify target test, homepage tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_reports_filtered_pagination_totals -- --nocapture`: failed before implementation with `left: 342`, `right: 3`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_reports_filtered_pagination_totals -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 172 passed, 2 ignored.
  - `cargo leptos build`: PASS.
  - `git diff --check`: PASS.

## PRD Coverage

- Supports `4.1.2` requirement that homepage pagination metadata reflect `page`, `page_size`, filters, and tab state.
- Prevents filtered Postgres pages from showing the design seed constants `342` and `29`.
