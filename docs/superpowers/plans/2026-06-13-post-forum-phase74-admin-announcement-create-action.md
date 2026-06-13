# Post Forum Phase 74 Admin Announcement Create Action

**Goal:** Make the admin "发布公告" entry create real announcement drafts through a session-backed Leptos server action.

## Scope

- Add `create_admin_announcement(session_id, title, content, announcement_type, pinned)`.
- Route creation through `AppState::create_announcement`.
- Render an admin announcement creation `ActionForm` in the announcement panel.
- Refresh the admin dashboard and show create success/failure feedback.

## Tasks

- [x] Add RED contract coverage for the create-announcement server action and UI form.
- [x] Implement `create_admin_announcement`.
- [x] Add announcement creation controls to the admin announcement panel.
- [x] Keep existing publish, withdraw, update, and push announcement actions working.
- [x] Verify admin page, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_creates_announcements_with_session_action -- --nocapture`: failed before implementation with `admin announcement create server action missing fragment: pub async fn create_admin_announcement(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_creates_announcements_with_session_action -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 15 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 156 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports announcement lifecycle management from the admin UI.
- Supports homepage/sidebar announcement freshness by allowing admins to create drafts before publish/push.
- Keeps admin operations session-backed instead of relying on static demo controls.
