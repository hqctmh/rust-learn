# Post Forum Phase 24 User Settings Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move user profile/avatar/password validation rules out of `state.rs` into `services::users`.

**Architecture:** `state.rs` remains responsible for locking, checking user existence/disabled state, updating maps, and propagating changed display names or avatars to posts/comments. `services::users` owns pure rules for trimming profile fields, validating length, validating avatar URLs, and checking password changes.

**Tech Stack:** Rust, existing user domain request models, existing in-memory `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract User Settings Rules

**Files:**
- Create/modify: `post/src/services/users.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::users::UserSettingsService` and verifies:
- nickname cannot be blank;
- bio over 160 chars is rejected;
- profile fields are trimmed;
- avatar URL must be relative or HTTP(S);
- password change rejects wrong old password and too-short new password;
- valid password change returns the trimmed new password.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml user_settings_service
```

Expected: compile failure because `services::users` does not exist.

- [x] **Step 3: Implement `UserSettingsService`**

Create methods for profile normalization, avatar URL normalization, and password change validation.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline validation and trimming in `update_profile`, `update_avatar`, and `change_password`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
