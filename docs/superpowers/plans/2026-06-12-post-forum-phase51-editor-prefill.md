# Post Forum Phase 51 Editor Prefill

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make `/posts/:id/edit` load the existing post and prefill the editor form so author edit is a complete workflow instead of an empty form with update/delete actions.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice adds an author-checked editor loader. The edit route now reads the route `post_id` and query `session_id`, loads the existing post through a server function, verifies the current session owns the post, and hydrates the form field signals with the stored title, summary, category, tags, and Markdown body.

## Tasks

- [x] Add a RED contract test for edit-form prefill.
- [x] Add `load_editor_post(session_id, post_id)` server function.
- [x] Return `Ok(None)` for the new-post route with no `post_id`.
- [x] Validate session and reject non-author edit loads.
- [x] Load existing `PostDetail` for the edit route.
- [x] Add editor field signals for title, summary, category, tags, and markdown.
- [x] Prefill those signals once the editor resource resolves.
- [x] Keep submit, draft save, preview, upload, update, and delete actions intact.
- [x] Run targeted test, full test, compile check, Leptos build, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml editor_page_loads_existing_post_into_edit_form -- --nocapture`: failed before implementation with `editor load server path missing fragment: pub async fn load_editor_post(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml editor_page_loads_existing_post_into_edit_form -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 132 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- Browser verification:
  - Created a published post through the API.
  - Opened `/posts/{post_id}/edit?session_id=...`.
  - Hidden `post_id` matched the route parameter.
  - Title, summary, category, tag names, and Markdown body were prefilled from the saved post.
  - No horizontal overflow and no console errors were observed.
