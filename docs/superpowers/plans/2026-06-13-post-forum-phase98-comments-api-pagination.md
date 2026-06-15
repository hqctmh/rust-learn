# Post Forum Phase 98 Comments API Pagination

**Goal:** Make the comments API return paginated comment data so the post detail UI can load comment pages without changing the backend contract again.

## Scope

- Add API query parameters for comment pagination.
- Parse `page` and `page_size` through Axum `Query`.
- Return `CommentPage` from `GET /api/posts/{post_id}/comments`.
- Delegate API loading to `AppState::comments_page_for_post`.
- Preserve existing protected create/delete comment API behavior.

## Tasks

- [x] Add RED API contract coverage for `CommentPage` response shape.
- [x] Add `CommentPageQueryParams` for HTTP query parsing.
- [x] Convert API query params into domain `CommentPageQuery`.
- [x] Change `list_comments` to return `Json<CommentPage>`.
- [x] Verify targeted API tests, related comment pagination tests, full suite, build, and formatting.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract comments_api_returns_paginated_comment_page -- --nocapture`: failed before implementation because `CommentPage` API response wiring was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract comments_api_returns_paginated_comment_page -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract api_routes_ -- --nocapture`: PASS, 2 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract protected_api_handlers_ -- --nocapture`: PASS, 2 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_comments_page_paginates_postgres_root_comments -- --nocapture`: PASS.
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 180 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_axum`.
- `Query<T>` parses URL query strings into `Deserialize` structs and can be combined with other extractors such as `Path` and `Extension`.
- Handlers can return `Json<T>`, so the comments endpoint can expose the domain `CommentPage` directly.

## PRD Coverage

- Supports PRD `12` API requirement by making the comment list endpoint suitable for paginated clients.
- Supports PRD `14.1` pagination requirement for comment lists.
