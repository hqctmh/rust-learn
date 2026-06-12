# Post Forum Phase 59 User Profile Follow Action

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the public user profile page's "关注用户 / 取消关注" control submit a real session-backed follow action instead of remaining a static button.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice reuses the existing `toggle_author_follow(session_id, author_id)` Leptos server function and `AppState::follow_user` backend behavior. Public user profile pages already accept `session_id` as viewer context; this phase wires the visible follow control to an `ActionForm`, submits the viewed profile's user ID as the target, and updates the button label from the action result.

## Tasks

- [x] Add a RED contract test for profile follow action wiring.
- [x] Reuse `ToggleAuthorFollow` server action for user profile follow/unfollow.
- [x] Add page-level `ServerAction<ToggleAuthorFollow>`.
- [x] Replace the static non-self follow button with an `ActionForm`.
- [x] Submit hidden `session_id` and target `author_id`.
- [x] Disable the button while pending or when no session is available.
- [x] Render success and failure feedback for follow/unfollow.
- [x] Update the button label from the returned `FollowState`.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml user_profile_page_toggles_follow_with_session_action -- --nocapture`: failed before implementation with `user profile follow UI missing fragment: ToggleAuthorFollow`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml user_profile_page_toggles_follow_with_session_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 140 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD requirement that a logged-in user can follow another user.
- Covers the PRD requirement that a logged-in user can cancel an existing follow.
- Reuses the existing follow notification backend so followed users posting can continue to notify followers.
