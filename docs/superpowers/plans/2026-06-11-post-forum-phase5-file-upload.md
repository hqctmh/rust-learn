# Post Forum Phase 5 File Upload Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable file upload foundation for PRD 4.2, 6, and 7: validate image MIME/size, store file metadata, produce Markdown-usable URLs, and expose upload metadata through API and editor UI.

**Architecture:** Add `domain::files` for upload request/metadata/limits. `ForumStore` stores file metadata in memory and validates uploader, MIME type, and size. `POST /api/files` accepts JSON metadata for the demo path and returns a `FileAsset` with bucket/key/hash/url fields. `POST /api/files/binary` accepts base64 image bytes, computes server-side SHA-256, validates the derived metadata, writes the object through a RustFS/S3-compatible adapter in PostgreSQL runtime mode, and returns the same Markdown-ready `FileAsset`.

**Tech Stack:** Rust 2024, Axum 0.8 JSON API, Leptos 0.8, existing in-memory `ForumStore`, RustFS through the S3-compatible `aws-sdk-s3 = "1.135.0"` client.

---

## Scope

This slice does not stream multipart content yet. It implements the validation, server-side binary decoding/hash contract, object-store payload, RustFS/S3-compatible object write for PostgreSQL runtime mode, metadata persistence, and Markdown URL shape needed by the editor and database schema.

## Tasks

### Task 1: Contract Tests

- [x] Valid PNG/JPEG/WebP upload returns `FileAsset` with `/uploads/...` URL and Markdown image snippet.
- [x] Unsupported MIME type is rejected.
- [x] Oversized file is rejected.
- [x] Binary/base64 upload computes server-side hash and file size.
- [x] Binary/base64 upload exposes object-store payload with bucket, key, content type, and bytes.
- [x] RustFS object-store adapter uses S3-compatible endpoint, credentials, path-style mode, and content-type object upload.
- [x] API inventory includes `/api/files` and `/api/files/binary`.
- [x] Editor inventory mentions MIME, size, URL, and Markdown insertion.

### Task 2: Domain and Store

- [x] Create `post/src/domain/files.rs`.
- [x] Expose module in `domain/mod.rs`.
- [x] Add `ForumData.files`.
- [x] Add `ForumStore::upload_file`.
- [x] Add `FileBinaryUploadRequest`.
- [x] Add `FileObjectUpload`.
- [x] Add `ForumStore::upload_binary_file`.
- [x] Add `AppState::upload_binary_file`.
- [x] Add `RustfsObjectStore` and runtime RustFS config mapping.

### Task 3: API and Editor

- [x] Add `POST /api/files`.
- [x] Add `POST /api/files/binary`.
- [x] Add `/api/files` and `/api/files/binary` to inventory.
- [x] Extend editor preview with upload contract and example Markdown image URL.

### Task 4: Verification

- [x] `cargo fmt`
- [x] `cargo test`
- [x] `cargo check`
- [x] `cargo leptos build`
- [x] IDEA error check not required by project instruction.
- [x] API/browser verify upload contract and editor page.

## Self-Review

- Covers PRD file metadata, MIME/size limits, generated URL, Markdown image use, and RustFS-ready shape.
- Covers base64 binary JSON upload with server-side SHA-256 and file size derivation.
- Covers object-store payload generation so the RustFS adapter can consume bytes, bucket, key, and content type without reparsing the API body.
- Covers actual RustFS/S3-compatible object persistence for PostgreSQL runtime mode before metadata is inserted.
- Leaves multipart streaming as a follow-up; current runnable API uses base64 JSON upload for the editor contract.

## Current verification evidence (2026-06-12)

- `cargo test --manifest-path post/Cargo.toml file_`: PASS, 8 passed.
- `cargo test --manifest-path post/Cargo.toml rustfs_object_store_adapter_contract_uses_s3_put_object`: PASS, 1 passed.
- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 103 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS. First sandbox run failed on Cargo registry cache permission for `web-time-1.1.0.crate`; non-sandbox rerun completed successfully.
- Project instruction says this project does not need IDEA MCP problem checks.
- `docker compose pull rustfs`: PASS after retry; `rustfs/rustfs:latest` pulled successfully.
- `docker compose up -d rustfs`: PASS; `post-rustfs` started on ports 9000/9001.
- API upload verification through `POST /api/files/binary?user_id=03ab4ea2-2a58-43ab-8f67-9ab89c419d8e`: PASS; returned `file_id=77c145df-a405-4cea-b67c-66c678f143f7`, `storage_key=markdown/2cf24dba5fb0/rustfs-unique-3117bde0.png`.
- RustFS object verification: PASS; `post-rustfs:/data/post-assets/markdown/2cf24dba5fb0/rustfs-unique-3117bde0.png/xl.meta` exists.
- PostgreSQL metadata verification: PASS; `file_assets` row exists for `77c145df-a405-4cea-b67c-66c678f143f7`.
- Browser editor verification at `/posts/new`: PASS; page shows Markdown editor and upload contract text for RustFS storage, MIME validation, file size limit, and Markdown image link generation.
- `aws-sdk-s3` default feature configuration verification: PASS; `post/Cargo.toml` uses `aws-sdk-s3 = { version = "1.135.0", optional = true }`, with `aws-smithy-eventstream` locked to `0.60.20` to avoid the `aws-runtime v1.7.4` E0282 compile issue seen with `0.60.21`.
- API upload verification through `POST /api/files/binary?user_id=0979af79-f821-43f1-842f-8dc94ae2e226`: PASS; returned `file_id=3588e0c1-978f-4fa7-926f-f43c6ee79bd5`, `storage_key=markdown/7ef42b07f828/aws-sdk-s3-default-6f2b8e.png`.
- RustFS object verification for AWS SDK upload: PASS; `post-rustfs:/data/post-assets/markdown/7ef42b07f828/aws-sdk-s3-default-6f2b8e.png/xl.meta` exists.
- PostgreSQL metadata verification for AWS SDK upload: PASS; `file_assets` row exists for `3588e0c1-978f-4fa7-926f-f43c6ee79bd5`.
