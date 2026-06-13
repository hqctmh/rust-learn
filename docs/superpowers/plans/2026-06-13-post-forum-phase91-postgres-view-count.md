# Post Forum Phase 91 Postgres View Count

**Goal:** Make Postgres post-detail reads increment `view_count` so homepage view counts reflect detail visits.

## Scope

- Preserve existing in-memory `ForumStore` behavior.
- Increment `posts.view_count` for published Postgres posts when detail data is loaded.
- Return the incremented count in `PostDetail`.
- Let homepage list queries show the updated count through existing Postgres summary queries.

## Tasks

- [x] Add RED Postgres runtime coverage for detail-read view count updates reflected on homepage.
- [x] Add `PostgresPostRepository::increment_view_count` using SQLx checked macros.
- [x] Wire `AppState::post_detail` to increment published post views before returning data.
- [x] Verify target test, post-detail tests, homepage tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_post_detail_increments_postgres_view_count_for_homepage -- --nocapture`: failed before implementation with returned `view_count` still `0`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_post_detail_increments_postgres_view_count_for_homepage -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_ -- --nocapture`: PASS, 9 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 174 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` count consistency requirement that homepage view counts can reflect detail page reads.
- Supports `4.3` requirement that post detail shows and updates view count data.
