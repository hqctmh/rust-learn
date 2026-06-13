# Post Forum Phase 73 Admin Announcement Edit Push Actions

**Goal:** Make the admin announcement table's "编辑" and "推送公告" controls submit real session-backed actions instead of static buttons.

## Scope

- Add an `UpdateAnnouncementRequest` domain request and service update path.
- Add `update_announcement` and `push_announcement` to `AppState` and `ForumStore`.
- Persist announcement edits through `PostgresAnnouncementRepository::update_announcement` using `sqlx::query!`.
- Push published announcements to announcement recipients and emit integration actions.
- Render admin announcement edit/push controls as `ActionForm` instances.

## Tasks

- [x] Add RED contract coverage for admin announcement edit/push server actions and UI forms.
- [x] Add RED behavior coverage for updating and pushing existing announcements.
- [x] Implement announcement update validation and field normalization.
- [x] Implement Postgres update and recipient lookup with checked SQL macros.
- [x] Wire Leptos server actions and feedback messages.
- [x] Verify announcement tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_edits_and_pushes_announcements_with_session_actions -- --nocapture`: failed before implementation with missing `UpdateAnnouncementRequest`, `ForumStore::update_announcement`, and `ForumStore::push_announcement`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_edits_and_pushes_announcements_with_session_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract announcement_contract_updates_and_pushes_existing_announcements -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml announcement -- --nocapture`: PASS, 7 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 155 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `5.4` content governance and announcement operations by making admin announcement edits and pushes executable from the UI.
- Supports homepage/sidebar announcement freshness by allowing admins to update announcement content before publishing or republishing.
- Supports notification and integration requirements by routing announcement push through notification records and integration actions.
