# Post Forum Phase 50 Editor Update And Delete

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make `/posts/:id/edit` submit updates to the existing post and make the editor's "删除自己的帖子" control call the real author-delete path.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice keeps the create-post editor flow intact while making the route-aware edit flow use the `:id` route parameter as `post_id`. When `post_id` is present, publishing updates the existing post through `AppState::update_post`; draft save passes the same `post_id` into `AutosaveDraftRequest`. Deletion uses a separate Leptos `Action` backed by `delete_editor_post`.

## Tasks

- [x] Add a RED contract test for edit-route update/delete wiring.
- [x] Add `post_id` to `submit_post`.
- [x] Parse optional `post_id` from the edit route and submit it as a hidden field.
- [x] Route publish with `post_id` to `UpdatePostRequest` and `AppState::update_post`.
- [x] Route draft save with `post_id` to existing draft autosave.
- [x] Add `delete_editor_post` server function.
- [x] Make "删除自己的帖子" dispatch a delete action and disable it on the new-post route.
- [x] Render delete success and failure feedback.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml editor_page_updates_and_deletes_existing_posts_from_edit_route -- --nocapture`: failed before implementation with `editor update/delete server path missing fragment: .update_post(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml editor_page_updates_and_deletes_existing_posts_from_edit_route -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 131 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/posts?session_id=...`: created a published post.
  - `POST /api/posts/{post_id}/update?session_id=...`: returned the updated title, updated markdown, and `status = Published`.
  - `POST /api/posts/{post_id}/delete?session_id=...`: returned `status = Deleted`.
- In-app browser verification:
  - `/posts/{post_id}/edit?session_id=...`: hidden `post_id` matched the route parameter.
  - Session input was prefilled from the query string.
  - Delete button rendered and was enabled for an edit route.
  - Draft and publish submit buttons still rendered.
  - No horizontal overflow and no console errors were observed.
