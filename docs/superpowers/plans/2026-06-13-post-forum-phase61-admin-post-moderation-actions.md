# Post Forum Phase 61 Admin Post Moderation Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin post-management table's moderation controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing post moderation backend. The dashboard post row now carries the target `post_id`, and each post row submits Leptos `ActionForm` controls for taking down, restoring, deleting, pinning, and unpinning posts. After each mutation, the server function reloads the admin dashboard so post status, pinned action labels, and moderation counts reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin post moderation action wiring.
- [x] Expose `post_id` on `domain::admin::AdminPostRow`.
- [x] Map PostgreSQL/in-memory moderation rows into dashboard rows with target IDs.
- [x] Add `take_down_admin_post(session_id, post_id)` server function.
- [x] Add `restore_admin_post(session_id, post_id)` server function.
- [x] Add `delete_admin_post(session_id, post_id)` server function.
- [x] Add `pin_admin_post(session_id, post_id)` server function.
- [x] Add `unpin_admin_post(session_id, post_id)` server function.
- [x] Replace static post moderation buttons with `ActionForm` controls.
- [x] Submit hidden `session_id` and `post_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after moderation changes.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml admin_page_moderates_posts_with_session_actions -- --nocapture`: failed before implementation with `admin dashboard post rows should expose target post id fragment: pub post_id: Uuid,`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml admin_page_moderates_posts_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 142 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to take posts offline.
- Covers the PRD admin requirement to restore offline/deleted posts.
- Covers the PRD admin requirement to delete posts from the moderation table.
- Covers the PRD admin requirement to pin and unpin posts.
- Keeps moderation actions session-backed, so only an authenticated admin session can trigger post state changes.
