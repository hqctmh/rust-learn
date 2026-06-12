# Post Forum Phase 60 Admin User Status Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin user-management table's "禁用用户 / 解禁用户" controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing user-admin backend. The dashboard user row now carries the target `user_id`, `/admin?session_id=...` loads the dashboard for the active admin session, and each user row submits a Leptos `ActionForm` to disable or enable the selected account. After the mutation, the server function reloads the admin dashboard so the row status and action labels reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin user status actions.
- [x] Expose `user_id` on `domain::admin::AdminUserRow`.
- [x] Map PostgreSQL/in-memory user-admin rows into dashboard rows with target IDs.
- [x] Let `load_admin_dashboard(session_id)` resolve the admin from the current session when provided.
- [x] Add `disable_admin_user(session_id, target_user_id)` server function.
- [x] Add `enable_admin_user(session_id, target_user_id)` server function.
- [x] Replace static user status buttons with `ActionForm` controls.
- [x] Submit hidden `session_id` and `target_user_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after status changes.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml admin_page_disables_and_enables_users_with_session_actions -- --nocapture`: failed before implementation with `admin dashboard user rows should expose target user id fragment: pub user_id: Uuid,`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml admin_page_disables_and_enables_users_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 141 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to disable users.
- Covers the PRD admin requirement to enable disabled users.
- Keeps the operation session-backed, so only an authenticated admin session can trigger user status changes.
- Reuses the existing user-admin audit backend through `AppState::disable_user` and `AppState::enable_user`.
