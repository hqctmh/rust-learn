# Post Forum Phase 81 Admin Post Lock Actions

**Goal:** Let admins lock and unlock posts from the management UI so homepage and detail workflows can reflect moderation state.

## Scope

- Add persistent `posts.is_locked` state with an incremental migration.
- Expose locked state through moderation, admin dashboard, and post detail DTOs.
- Add admin server actions and API routes for lock/unlock.
- Add lock/unlock forms in the admin post table.
- Keep deleted posts unlocked and reject locking deleted posts.

## Tasks

- [x] Add RED contract coverage for post lock schema, DTOs, server actions, and UI forms.
- [x] Add incremental Postgres migration for `posts.is_locked`.
- [x] Map locked state through SQLx repositories and demo store.
- [x] Add `lock_post` / `unlock_post` state methods and Leptos server functions.
- [x] Register lock/unlock admin API routes.
- [x] Render lock/unlock actions in the admin post table.
- [x] Verify admin tests, moderation tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_locks_and_unlocks_posts_with_session_actions -- --nocapture`: failed before implementation with missing `is_locked boolean not null default false`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_locks_and_unlocks_posts_with_session_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 35 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 164 passed, 2 ignored.
  - `cargo leptos build`: PASS.
  - `git diff --check`: PASS.

## PRD Coverage

- Supports admin post governance requirements for lock/unlock operations.
- Provides durable lock state for homepage row state, post detail behavior, and moderation audit flows.
