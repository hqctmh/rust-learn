# Post Forum Phase 52 Post Detail Comment Submit

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the post detail page submit real session-backed comments instead of only showing a static textarea.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice wires the post detail comment composer to a Leptos server action. The page reads `session_id` from the query string, submits `session_id`, `post_id`, optional `parent_comment_id`, and `content` through `ActionForm`, validates the current session on the server, and delegates comment creation to `AppState::add_comment`.

## Tasks

- [x] Add a RED contract test for post-detail comment submission.
- [x] Add `submit_comment(session_id, post_id, parent_comment_id, content)` server function.
- [x] Parse and validate `session_id` and `post_id`.
- [x] Parse optional `parent_comment_id` only when present.
- [x] Validate the session through `AppState::current_session`.
- [x] Create comments through `AppState::add_comment` and `CreateCommentRequest`.
- [x] Read `session_id` from the detail page query string.
- [x] Replace the static comment composer with `ActionForm`.
- [x] Submit hidden `session_id`, `post_id`, and `parent_comment_id` fields.
- [x] Disable submit when no session is available or the action is pending.
- [x] Render success and failure feedback from `comment_action.value()`.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_submits_comments_with_session_action -- --nocapture`: failed before implementation with `post detail comment server path missing fragment: pub async fn submit_comment(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_submits_comments_with_session_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 133 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/posts?session_id=...`: created a published post.
  - `POST /api/posts/{post_id}/comments?session_id=...`: returned `content = Phase52 API comment`.
  - `GET /api/posts/{post_id}/comments`: returned one comment for the created post.
- In-app browser verification:
  - Opened `/posts/{post_id}?session_id=...` for the API-created post.
  - Comment form rendered with `textarea[name="content"]`.
  - Hidden `session_id`, `post_id`, and `parent_comment_id` fields rendered with expected values.
  - Submit button rendered as `发表评论` and was enabled with a valid session.
  - No horizontal overflow and no console errors were observed.
