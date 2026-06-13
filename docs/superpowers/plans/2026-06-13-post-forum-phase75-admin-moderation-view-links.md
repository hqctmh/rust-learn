# Post Forum Phase 75 Admin Moderation View Links

**Goal:** Replace static admin moderation "查看" buttons with real links to post detail pages.

## Scope

- Add `post_id` to `AdminCommentRow`.
- Populate comment moderation rows from `ModerationCommentRow::post_id`.
- Render comment "查看帖子" as `/posts/{post_id}`.
- Render post "查看" as `/posts/{post_id}`.

## Tasks

- [x] Add RED contract coverage for admin moderation row links.
- [x] Extend admin comment row data with `post_id`.
- [x] Replace static view buttons with anchors to post detail routes.
- [x] Verify admin page, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_links_moderation_rows_to_post_detail_pages -- --nocapture`: failed before implementation with `admin moderation row link support missing fragment: comment.post_id`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_links_moderation_rows_to_post_detail_pages -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 16 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 157 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports admin content moderation workflows by making review navigation actionable.
- Keeps moderation tables connected to existing public post detail pages instead of static controls.
