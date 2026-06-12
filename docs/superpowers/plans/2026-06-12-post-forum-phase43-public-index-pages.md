# Post Forum Phase 43 Public Index Pages

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and update checkboxes as work is completed.

**Goal:** Satisfy the homepage UI/PRD requirement that the four sidebar "view all" links have real public destination routes: categories, tags, announcements, and active authors.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice adds public route/page support for the homepage sidebar modules. The pages reuse the same homepage aggregate/demo data shape so they stay visually and semantically aligned with the supplied homepage design.

## Tasks

- [x] Add contract tests for `/categories`, `/tags`, `/announcements`, and `/users` public routes.
- [x] Add `public_indexes` pages for categories, tags, announcements, and active authors.
- [x] Update homepage sidebar "查看全部" links to point at the corresponding public pages.
- [x] Update top navigation "标签" and "用户" links to point at public index pages instead of query placeholders.
- [x] Add compact public-index styles with responsive grid/list behavior.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml homepage_sidebar_view_all_links_have_public_routes -- --nocapture`: PASS.
- `cargo test --manifest-path post/Cargo.toml public_index_pages_render_homepage_sidebar_data_sources -- --nocapture`: PASS.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 123 passed, 2 ignored.
- `cargo leptos build`: PASS.
- In-app browser verification:
  - `/categories`: 6 items, no horizontal overflow.
  - `/tags`: 8 items, no horizontal overflow.
  - `/announcements`: 3 items, no horizontal overflow.
  - `/users`: 5 items, no horizontal overflow.
