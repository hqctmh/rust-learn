# Post Forum Phase 82 Admin Section Anchor Regression

**Goal:** Fix admin permission shortcuts so each shortcut targets the correct unique management section.

## Scope

- Ensure post management uses `id="admin-posts"`.
- Ensure audit log uses a unique `id="admin-audit"`.
- Keep comment management on a separate `id="admin-comments"`.
- Add a post management shortcut in the permission grid.

## Tasks

- [x] Add RED contract coverage for concrete section-to-heading mapping and unique audit id.
- [x] Fix admin page section ids.
- [x] Add the post management shortcut.
- [x] Verify target test, admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_permission_shortcuts_link_to_page_sections -- --nocapture`: failed first with missing `id="admin-posts"`, then exposed duplicate `id="admin-audit"`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_permission_shortcuts_link_to_page_sections -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 35 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 164 passed, 2 ignored.
  - `cargo leptos build`: PASS.
  - `git diff --check`: PASS.

## PRD Coverage

- Keeps RBAC permission shortcuts usable for the dense admin workbench.
- Prevents duplicate anchors from sending admins to the wrong governance panel.
