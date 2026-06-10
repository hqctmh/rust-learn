# Post Forum PRD Completion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `post` 项目中补齐 `prd.md` 中 Phase 1 主干之外的可验证论坛能力。

**Architecture:** 沿用现有 Leptos SSR + Axum + 内存 `ForumStore` 的第一阶段结构，把搜索、上传、通知、公告、举报、审计和统计先做成领域契约与 JSON API。Redis、NATS、RustFS、Elasticsearch 继续保留在 Docker Compose 和 README 的真实接入边界中，当前实现用内存仓储证明业务规则和接口形状。

**Tech Stack:** Rust、Leptos、Axum、serde、uuid、time、Docker Compose。

---

## File Structure

- Modify: `post/tests/phase1_contract.rs`，增加 PRD completion 契约测试。
- Modify: `post/src/state.rs`，扩展内存仓储，支持搜索、通知、公告、上传、举报、审计和统计。
- Modify: `post/src/domain/posts.rs`，增加搜索查询与结果类型。
- Modify: `post/src/domain/notifications.rs`，增加公告类型。
- Modify: `post/src/domain/mod.rs`，导出新增领域模块。
- Create: `post/src/domain/files.rs`，上传文件元信息与上传请求。
- Create: `post/src/domain/moderation.rs`，举报、审计、统计类型。
- Modify: `post/src/api.rs`，暴露搜索、通知、公告、上传、举报、统计和 WebSocket 通知快照接口。
- Modify: `post/src/api.rs`，暴露注册、当前用户和退出登录接口，并支持 `x-session-id` header。
- Modify: `post/src/pages/admin.rs`，展示统计、举报、文件、分类标签和公告推送入口。
- Modify: `post/README.md`，补充新增 API 与当前 RustFS/NATS/Elasticsearch 边界。

## Task 1: PRD Completion Contract Tests

- [x] **Step 1: Write failing tests**

Add tests for search, upload validation, notifications, announcement push, report handling, audit logs and admin stats in `post/tests/phase1_contract.rs`.

- [x] **Step 2: Run tests to verify failure**

Run: `cd post && cargo test prd_completion`

Expected: FAIL because the new store methods and domain types are not defined.

## Task 2: Domain Types and Store Behavior

- [x] **Step 1: Implement minimal domain types**

Add `SearchQuery`, `FileUploadRequest`, `StoredFile`, `Report`, `AuditLogEntry`, `AdminStats`, and announcement types with serde derives.

- [x] **Step 2: Implement store methods**

Add methods for `search_posts`, `upload_file`, `list_notifications`, `mark_all_notifications_read`, `publish_announcement`, `create_report`, `resolve_report`, `audit_logs`, and `admin_stats`.

- [x] **Step 3: Run tests**

Run: `cd post && cargo test prd_completion`

Expected: PASS.

## Task 3: HTTP/API and Admin Surface

- [x] **Step 1: Expose JSON API endpoints**

Add routes for `/api/search/posts`, `/api/notifications`, `/api/announcements`, `/api/files`, `/api/reports`, `/api/admin/stats`, and `/api/ws/notifications`.

- [x] **Step 2: Update visible admin page**

Show stats, moderation, announcement, search index, file and audit sections.

- [x] **Step 3: Run full verification**

Run: `cd post && cargo test` and `cd post && cargo check`.

Expected: PASS.

## Task 4: Auth and Session Surface

- [x] **Step 1: Write failing tests**

Add tests for registration, duplicate username conflict, session lookup, logout invalidation and auth API route coverage.

- [x] **Step 2: Implement store session behavior**

Add `ForumStore::register`, `ForumStore::current_user`, `ForumStore::logout`, and shared session creation.

- [x] **Step 3: Expose API routes**

Add `/api/register`, `/api/me`, `/api/logout`, and parse `x-session-id` for session-bound requests.

- [x] **Step 4: Run auth tests**

Run: `cd post && cargo test auth_`

Expected: PASS.

## Task 5: Password Hashing and Login Verification

- [x] **Step 1: Write failing tests**

Add a contract test that unknown users cannot log in, wrong passwords fail, registered passwords are stored as non-plaintext Argon2 PHC hashes, and correct passwords create sessions.

- [x] **Step 2: Add Argon2 password hashing**

Add `argon2` and store password hashes separately from public `SessionUser` data in `ForumStore`.

- [x] **Step 3: Update login behavior**

Change `ForumStore::login` from implicit user creation to registered-user password verification.

- [x] **Step 4: Update affected tests**

Replace implicit `login` user creation in tests with explicit `register` calls.

- [x] **Step 5: Run auth password tests**

Run: `cd post && cargo test auth_password_contract_requires_registered_user_and_hashes_password`

Expected: PASS.

## Task 6: RBAC Backend Enforcement

- [x] **Step 1: Write failing tests**

Add contract tests proving seed admin has `announcement:publish`, `report:resolve`, `stats:view`, and `audit:view`, while regular users are rejected.

- [x] **Step 2: Store user permissions**

Add per-user permission storage to `ForumStore`; seed admin receives `admin_permissions()`, newly registered users receive no admin permissions.

- [x] **Step 3: Add permission checks**

Add `permissions_for_user`, `require_permission`, and API-level `authorize_session_for_permission`.

- [x] **Step 4: Protect management handlers**

Require `x-session-id` and matching permission for announcement publishing, report resolving, admin stats, and audit logs.

- [x] **Step 5: Run RBAC tests**

Run: `cd post && cargo test permission`

Expected: PASS.

## Task 7: User Action Session Enforcement

- [x] **Step 1: Write failing tests**

Add a contract test proving API-level user action authorization rejects missing, invalid, and logged-out sessions, and accepts a valid session.

- [x] **Step 2: Add shared session authorization**

Expose `authorize_session_user` and reuse it from permission authorization so user and admin checks share the same session boundary.

- [x] **Step 3: Protect user action handlers**

Require `x-session-id` for post creation, comment creation, like, favorite, follow, notification reads, uploads, reports, and WebSocket notification snapshots.

- [x] **Step 4: Run session enforcement tests**

Run: `cd post && cargo test user_action_api_contract_requires_valid_session`

Expected: PASS.

## Task 8: Profile, CRUD, and Management Surface

- [x] **Step 1: Write failing tests**

Add contract tests for profile updates, owner-only post/comment CRUD, admin user disabling, admin post/comment moderation, category/tag creation, and API route coverage.

- [x] **Step 2: Add domain types**

Add `UserStatus`, `UpdateProfileRequest`, `UpdatePostRequest`, and taxonomy models for categories and tags.

- [x] **Step 3: Implement Store behavior**

Implement profile updates, owner/admin post updates and deletes, owner/admin comment deletes, user disabling, post status management, category/tag creation, and audit logging for admin actions.

- [x] **Step 4: Expose API routes**

Add profile, post CRUD, comment delete, category/tag list, and management routes for users, posts, comments, categories, and tags.

- [x] **Step 5: Run CRUD and management tests**

Run: `cd post && cargo test user_profile_and_owner_content_crud_contract`, `cd post && cargo test admin_management_contract_covers_users_content_and_taxonomy`, and `cd post && cargo test admin_and_crud_api_routes_cover_prd_management_surface`.

Expected: PASS.

## Task 9: Live Notification Push

- [x] **Step 1: Write failing tests**

Add a contract test proving a subscribed user receives newly created notifications without reconnecting.

- [x] **Step 2: Add store subscriptions**

Add per-user notification subscribers to `ForumStore`, persist notifications as before, and broadcast new notifications to active subscribers.

- [x] **Step 3: Keep WebSocket connections live**

Update `/api/ws/notifications` so it sends the initial snapshot, subscribes the current session user, and forwards future notifications as `notification.created` messages.

- [x] **Step 4: Run notification tests**

Run: `cd post && cargo test notification_subscriber_receives_new_notifications_without_reconnect`.

Expected: PASS.

## Task 10: NATS Event Boundary

- [x] **Step 1: Write failing tests**

Add contract tests proving key forum actions record PRD event subjects and NATS payloads serialize with stable `event_type` names.

- [x] **Step 2: Add event model**

Add `ForumEvent` variants for user registration, follows, post CRUD, comments, announcements, notifications, and search indexing/deletion events.

- [x] **Step 3: Add async-nats publisher boundary**

Add `NatsEventPublisher` using current async-nats connect and publish APIs, with a testable JSON payload function.

- [x] **Step 4: Record events from Store actions**

Record events in `ForumStore` when users register, follow, create/update/delete posts, comment/reply, like posts, publish announcements, create notifications, and update search index state.

- [x] **Step 5: Run event tests**

Run: `cd post && cargo test forum_actions_record_prd_nats_events` and `cd post && cargo test nats_event_payload_contract_serializes_json_subject_and_event_type`.

Expected: PASS.

## Task 11: Elasticsearch Index Boundary

- [x] **Step 1: Fetch current docs**

Use Context7 for the official Elasticsearch Rust client. Current examples use `Elasticsearch`, `SearchParts`, JSON request bodies, and async `.send().await`.

- [x] **Step 2: Write failing tests**

Add contract tests for mapping `search.post.index` / `search.post.delete` events to index operations and for constructing a multi-field search body.

- [x] **Step 3: Add search module**

Add `SearchPostDocument`, `SearchIndexOperation`, and `ElasticsearchPostIndexer` with SSR-only official client calls for index, delete, and search.

- [x] **Step 4: Add Elasticsearch dependency**

Add the official `elasticsearch` crate as an SSR-only dependency and compile the boundary.

- [x] **Step 5: Run Elasticsearch tests**

Run: `cd post && cargo test elasticsearch_`.

Expected: PASS.

## Task 12: RustFS Object Storage Boundary

- [x] **Step 1: Fetch current docs**

Use Context7 for the AWS SDK for Rust S3 client. Current examples create an S3 client from config and use `put_object` with a byte stream body.

- [x] **Step 2: Write failing tests**

Add contract tests for converting upload requests into RustFS/S3 object uploads and for deduplicating metadata by user, bucket, key, and hash.

- [x] **Step 3: Add storage module**

Add `ObjectUpload` and `RustFsObjectStore`, with SSR-only AWS S3 SDK calls using path-style access for RustFS compatibility.

- [x] **Step 4: Normalize metadata generation**

Use shared bucket and safe filename helpers so metadata and object uploads generate the same bucket, key, and public URL. Deduplicate repeated metadata uploads for the same object hash.

- [x] **Step 5: Run storage tests**

Run: `cd post && cargo test rustfs_` and `cd post && cargo test file_metadata_upload_contract_deduplicates_same_hash_for_user`.

Expected: PASS.

## Self-Review

- Spec coverage: This plan covers the PRD items that were not already exercised by Phase 1: registration/session management, profile updates, owner content CRUD, user action session enforcement, Argon2 password hashing, login verification, RBAC backend permission enforcement, admin user/content/category/tag management, search, upload metadata, notifications, announcements, reports, audit logs, admin stats, WebSocket notification snapshot and live notification push, NATS event subjects and publisher boundary, Elasticsearch index/search client boundary, RustFS/S3 object upload client boundary, and API boundaries. Multipart/binary upload API, background NATS delivery/consumers, Elasticsearch consumer wiring and PostgreSQL password-hash persistence remain integration work behind the same boundaries.
- Placeholder scan: No open placeholders are required for the implementation tasks.
- Type consistency: All planned public methods are named from the new tests and implemented in `ForumStore`.
