# Post Forum Phase 47 Editor Autosave Draft

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the editor's "自动保存草稿" control perform a real session-backed draft save instead of being a static button.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice extends the existing editor server action with a `save_mode` submit value. `save_mode=draft` calls `AppState::autosave_draft`; `save_mode=publish` keeps the existing publish path through `AppState::create_post`.

## Tasks

- [x] Add a contract test proving the editor supports draft autosave through the same session-backed action.
- [x] Extend `submit_post` with `save_mode`.
- [x] Route `save_mode=draft` to `AutosaveDraftRequest` and `AppState::autosave_draft`.
- [x] Convert "自动保存草稿" into a real submit button with `name="save_mode"` and `value="draft"`.
- [x] Keep "发布帖子" as `save_mode=publish`.
- [x] Show separate success copy for saved drafts and published posts.
- [x] Run targeted test, full test, compile check, Leptos build, API verification, and browser verification.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml editor_page_autosaves_drafts_with_same_session_backed_action`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 128 passed, 2 ignored.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
- API verification:
  - `POST /api/register`: returned a valid `session_id`.
  - `POST /api/posts/drafts/autosave?session_id=...`: returned `status = Draft`, `published_at = null`, and the submitted tags.
- In-app browser verification:
  - `/posts/new?session_id=...`: one `submit_post` form rendered.
  - `save_mode=draft` and `save_mode=publish` buttons both rendered.
  - Session input was prefilled from the query string.
  - No horizontal overflow and no console errors were observed.
