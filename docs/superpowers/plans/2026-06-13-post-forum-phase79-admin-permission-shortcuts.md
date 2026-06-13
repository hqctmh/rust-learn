# Post Forum Phase 79 Admin Permission Shortcuts

**Goal:** Make the admin permission shortcut links navigate to concrete management sections instead of pointing back to `/admin`.

## Scope

- Add section anchors for users, roles, permissions, and audit logs.
- Update permission shortcut links to use the new anchors.
- Keep the permission grid as an in-page navigation aid.

## Tasks

- [x] Add RED contract coverage for section ids and anchor hrefs.
- [x] Add ids to the relevant admin sections.
- [x] Replace `/admin` shortcut hrefs with in-page anchors.
- [x] Verify admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_permission_shortcuts_link_to_page_sections -- --nocapture`: failed before implementation with missing `id="admin-users"`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_permission_shortcuts_link_to_page_sections -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 33 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 162 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports admin discoverability by connecting permission shortcuts to the relevant management panels.
- Removes dead self-links from the management UI.
