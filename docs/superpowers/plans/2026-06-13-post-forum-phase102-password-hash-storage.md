# Post Forum Phase 102 Password Hash Storage

**Goal:** Stop storing newly written passwords as plaintext and satisfy the PRD security requirement for non-plaintext password storage.

## Scope

- Add `AuthService::hash_password`.
- Prefix password hashes with `sha256$v1$` for versioned verification.
- Keep `validate_password_match` compatible with legacy plaintext values so old demo data and tests can still authenticate.
- Hash PostgreSQL registration passwords before insert.
- Hash PostgreSQL password-change values before update.
- Hash demo seed, demo first-login, demo registration, and demo password-change values before storing in memory.
- Reuse `AuthService::validate_password_match` in user password-change validation.

## Tasks

- [x] Add RED auth service contract coverage for password hashing and storage call sites.
- [x] Implement SHA-256 password hashing with a versioned prefix.
- [x] Wire all password write paths to store hashes.
- [x] Verify auth and password-change regressions.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_service_normalizes_credentials_and_builds_sessions -- --nocapture`: failed before implementation because `AuthService::hash_password` was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_service_normalizes_credentials_and_builds_sessions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_ -- --nocapture`: PASS, 6 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract user_profile_contract_supports_profile_avatar_and_password_updates -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_user_settings_persist_to_postgres -- --nocapture`: PASS.

## API Notes

- `sha2` exposes `Sha256::new()`, `Digest::update`, and `Digest::finalize`.
- The current implementation is a non-plaintext compatibility step using existing project dependencies.
- A production deployment should replace this with a password KDF such as Argon2 or bcrypt.

## PRD Coverage

- Supports PRD `7` password storage security requirement.
- Supports PRD `13` login and password-change permission rules without breaking existing demo fallback behavior.
