# Post Forum Phase 54 Post Detail Comment Replies

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make each comment on the post detail page submit real replies through the existing session-backed comment server action.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice reuses `submit_comment(session_id, post_id, parent_comment_id, content)` for comment replies. `PostDetailView` passes the existing `SubmitComment` server action, `session_id`, and `post_id` into each recursive comment item. Each comment now renders a compact reply form whose hidden `parent_comment_id` is the current comment ID.

## Tasks

- [x] Add a RED contract test for comment reply forms.
- [x] Pass `ServerAction<SubmitComment>` into the recursive comment tree.
- [x] Pass `session_id` and `post_id` into each `CommentItem`.
- [x] Render a reply `ActionForm` for each comment.
- [x] Submit hidden `session_id`, `post_id`, and `parent_comment_id` fields.
- [x] Disable reply submit when no session is available.
- [x] Preserve nested reply rendering.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_replies_to_comments_with_parent_comment_action_forms -- --nocapture`: failed before implementation with `post detail reply form should submit parent comments through server action fragment: comment_action: ServerAction<SubmitComment>`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml post_detail_page_replies_to_comments_with_parent_comment_action_forms -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 135 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/posts?session_id=...`: created a published post.
  - `POST /api/posts/{post_id}/comments?session_id=...`: created a parent comment.
  - `POST /api/posts/{post_id}/comments?session_id=...` with `parent_comment_id`: created a child reply.
  - `GET /api/posts/{post_id}/comments`: returned one root comment with one reply.
- In-app browser verification:
  - Restarted `cargo leptos serve` so the page used the new build.
  - Opened `/posts/{post_id}?session_id=...`.
  - Parent comment and child reply both rendered.
  - Comment items rendered reply forms with `session_id`, `post_id`, and `parent_comment_id`.
  - Browser-submitted reply returned `评论成功Phase54 browser reply`.
  - `GET /api/posts/{post_id}/comments` after browser submit returned two replies and included `Phase54 browser reply`.
  - No horizontal overflow and no console errors were observed.
