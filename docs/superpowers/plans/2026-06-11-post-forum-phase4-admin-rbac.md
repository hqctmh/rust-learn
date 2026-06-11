# Post Forum Phase 4 Admin RBAC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable management backend foundation for PRD 5: admin-only dashboard data, RBAC menu permissions, moderation queues, and API-level 403 enforcement.

**Architecture:** Add `domain::admin` for dashboard DTOs. `ForumStore::admin_dashboard(user_id)` checks `SessionUser::is_admin` before returning stats, menu items, post moderation rows, governance queues, permissions, and audit entries. Axum exposes `GET /api/admin/dashboard?user_id=...`, while the Leptos admin page renders the same DTO from deterministic demo data.

**Tech Stack:** Rust 2024, Axum 0.8, Leptos 0.8, Serde, existing in-memory `ForumStore`, existing `rbac::admin_permissions`.

---

## Scope

This slice does not implement all admin CRUD mutation endpoints. It establishes the RBAC-protected dashboard and data contract that later user/post/comment/category/tag/announcement management actions can build on.

## Tasks

### Task 1: Contract Tests

- [ ] Admin user can fetch dashboard.
- [ ] Non-admin user gets `Forbidden`.
- [ ] Dashboard contains permission-driven menu items for users, roles, permissions, posts, comments, categories, tags, announcements, reports, audit.
- [ ] API inventory includes `/api/admin/dashboard`.

### Task 2: Domain and Store

- [ ] Create `post/src/domain/admin.rs`.
- [ ] Expose it in `domain/mod.rs`.
- [ ] Add `ForumStore::admin_dashboard(user_id)`.
- [ ] Reuse `rbac::admin_permissions()`.

### Task 3: API and Page

- [ ] Add `GET /api/admin/dashboard`.
- [ ] Update `api_route_inventory`.
- [ ] Bind `AdminPage` to `admin_dashboard_demo()`.

### Task 4: Verification

- [ ] `cargo fmt`
- [ ] `cargo test`
- [ ] `cargo check`
- [ ] `cargo leptos build`
- [ ] IDEA error check
- [ ] Browser/API verify `/admin` and `/api/admin/dashboard`

## Self-Review

- Covers PRD 5.1 and 5.2 core requirements: admin-only access, role/permission menu, backend permission check, and management dashboard data.
- Leaves mutations and audit persistence as follow-up slices.
