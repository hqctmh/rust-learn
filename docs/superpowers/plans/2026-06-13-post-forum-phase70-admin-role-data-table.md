# Post Forum Phase 70 Admin Role Data Table

**Goal:** Replace the hard-coded admin role table with roles loaded through the admin dashboard data model.

## Scope

- Add dashboard role data to `AdminDashboard`.
- Populate dashboard roles from PostgreSQL via `AppState::list_roles`.
- Populate demo dashboard roles from the in-memory role store.
- Render the admin role table from `current_dashboard.get().roles`.
- Keep role action controls visible for the next create/update/delete action slices.

## Tasks

- [x] Add RED contract coverage for role dashboard data and role table rendering.
- [x] Add `roles: Vec<Role>` to `AdminDashboard`.
- [x] Seed demo role rows for fallback SSR/hydration.
- [x] Fill PostgreSQL and in-memory dashboard roles from real role data sources.
- [x] Replace hard-coded role rows with `RoleRow`.
- [x] Verify admin page tests, RBAC tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_renders_roles_from_dashboard_data -- --nocapture`: failed before implementation with `admin dashboard role domain missing fragment: rbac::{Permission, Role, admin_permissions}`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_renders_roles_from_dashboard_data -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 11 passed.
  - `cargo test --manifest-path post/Cargo.toml rbac_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 151 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `5.2` RBAC management by making the admin role table data-driven.
- Supports `5.3` user management and `5.8` audit/admin workflows indirectly by exposing current roles from the same dashboard source that later role actions can refresh.
