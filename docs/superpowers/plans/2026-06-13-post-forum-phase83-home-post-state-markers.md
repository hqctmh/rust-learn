# Post Forum Phase 83 Home Post State Markers

**Goal:** Make homepage topic markers derive from backend post state instead of always rendering normal unread markers.

## Scope

- Add `pinned` and `locked` fields to `PostSummary`.
- Select `p.is_pinned` and `p.is_locked` in homepage post SQLx queries.
- Map `PostSummaryRow` and `PostDetailRow` state fields into domain summaries.
- Render homepage markers as pinned first, then locked, then unread.
- Keep new posts and drafts defaulting to unpinned and unlocked.

## Tasks

- [x] Add RED contract coverage for SQL state fields and home marker mapping.
- [x] Extend `PostSummary` and repository row mappings.
- [x] Update SQLx queries to return pinned and locked state.
- [x] Update services, demo store, user-space rows, and test fixtures.
- [x] Verify target tests, homepage tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_topic_marker_is_driven_by_post_state_fields -- --nocapture`: failed before implementation with missing `pinned` and `locked` fields.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_topic_marker_is_driven_by_post_state_fields -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_post_repository_contract_maps_homepage_post_rows -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract home_ -- --nocapture`: PASS, 10 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 165 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` requirement that pinned and locked homepage row states are driven by backend fields.
- Ensures admin pin/lock operations can affect homepage marker rendering through the normal post summary data path.
