# Post Forum Phase 90 Home Last Reply Data

**Goal:** Drive the homepage `last reply` column from the latest visible comment instead of post author fallback text.

## Scope

- Extend `PostSummary` with latest reply author, avatar, and timestamp fields.
- Load latest visible comment per post through SQLx checked Postgres queries.
- Render homepage last-reply author and relative time from runtime data.
- Preserve fallback behavior for posts without comments.

## Tasks

- [x] Add RED Postgres runtime coverage for homepage latest visible comment mapping.
- [x] Add latest reply fields to `PostSummary` and repository row mapping.
- [x] Use `left join lateral` to fetch the latest visible comment without N+1 queries.
- [x] Use SQLx nullable aliases for lateral fields.
- [x] Verify target tests, homepage tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_uses_latest_visible_comment_for_last_reply -- --nocapture`: failed before implementation with homepage last reply author equal to the post author.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_homepage_postgres_uses_latest_visible_comment_for_last_reply -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_create_post_persists_to_postgres_and_reads_back -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 173 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Notes

- SQLx inferred nullable lateral columns as non-null until aliases such as `"last_reply_author_name?"` were added.
- A full-suite run caught the generic published-list path separately from the homepage-specific path.

## PRD Coverage

- Supports `4.1.2` requirement that each homepage row includes latest reply user and timestamp data.
- Keeps the query N+1-free by loading latest reply data inside the list SQL.
