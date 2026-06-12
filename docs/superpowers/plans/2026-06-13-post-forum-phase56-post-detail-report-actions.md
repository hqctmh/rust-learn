# Post Forum Phase 56 Post Detail Report Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the post detail page's post and comment report controls submit real session-backed reports instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the post detail report controls to Leptos server actions. `report_post(session_id, target_id, reason, description)` validates the current session and delegates to `AppState::create_report`. `report_comment(session_id, comment_id, reason, description)` validates the current session and delegates to `AppState::report_comment`. Recursive comment rows submit hidden session and comment identifiers with a fixed default reason from the detail UI.

## Tasks

- [x] Add a RED contract test for post and comment report server actions.
- [x] Add `report_post` server function.
- [x] Add `report_comment` server function.
- [x] Validate `session_id`, `target_id`, and `comment_id` through `Uuid::parse_str`.
- [x] Validate the current session through `AppState::current_session`.
- [x] Delegate post reports to `AppState::create_report`.
- [x] Delegate comment reports to `AppState::report_comment`.
- [x] Add page-level `ServerAction<ReportPost>` and `ServerAction<ReportComment>`.
- [x] Replace static report buttons with `ActionForm` controls.
- [x] Render success and failure feedback.
- [x] Preserve recursive comment rendering and report action propagation.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml post_detail_page_reports_posts_and_comments_with_session_actions -- --nocapture`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 138 passed, 2 ignored.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD requirement that logged-in users can report posts from the post detail page.
- Covers the PRD requirement that logged-in users can report comments.
- Reuses the existing report moderation backend so admin report handling remains the single processing path.
