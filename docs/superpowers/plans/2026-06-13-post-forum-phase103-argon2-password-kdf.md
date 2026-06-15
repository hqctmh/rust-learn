# Post Forum Phase 103 Argon2 Password KDF

**Goal:** Replace the temporary SHA-256 password storage with a production-oriented Argon2 password KDF while preserving legacy login compatibility.

## Scope

- Add the RustCrypto `argon2` crate.
- Generate new password hashes as PHC strings starting with `$argon2id$`.
- Verify Argon2 hashes through `PasswordHash::new` and `PasswordVerifier`.
- Preserve compatibility with the previous `sha256$v1$` hashes.
- Preserve compatibility with old plaintext demo passwords so existing local data can still authenticate.
- Keep all password write paths using `AuthService::hash_password`.

## Tasks

- [x] Query current Argon2 API documentation through Context7.
- [x] Add RED auth service coverage requiring `$argon2id$` output.
- [x] Add `argon2 = "0.5.3"`.
- [x] Implement Argon2 PHC hashing and verification.
- [x] Keep legacy SHA-256 and plaintext fallback verification.
- [x] Verify auth and password-change regressions.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_service_normalizes_credentials_and_builds_sessions -- --nocapture`: failed before implementation because `hash_password` still returned `sha256$v1$`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_service_normalizes_credentials_and_builds_sessions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract auth_ -- --nocapture`: PASS, 6 passed.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract user_profile_contract_supports_profile_avatar_and_password_updates -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_user_settings_persist_to_postgres -- --nocapture`: PASS.

## Context7 Notes

- Library: `/websites/rs_argon2`.
- Recommended password hashing flow:
  - `SaltString::generate(&mut OsRng)`.
  - `Argon2::default().hash_password(password, &salt)?.to_string()`.
  - `PasswordHash::new(&password_hash)?` for parsing.
  - `Argon2::default().verify_password(password, &parsed_hash)` for verification.
- `Argon2::default()` uses Argon2id v19 according to the docs.
- Verification uses parameters encoded in the stored PHC string.

## PRD Coverage

- Strengthens PRD `7` password encrypted storage from non-plaintext hashing to a password KDF.
- Keeps PRD `13` authentication and password-change flows working for current local/demo data.
