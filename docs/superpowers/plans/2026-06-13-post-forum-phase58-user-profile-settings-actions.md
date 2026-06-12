# Post Forum Phase 58 User Profile Settings Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the `/me` user center's profile, avatar, and password controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice reuses the existing user settings domain, service validation, in-memory store, PostgreSQL repository, and JSON API behavior. The page data layer now exposes Leptos server functions for profile, avatar, and password updates. The `/me` page reads `session_id` from the query string, loads the authenticated user's space when a session is present, and renders compact settings forms that submit to those server actions.

## Tasks

- [x] Add a RED contract test for `/me` profile settings server actions.
- [x] Make `load_user_space_page` accept a viewer session identifier.
- [x] Use the viewer session to load the authenticated user's own space for `/me`.
- [x] Add `update_me_profile(session_id, nickname, bio)` server function.
- [x] Add `update_me_avatar(session_id, avatar_url)` server function.
- [x] Add `change_me_password(session_id, old_password, new_password)` server function.
- [x] Validate session IDs through `Uuid::parse_str` and `AppState::current_session`.
- [x] Delegate profile, avatar, and password changes to existing `AppState` methods.
- [x] Replace static profile buttons with `ActionForm` controls.
- [x] Render success and failure feedback for all three actions.
- [x] Keep public user profile pages loadable with optional viewer session context.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml user_space_page_updates_profile_avatar_and_password_with_session_actions -- --nocapture`: failed before implementation with `user settings server action missing fragment: pub async fn update_me_profile(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml user_space_page_updates_profile_avatar_and_password_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 139 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD requirement that a user can modify their nickname.
- Covers the PRD requirement that a user can modify their bio.
- Covers the PRD requirement that a user can modify their avatar URL.
- Covers the PRD requirement that a user can change their password.
