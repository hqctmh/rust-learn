# Post Forum Phase 55 Post Detail Comment Like And Delete

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the post detail page's comment like and own-comment delete controls submit real session-backed interactions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice wires each comment row to two Leptos server actions. `toggle_comment_like(session_id, comment_id)` validates the current session and delegates to `AppState::toggle_comment_like`. `delete_own_comment(session_id, comment_id)` validates the current session and delegates to `AppState::delete_own_comment`. The recursive comment component now receives those actions and submits hidden `session_id` and `comment_id` fields.

## Tasks

- [x] Add a RED contract test for comment like and delete server actions.
- [x] Add `toggle_comment_like(session_id, comment_id)` server function.
- [x] Add `delete_own_comment(session_id, comment_id)` server function.
- [x] Validate `session_id` and `comment_id` through `Uuid::parse_str`.
- [x] Validate the current session through `AppState::current_session`.
- [x] Delegate to existing AppState comment interaction APIs.
- [x] Add page-level `ServerAction<ToggleCommentLike>` and `ServerAction<DeleteOwnComment>`.
- [x] Pass comment interaction actions into the recursive comment tree.
- [x] Render comment like and delete `ActionForm` controls with hidden `session_id` and `comment_id`.
- [x] Render success and failure feedback for comment interaction actions.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_toggles_and_deletes_comments_with_session_actions -- --nocapture`: failed before implementation with `comment interaction server function missing fragment: pub async fn toggle_comment_like(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_toggles_and_deletes_comments_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 136 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/posts?session_id=...`: created a published post.
  - `POST /api/posts/{post_id}/comments?session_id=...`: created a comment.
  - `POST /api/comments/{comment_id}/like` with `{"session_id": "..."}` returned `active = true`, `count = 1`.
  - `POST /api/comments/{comment_id}/delete` with `{"session_id": "..."}` returned `content = 该评论已被删除`.
  - `GET /api/posts/{post_id}/comments` returned the deleted placeholder as public comment content.
- In-app browser verification:
  - Restarted `cargo leptos serve` so the page used the new build.
  - Opened `/posts/{post_id}?session_id=...`.
  - Comment row rendered like and delete forms with hidden `session_id` and `comment_id`.
  - Browser click on like returned `评论点赞成功已点赞评论 · 1`.
  - Browser click on delete returned `评论删除成功该评论已被删除`.
  - `GET /api/posts/{post_id}/comments` after browser actions returned `public_content = 该评论已被删除` and `like_count = 1`.
  - No horizontal overflow and no console errors were observed.
