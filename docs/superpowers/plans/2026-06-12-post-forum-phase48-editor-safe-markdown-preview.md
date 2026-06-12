# Post Forum Phase 48 Editor Safe Markdown Preview

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the editor's "预览安全过滤" control execute a real server-backed Markdown preview that uses the same safe rendering path as published posts and drafts.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice turns the preview button from a static UI affordance into a Leptos `Action` that calls a server function. The server function delegates to `PostAuthoringService::preview_markdown`, so preview, draft, and publish paths share the same XSS escaping behavior.

## Tasks

- [x] Add a RED contract test proving the editor preview must use a safe server action.
- [x] Add `PostAuthoringService::preview_markdown` with empty-body validation.
- [x] Add `preview_markdown` server function returning sanitized HTML.
- [x] Track textarea content with a Leptos signal.
- [x] Dispatch preview from the `type="button"` preview control without submitting the post form.
- [x] Render preview HTML with `inner_html` only after server-side escaping.
- [x] Show a clear preview error state for validation/server errors.
- [x] Run targeted test, full test, compile check, Leptos build, and in-app browser verification.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml editor_page_previews_markdown_through_safe_server_action -- --nocapture`: failed before implementation with `preview server function missing fragment: pub async fn preview_markdown(`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml editor_page_previews_markdown_through_safe_server_action -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 129 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
- In-app browser verification:
  - `/posts/new`: Markdown textarea rendered.
  - "预览安全过滤" rendered as `type="button"`.
  - No horizontal overflow and no console errors were observed.
  - Previewing `# 安全预览\n<script>alert(1)</script>` rendered `<h1>安全预览</h1>` and escaped the script as `&lt;script&gt;alert(1)&lt;/script&gt;`.
  - The preview DOM contained no raw `<script>` element.
