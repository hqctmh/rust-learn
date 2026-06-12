# Post Forum Phase 62 Admin Comment Moderation Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin comment-management table's moderation controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing comment moderation backend. The dashboard comment row now carries the target `comment_id`, and each comment row submits Leptos `ActionForm` controls for deleting and recovering comments. After each mutation, the server function reloads the admin dashboard so comment status, available actions, and moderation counts reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin comment moderation action wiring.
- [x] Expose `comment_id` on `domain::admin::AdminCommentRow`.
- [x] Map PostgreSQL/in-memory moderation rows into dashboard comment rows with target IDs.
- [x] Add `delete_admin_comment(session_id, comment_id)` server function.
- [x] Add `recover_admin_comment(session_id, comment_id)` server function.
- [x] Replace static comment moderation buttons with `ActionForm` controls.
- [x] Submit hidden `session_id` and `comment_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after moderation changes.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml admin_page_moderates_comments_with_session_actions -- --nocapture`: failed before implementation with `admin dashboard comment rows should expose target comment id fragment: pub comment_id: Uuid,`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml admin_page_moderates_comments_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 143 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to delete comments from the moderation table.
- Covers the PRD admin requirement to recover deleted comments.
- Keeps moderation actions session-backed, so only an authenticated admin session can trigger comment state changes.
