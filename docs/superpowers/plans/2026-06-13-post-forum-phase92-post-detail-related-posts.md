# Post Forum Phase 92 Post Detail Related Posts

**Goal:** Render post detail related posts from real runtime data instead of hard-coded search links.

## Scope

- Add `related_posts` to `PostDetailPageData`.
- Load related posts through `AppState` when the detail server function runs.
- Support demo fallback through `ForumStore`.
- Support Postgres runtime through a SQLx checked query that ranks shared-tag posts before same-category fallbacks.
- Render related post links to `/posts/{post_id}` with live reply counts.

## Tasks

- [x] Add RED source coverage that forbids hard-coded related links in the detail page.
- [x] Add RED Postgres runtime coverage for same-tag related posts and self-exclusion.
- [x] Add `AppState::related_posts_for_post` and `ForumStore::related_posts_for_post`.
- [x] Add `PostgresPostRepository::list_related_summaries`.
- [x] Wire `load_post_detail_page` and `PostDetailView` to `related_posts`.
- [x] Verify target tests, post-detail tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_renders_related_posts_from_page_data -- --nocapture`: failed before implementation because `PostDetailPageData` had no `related_posts` and the page hard-coded three links.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_post_detail_related_posts_use_shared_tags -- --nocapture`: failed before implementation because `AppState::related_posts_for_post` did not exist.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_renders_related_posts_from_page_data -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_post_detail_related_posts_use_shared_tags -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_ -- --nocapture`: PASS, 11 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 176 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps output columns to struct fields by name and rejects unused or missing fields.
- Nullable outer-join columns should be marked with aliases such as `"field?"`.
- Multi-row list queries use `fetch_all`.

## PRD Coverage

- Supports `4.3` requirement that post detail shows related posts.
- Moves related recommendations from static UI text to server-provided runtime data.
