# Post Forum Phase 65 Admin Taxonomy Status Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin category/tag status controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing taxonomy backend. The dashboard category and tag rows now carry target IDs, and the category/tag tables submit Leptos `ActionForm` controls for enabling or disabling taxonomy records. After each mutation, the server function reloads the admin dashboard so status, available actions, and homepage-operating taxonomy data reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin taxonomy status action wiring.
- [x] Expose `category_id` on `domain::admin::AdminCategoryRow`.
- [x] Expose `tag_id` on `domain::admin::AdminTagRow`.
- [x] Map PostgreSQL/in-memory taxonomy rows into dashboard rows with target IDs.
- [x] Add `enable_admin_category(session_id, category_id)` server function.
- [x] Add `disable_admin_category(session_id, category_id)` server function.
- [x] Add `enable_admin_tag(session_id, tag_id)` server function.
- [x] Add `disable_admin_tag(session_id, tag_id)` server function.
- [x] Replace static taxonomy status buttons with `ActionForm` controls.
- [x] Submit hidden `session_id`, `category_id`, and `tag_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after taxonomy status changes.

## Verification Evidence

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_toggles_taxonomy_status_with_session_actions -- --nocapture`: failed before implementation with `admin taxonomy rows should expose target ids fragment: pub category_id: Uuid,`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_toggles_taxonomy_status_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml taxonomy_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 146 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to enable/disable categories that drive homepage category chips and sidebar counts.
- Covers the PRD admin requirement to enable/disable tags that drive homepage hot tags and post tag filters.
- Keeps taxonomy operations session-backed, so only an authenticated admin session can change category or tag status.
