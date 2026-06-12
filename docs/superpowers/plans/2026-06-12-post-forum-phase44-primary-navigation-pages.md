# Post Forum Phase 44 Primary Navigation Pages

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Replace homepage-query placeholder links in the primary navigation with real public pages for the PRD/design navigation items: posts, docs, and activities.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice adds public pages for the remaining top navigation entries that were still placeholders. Tags and users were already handled by the public index pages slice.

## Tasks

- [x] Add contract tests that require `/posts`, `/docs`, and `/activities` in `primary_routes()`.
- [x] Update top navigation links from `/?tab=posts`, `/?tab=docs`, and `/?tab=events` to real routes.
- [x] Add `PostsIndexPage` using homepage topic data.
- [x] Add `DocsIndexPage` with forum documentation entry points.
- [x] Add `ActivitiesIndexPage` with community activity entry points.
- [x] Preserve responsive layout and avoid horizontal overflow.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml top_navigation_primary_tabs_have_real_public_routes -- --nocapture`: PASS.
- `cargo test --manifest-path post/Cargo.toml public_primary_pages_render_posts_docs_and_activities -- --nocapture`: PASS.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 125 passed, 2 ignored.
- `cargo leptos build`: PASS.
- HTTP checks:
  - `/posts`: 200.
  - `/docs`: 200.
  - `/activities`: 200.
- In-app browser verification:
  - `/posts`: 12 items, nav hrefs are `/`, `/posts`, `/tags`, `/users`, `/docs`, `/activities`, no horizontal overflow.
  - `/docs`: 3 items, no horizontal overflow.
  - `/activities`: 3 items, no horizontal overflow.
