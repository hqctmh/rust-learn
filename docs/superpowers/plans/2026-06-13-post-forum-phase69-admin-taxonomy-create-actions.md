# Post Forum Phase 69 Admin Taxonomy Create Actions

**Goal:** Make the admin category and tag management panels create real taxonomy rows through session-backed Leptos actions instead of relying only on API endpoints.

## Scope

- Add a contract test for category and tag creation forms on the admin page.
- Add `create_admin_category(session_id, name, color, sort_order)`.
- Add `create_admin_tag(session_id, name, sort_order)`.
- Render category and tag creation `ActionForm`s in the management panels.
- Refresh the admin dashboard from action results and show success/failure feedback.

## Tasks

- [x] Add RED contract coverage for category and tag create server actions and forms.
- [x] Implement create server functions through `AppState::create_category` and `AppState::create_tag`.
- [x] Render category creation inputs for name, color, and sort order.
- [x] Render tag creation inputs for name and sort order.
- [x] Keep existing taxonomy update, status, and merge actions working.
- [x] Verify target test, admin page action tests, taxonomy tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_creates_taxonomy_items_with_session_actions -- --nocapture`: failed before implementation with `admin taxonomy create server action missing fragment: pub async fn create_admin_category(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_creates_taxonomy_items_with_session_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 10 passed.
  - `cargo test --manifest-path post/Cargo.toml taxonomy_ -- --nocapture`: PASS, 6 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 150 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` management-side requirement: category and tag data used by homepage sidebar cards and post row badges can be created from the admin page.
- Supports `5.6` taxonomy management requirement: create category and create tag.
