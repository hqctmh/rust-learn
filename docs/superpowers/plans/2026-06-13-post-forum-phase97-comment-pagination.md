# Post Forum Phase 97 Comment Pagination

**Goal:** Support paginated comment loading for post detail comment lists as required by PRD 14.1.

## Scope

- Add `CommentPageQuery` and `CommentPage`.
- Keep `comments_for_post` as the compatibility full-list API.
- Add paginated Postgres comment loading for root comments with replies attached for the current page.
- Add demo/in-memory fallback pagination.
- Carry `comments_page` in `PostDetailPageData` while preserving the existing `comments` field for the current page.

## Tasks

- [x] Add RED source coverage for paginated comment SQL and post detail page data.
- [x] Add RED Postgres runtime coverage for page/page size/total/total pages.
- [x] Implement SQLx checked paginated comment query with `limit` and `offset`.
- [x] Implement `AppState::comments_page_for_post` and `ForumStore::comments_page_for_post`.
- [x] Wire post detail server loader to use the first comment page.
- [x] Verify target tests, related comment/detail tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_loads_route_post_and_comments_through_server_state -- --nocapture`: failed before implementation because `PostDetailPageData::comments_page` and `.comments_page_for_post(` did not exist.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_comment_repository_contract_maps_comment_tree_rows -- --nocapture`: failed before implementation because `comments_for_post_page_sql` and `count_root_comments_sql` did not exist.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_comments_page_paginates_postgres_root_comments -- --nocapture`: failed before implementation because `CommentPageQuery` and `AppState::comments_page_for_post` did not exist.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_loads_route_post_and_comments_through_server_state -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_comment_repository_contract_maps_comment_tree_rows -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_comments_page_paginates_postgres_root_comments -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_add_comment_persists_to_postgres_and_updates_post_count -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_comment_reactions_and_delete_persist_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract post_detail_page_ -- --nocapture`: PASS, 7 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 179 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps selected columns to struct fields by name and validates them at compile time.
- Multiple-row list queries use `fetch_all`.
- COUNT aggregates should use `i64`; the query aliases the value as `"count!"`.
- The paginated SQL uses `limit $2` and `offset $3` for bind-checked pagination parameters.

## PRD Coverage

- Supports `14.1` non-functional requirement that comment lists are paginated.
- Gives post detail server data a page object with totals so UI pagination controls can be added without changing the backend contract.
