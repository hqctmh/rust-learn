# Post Forum Phase 49 Editor Image Upload

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the editor's "插入图片" control use the existing binary upload backend and insert the returned Markdown image link into the post body.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice connects the editor UI to the existing `/api/files/binary` upload model through a Leptos server function. The browser reads the selected image as a data URL, sends base64 content to the server, and the server pins the usage to `MarkdownImage` before delegating to `AppState::upload_binary_file`.

## Tasks

- [x] Add a RED contract test for editor image upload and Markdown insertion.
- [x] Add hydrate-only browser file API dependency features.
- [x] Add `upload_editor_image` server function.
- [x] Read `PNG/JPEG/WebP` file input with `FileReader`.
- [x] Strip the data URL prefix and upload base64 content.
- [x] Append returned `asset.markdown_image` into the Markdown textarea signal.
- [x] Render upload success and failure feedback.
- [x] Style the upload label as a normal editor toolbar button and hide the raw file input.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml editor_page_uploads_image_and_inserts_markdown_link -- --nocapture`: failed before implementation with `editor image upload server function missing fragment: pub async fn upload_editor_image(`.
  - After adding upload behavior, the same test failed on the missing toolbar upload button style fragment, then passed after CSS was added.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml editor_page_uploads_image_and_inserts_markdown_link -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 130 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/files/binary?session_id=...`: returned `mime_type = image/png`, a `/uploads/markdown/...` public URL, and a Markdown image string.
- In-app browser verification:
  - `/posts/new?session_id=...`: file input and upload toolbar control rendered.
  - File input `accept` was `image/png,image/jpeg,image/webp`.
  - Raw file input was hidden with `display: none`.
  - Upload toolbar control used the toolbar button styling.
  - No horizontal overflow and no console errors were observed.
