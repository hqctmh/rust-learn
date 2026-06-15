# Post Forum Phase 93 Admin Post Recommend Actions

**Goal:** Support admin recommend and unrecommend actions for posts so the system can back homepage recommendation and moderation workflows.

## Scope

- Expose post recommendation state in moderation DTOs and admin dashboard rows.
- Persist recommendation state through Postgres with SQLx checked macros.
- Preserve recommendation state when pinning, locking, and taking posts offline.
- Clear recommendation state when deleting posts.
- Add admin server functions and action forms for recommending and unrecommending posts.
- Keep the demo in-memory store aligned with the Postgres runtime behavior.

## Tasks

- [x] Add RED runtime coverage for `AppState::recommend_post` and `AppState::unrecommend_post`.
- [x] Add RED source coverage for admin server actions and UI action forms.
- [x] Add `recommended` to moderation/domain/admin rows.
- [x] Add Postgres repository update/read mapping for `posts.is_recommended`.
- [x] Add in-memory and Postgres `AppState` recommendation flows.
- [x] Wire admin page `推荐` and `取消推荐` session-backed actions.
- [x] Verify target tests, admin tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_content_moderation_persists_to_postgres -- --nocapture`: failed before implementation because `AppState::recommend_post`, `AppState::unrecommend_post`, and `ModerationPostRow::recommended` did not exist.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_recommends_and_unrecommends_posts_with_session_actions -- --nocapture`: failed before implementation because admin recommendation server actions and UI forms did not exist.
- GREEN:
  - `cargo fmt --manifest-path post/Cargo.toml --check`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_recommends_and_unrecommends_posts_with_session_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_content_moderation_persists_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 22 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract moderation_service_applies_post_and_comment_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 177 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query_as!` maps output columns to struct fields by name and rejects unused or missing fields.
- Boolean columns returned from expressions or aliases should use checked aliases such as `"recommended!"` when SQLx cannot infer nullability.
- Mutating admin actions use `returning` rows so the service receives the persisted status, pin, recommend, and lock state from the database.

## PRD Coverage

- Supports `5.4` content management requirement for admin recommending and cancelling recommendation of posts.
- Ensures recommendation state is part of the moderation and dashboard data model instead of being UI-only.
