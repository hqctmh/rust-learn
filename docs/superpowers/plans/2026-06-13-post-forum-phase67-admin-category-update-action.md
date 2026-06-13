# Post Forum Phase 67 Admin Category Update Action

**Goal:** Make the admin category table's edit/sort controls submit a real session-backed update action so homepage category color and ordering can be operated from the admin page.

## Scope

- Add a contract test for category update server action and admin UI form wiring.
- Add `update_admin_category(session_id, category_id, name, color, sort_order)`.
- Replace static category edit/sort buttons with an `ActionForm`.
- Refresh the admin dashboard from the action result and show success/failure feedback.

## Tasks

- [x] Add RED contract coverage for category update form and server function.
- [x] Implement the server function through `AppState::update_category`.
- [x] Render row-level inputs for category name, color, and sort order.
- [x] Keep existing enable/disable actions working.
- [x] Verify target test, taxonomy tests, full test suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_categories_with_session_action -- --nocapture`: failed before implementation with `admin category update server action missing fragment: pub async fn update_admin_category(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_categories_with_session_action -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_merges_tags_with_session_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml taxonomy_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 148 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `4.1.2` management-side requirement: admins can configure category color and category sort order for homepage category cards and post row category badges.
- Supports `5.6` category management requirement: edit category and adjust category sorting.
