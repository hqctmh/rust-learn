# Post Forum Phase 46 Editor Session Submit

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Connect the auth server-action result to the post editor so a logged-in user can carry a `session_id` into `/posts/new` and publish through a Leptos server action backed by `AppState`.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice makes the visible editor form submit a real post. It uses the existing session model and validates the submitted `session_id` with `AppState::current_session` before calling `AppState::create_post`.

## Tasks

- [x] Add a contract test proving the editor submits through a session-backed Leptos server action.
- [x] Add `submit_post` server function with session validation and tag splitting.
- [x] Update login/register success links to open `/posts/new?session_id=...`.
- [x] Convert the editor UI to an `ActionForm` with real `session_id`, title, summary, category, tag, markdown, and publish fields.
- [x] Add publish pending, success, and error feedback.
- [x] Keep editor layout compact and responsive.
- [x] Run targeted test, full test, compile check, Leptos build, and browser verification.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml editor_page_submits_posts_with_session_backed_server_action`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 127 passed, 2 ignored.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
- In-app browser verification:
  - Registered a test account through `/register`.
  - Registration success link opened `/posts/new?session_id=...`.
  - Editor form posted to `/api/submit_post...` with session, title, summary, category, tags, markdown, and publish fields.
  - Publish success feedback appeared with the submitted title.
  - No horizontal overflow and no console errors were observed.
