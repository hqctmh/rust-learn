# Post Forum Phase 64 Admin Announcement Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin announcement table's "发布公告 / 下线公告 / 重新发布" controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing announcement publishing backend. The dashboard announcement row now carries the target `announcement_id`, and each draft, published, or withdrawn announcement row submits Leptos `ActionForm` controls for publishing or withdrawing the announcement. After each mutation, the server function reloads the admin dashboard so announcement status, available actions, and homepage-operating data reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin announcement publish/withdraw action wiring.
- [x] Expose `announcement_id` on `domain::admin::AdminAnnouncementRow`.
- [x] Map PostgreSQL/in-memory announcement rows into dashboard announcement rows with target IDs.
- [x] Add `publish_admin_announcement(session_id, announcement_id)` server function.
- [x] Add `withdraw_admin_announcement(session_id, announcement_id)` server function.
- [x] Replace static announcement status buttons with `ActionForm` controls.
- [x] Submit hidden `session_id` and `announcement_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after announcement status changes.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml admin_page_publishes_and_withdraws_announcements_with_session_actions -- --nocapture`: failed before implementation with `admin dashboard announcement rows should expose target announcement id fragment: pub announcement_id: Uuid,`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml admin_page_publishes_and_withdraws_announcements_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml announcement_ -- --nocapture`: PASS, 4 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 145 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to publish announcements.
- Covers the PRD admin requirement to withdraw announcements from public visibility.
- Keeps announcement operations session-backed, so only an authenticated admin session can change announcement status.
- Supports the homepage announcement card by making announcement status operational from the management UI.
