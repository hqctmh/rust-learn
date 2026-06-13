# Post Forum Phase 66 Admin Tag Merge Action

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin tag table's "合并标签" control submit a real session-backed merge action instead of remaining a static button.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing tag merge backend. Each tag row now renders a merge form with the current tag as the source and a selectable target tag. After a successful merge, the server function reloads the admin dashboard so tag usage counts, enabled status, and homepage-operating hot tag data reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin tag merge action wiring.
- [x] Add `merge_admin_tag(session_id, source_tag_id, target_tag_id)` server function.
- [x] Parse source and target tag IDs from form fields.
- [x] Call `AppState::merge_tag` with `MergeTagRequest`.
- [x] Pass current dashboard tag rows into each `TagRow` as merge target options.
- [x] Render a `target_tag_id` select that excludes the current source tag.
- [x] Replace the static "合并标签" button with an `ActionForm`.
- [x] Disable the merge button while pending or when no session is available.
- [x] Render success and failure feedback after tag merge changes.

## Verification Evidence

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_merges_tags_with_session_action -- --nocapture`: failed before implementation with `admin tag merge server action missing fragment: pub async fn merge_admin_tag(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_merges_tags_with_session_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml taxonomy_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 147 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to merge tags.
- Keeps tag merge operations session-backed, so only an authenticated admin session can merge tags.
- Supports homepage hot tag quality by making duplicate or noisy tags operationally mergeable from the management UI.
