# Post Forum Phase 57 Notification Read Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the notification center's "标记已读" and "全部已读" controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice keeps the existing notification domain and repositories intact. The notification page now reads `session_id` from the query string, loads the current user's notification center when a session is present, and submits read operations through Leptos server actions backed by `AppState::mark_notification_read` and `AppState::mark_all_notifications_read`.

## Tasks

- [x] Add a RED contract test for notification center read actions.
- [x] Change `load_notifications_page` to accept optional `session_id`.
- [x] Validate a provided session through `AppState::current_session`.
- [x] Keep demo notification fallback behavior when no session is provided.
- [x] Add `mark_page_notification_read(session_id, notification_id)` server function.
- [x] Add `mark_all_page_notifications_read(session_id)` server function.
- [x] Wire "全部已读" to `ServerAction<MarkAllPageNotificationsRead>`.
- [x] Wire each unread notification row's "标记已读" to `ServerAction<MarkPageNotificationRead>`.
- [x] Disable read actions when no session is available or an action is pending.
- [x] Render read success and failure feedback with the updated unread count.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml notifications_page_marks_read_with_session_actions -- --nocapture`: failed before implementation with `notification page server action missing fragment: pub async fn load_notifications_page(session_id: String)`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml notifications_page_marks_read_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 138 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD requirement that users can view historical notifications from the notification center.
- Covers the PRD requirement that users can mark a single notification as read.
- Covers the PRD requirement that users can mark all notifications as read.
