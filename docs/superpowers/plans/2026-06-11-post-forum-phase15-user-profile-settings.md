# Post Forum Phase 15 User Profile Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Complete user profile editing, avatar URL update, password change, and personal-center list routing required by `post/prd.md` sections 4.10, 11.1, 12, 13, and 16.

**Architecture:** Extend the existing `users` domain with request structs, keep persistence in `ForumStore`, and expose thin Axum handlers under `/api/users/...`. Store demo password state in-memory for MVP validation while keeping the public `UserProfile` response free of password data. Reuse the existing `UserSpace` aggregate for personal lists and add visible route entries for `/me/posts`, `/me/drafts`, `/me/comments`, `/me/favorites`, `/me/following`, and `/me/followers`.

**Tech Stack:** Rust, Leptos, Axum 0.8, serde, uuid, existing `ForumStore`, `ForumError`, and `UserSpace` patterns.

---

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing profile test**

Add `user_profile_contract_supports_profile_avatar_and_password_updates` proving:
- a user can update nickname and bio;
- nickname and bio changes are reflected in `user_space`;
- avatar URL can be updated separately;
- password change rejects a wrong old password;
- password change accepts the correct old password;
- after password change, old password no longer logs in and new password does.

- [x] **Step 2: Write failing route test**

Add `user_profile_routes_are_registered` asserting:
- `/me/posts`
- `/me/drafts`
- `/me/comments`
- `/me/favorites`
- `/me/following`
- `/me/followers`
- `/api/users/{user_id}/profile`
- `/api/users/{user_id}/avatar`
- `/api/users/{user_id}/password`

- [x] **Step 3: Run targeted tests and verify red**

Run:

```bash
cargo test user_profile --test phase1_contract
```

Expected: fail because request structs, store methods, and routes do not exist.

### Task 2: Domain and Store

**Files:**
- Modify: `post/src/domain/users.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Add request structs**

Add:
- `UpdateProfileRequest { nickname, bio }`
- `UpdateAvatarRequest { avatar_url }`
- `ChangePasswordRequest { old_password, new_password }`

- [x] **Step 2: Add profile metadata and password state**

Extend `ForumData` with:
- `user_bios: HashMap<Uuid, String>`
- `user_registered_at: HashMap<Uuid, OffsetDateTime>`
- `user_passwords: HashMap<Uuid, String>`

Seed demo user and login-created users with password data.

- [x] **Step 3: Implement store methods**

Add:
- `ForumStore::update_profile(user_id, request) -> UserProfile`
- `ForumStore::update_avatar(user_id, request) -> UserProfile`
- `ForumStore::change_password(user_id, request) -> Result<(), ForumError>`

Rules:
- missing user returns unauthorized;
- disabled user returns forbidden;
- nickname must be non-empty and <= 32 chars;
- bio must be <= 160 chars;
- avatar URL must be non-empty and start with `/` or `http://` or `https://`;
- password must be non-empty and new password must be at least 6 chars.

- [x] **Step 4: Run targeted behavior test and verify green**

Run:

```bash
cargo test user_profile_contract --test phase1_contract
```

Expected: pass.

### Task 3: API and UI Routes

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/pages/user_space.rs`

- [x] **Step 1: Add API routes and handlers**

Add:
- `POST /api/users/{user_id}/profile`
- `POST /api/users/{user_id}/avatar`
- `POST /api/users/{user_id}/password`

Handlers use `Path<Uuid>`, `Extension<ForumStore>`, and `Json<T>` per Axum 0.8 docs.

- [x] **Step 2: Add personal center routes**

Add primary routes and Leptos routes:
- `/me/posts`
- `/me/drafts`
- `/me/comments`
- `/me/favorites`
- `/me/following`
- `/me/followers`

They may reuse `MePage` in this MVP while preserving explicit route surfaces.

- [x] **Step 3: Update visible personal-center links**

Point the personal function grid to the dedicated `/me/...` routes and add visible settings actions for 修改头像、修改昵称、修改简介、修改密码.

- [x] **Step 4: Run route test and verify green**

Run:

```bash
cargo test user_profile_routes_are_registered --test phase1_contract
```

Expected: pass.

### Task 4: Verification

**Files:**
- All touched files.

- [x] **Step 1: Format**

Run:

```bash
cargo fmt
```

- [x] **Step 2: Full verification**

Run:

```bash
cargo test
cargo check
cargo leptos build
```

- [x] **Step 3: IDEA error inspection**

Check errors only for:
- `post/src/domain/users.rs`
- `post/src/state.rs`
- `post/src/api.rs`
- `post/src/app.rs`
- `post/src/pages/user_space.rs`
- `post/tests/phase1_contract.rs`

Expected: no errors.
