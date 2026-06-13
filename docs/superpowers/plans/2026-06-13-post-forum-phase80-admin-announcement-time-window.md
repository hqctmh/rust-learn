# Post Forum Phase 80 Admin Announcement Time Window

**Goal:** Let admins configure announcement effective and expiry times from the management UI.

## Scope

- Add `effective_at` and `expires_at` form fields to announcement create and edit actions.
- Parse HTML `datetime-local` values in server functions.
- Pass parsed times into `CreateAnnouncementRequest` and `UpdateAnnouncementRequest`.
- Preserve existing announcement publish, withdraw, push, and update flows.

## Tasks

- [x] Use Context7 to verify `time` crate parsing API.
- [x] Add RED contract coverage for announcement time fields and parsing.
- [x] Enable `time` crate parsing/macros features.
- [x] Add `parse_optional_announcement_time`.
- [x] Add `datetime-local` inputs to create and edit announcement forms.
- [x] Verify announcement tests, admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_configures_announcement_effective_and_expiry_times -- --nocapture`: failed before implementation with missing `effective_at: String`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_configures_announcement_effective_and_expiry_times -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml announcement -- --nocapture`: PASS, 9 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 34 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 163 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/time-rs/time`.
- `PrimitiveDateTime::parse(value, format)` parses custom no-offset date-time strings when the `parsing` feature is enabled.
- `time::macros::format_description!` provides compile-time format descriptions when the `macros` feature is enabled.
- A parsed `PrimitiveDateTime` can be converted to `OffsetDateTime` with `assume_utc()`.

## PRD Coverage

- Supports `4.1.2` and `5` announcement operations by exposing effective and expiry time configuration.
- Keeps homepage announcement visibility driven by stored announcement time windows instead of admin UI defaults.
