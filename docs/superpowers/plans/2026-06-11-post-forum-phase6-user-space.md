# Post Forum Phase 6 User Space Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add user profile and personal center foundations for PRD 4.10 and the user page list: public profile, my posts, drafts, comments, favorites, following, and followers.

**Architecture:** Add `domain::users` for `UserSpace` aggregate data. `ForumStore::user_space(profile_user_id, viewer_user_id)` derives stats from existing posts, comments, favorites, and follows. Expose `GET /api/users/{user_id}/space` and render `/users/:id` plus `/me` pages from deterministic demo data.

**Tech Stack:** Rust 2024, Axum 0.8, Leptos 0.8 Router, Serde, existing in-memory `ForumStore`.

---

## Scope

This slice does not implement profile mutation or password change yet. It creates the read-side profile/personal-center contract and page surfaces that later edit APIs can update.

## Tasks

### Task 1: Contract Tests

- [ ] User space includes profile, stats, published posts, comments, favorites, following, and followers.
- [ ] Favorites and follows update user space after toggle actions.
- [ ] Routes `/users/sample`, `/me`, and `/api/users/{user_id}/space` are registered.

### Task 2: Domain and Store

- [ ] Create `post/src/domain/users.rs`.
- [ ] Expose module in `domain/mod.rs`.
- [ ] Add `ForumStore::user_space(profile_user_id, viewer_user_id)`.
- [ ] Add `ForumStore::user_space_demo()`.

### Task 3: API and Pages

- [ ] Add `GET /api/users/{user_id}/space`.
- [ ] Add `UserProfilePage` and `MePage`.
- [ ] Register routes.
- [ ] Update login account links to the new pages.

### Task 4: Verification

- [ ] `cargo fmt`
- [ ] `cargo test`
- [ ] `cargo check`
- [ ] `cargo leptos build`
- [ ] IDEA error check
- [ ] Browser/API verify `/users/sample`, `/me`, and user-space API.

## Self-Review

- Covers PRD 4.10 read-side requirements and user page list.
- Leaves profile editing/password change as a follow-up mutation slice.
