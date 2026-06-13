# Post Forum Phase 68 Admin Tag Update Action

**Goal:** Make the admin tag table's edit/sort control submit a real session-backed update action so homepage hot tags and tag navigation can be operated from the admin page.

## Scope

- Add a contract test for tag update server action and admin UI form wiring.
- Add `update_admin_tag(session_id, tag_id, name, sort_order)`.
- Replace the static tag edit button with an `ActionForm`.
- Refresh the admin dashboard from the action result and show success/failure feedback.

## Tasks

- [x] Add RED contract coverage for tag update form and server function.
- [x] Implement the server function through `AppState::update_tag`.
- [x] Render row-level inputs for tag name and sort order.
- [x] Keep existing merge, enable, and disable tag actions working.
- [x] Verify target tests, taxonomy tests, full test suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_tags_with_session_action -- --nocapture`: failed before implementation with `admin tag update server action missing fragment: pub async fn update_admin_tag(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_tags_with_session_action -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates -- --nocapture`: PASS, 2 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_merges_tags_with_session_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml taxonomy_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 149 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` management-side requirement: admins can configure tag sorting and maintain tags used by homepage hot tag cards and post row tag pills.
- Supports `5.6` tag management requirement: edit tag and adjust tag sorting.
