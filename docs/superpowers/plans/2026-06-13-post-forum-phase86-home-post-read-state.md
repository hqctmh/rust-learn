# Post Forum Phase 86 Home Post Read State

**Goal:** Drive homepage read/unread topic markers from per-user post read records.

## Scope

- Add persistent `post_reads` records keyed by `(user_id, post_id)`.
- Add `read_by_me` to `PostSummary`.
- Let homepage Postgres summaries left join read records for the current user.
- Mark a post as read when a logged-in user loads the post detail page.
- Pass `session_id` from homepage and post detail routes into server functions.

## Tasks

- [x] Use Context7 to verify SQLx `query!` / `query_as!` macro behavior.
- [x] Add RED source contracts for `post_reads`, repository upsert, and page session propagation.
- [x] Add RED Postgres runtime contract for unread-to-read homepage marker transition.
- [x] Add `post_reads` migration.
- [x] Implement `read_by_me` domain and repository mapping.
- [x] Implement `post_detail_for_user` read tracking.
- [x] Update homepage and post detail loaders to pass session context.
- [x] Verify target tests, related test groups, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_read_records_are_persisted_for_homepage_read_markers -- --nocapture`: failed before implementation with missing `read_by_me` and `post_detail_for_user`.
- GREEN:
  - `sqlx migrate run`: applied `202606130002/migrate post reads`.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_read_records_are_persisted_for_homepage_read_markers -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_post_read_records_drive_homepage_read_markers -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract homepage_component_loads_data_through_server_state -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_loads_route_post_and_comments_through_server_state -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_ -- --nocapture`: PASS, 8 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 167 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps returned columns to struct fields by name and rejects unused or missing fields.
- Nullable columns must map to `Option<T>` unless overridden.
- PostgreSQL aliases like `"field!"` can force non-null inference for computed columns such as `read_by_me`.

## PRD Coverage

- Supports `4.1.2` requirement that homepage read/unread state is based on logged-in user reading records.
- Keeps anonymous homepage loading unpersonalized while allowing session-backed pages to show read markers.
