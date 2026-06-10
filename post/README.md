# Leptos 技术论坛

这是基于 `prd.md` 的独立 Leptos 全栈论坛项目。第一阶段目标是交付可运行主干：注册登录、帖子流、Markdown 发帖、详情、评论、点赞、收藏、关注、通知入口、管理端 RBAC 骨架和本地依赖环境。

## 技术栈

- Leptos SSR + Axum
- SQLx + PostgreSQL
- DaisyUI + Tailwind CSS
- Redis、NATS、RustFS、Elasticsearch

## 本地依赖

启动依赖：

```bash
docker compose up -d
```

默认端口：

- PostgreSQL: `localhost:5433`
- Redis: `localhost:6380`
- NATS: `localhost:4222`
- NATS monitor: `localhost:8222`
- RustFS S3: `localhost:9000`
- RustFS console: `localhost:9001`
- Elasticsearch: `localhost:9200`

## 配置

复制环境变量示例：

```bash
cp .env.example .env
```

开发时 SQLx 编译期校验依赖 `DATABASE_URL` 指向已迁移的 PostgreSQL，当前默认值是：

```bash
postgres://post:post@localhost:5433/post
```

## 运行

安装 Leptos 相关工具后运行：

```bash
cargo leptos serve
```

普通 Rust 检查：

```bash
cargo check
cargo test
```

启动后访问：

- 首页：`http://127.0.0.1:3000`
- 发帖页：`http://127.0.0.1:3000/posts/new`
- 管理端：`http://127.0.0.1:3000/admin`

## API

当前阶段提供内存仓储版本的 JSON API，用于跑通论坛主链路；PostgreSQL schema 已准备好，后续可以把 `ForumStore` 替换为 SQLx 仓储。

- `POST /api/register`：注册用户并返回 session。
- `POST /api/login`：校验已注册用户密码并返回 session。
- `GET /api/me`：通过 `x-session-id` header 查看当前用户。
- `POST /api/logout`：通过 `x-session-id` header 退出登录并使 session 失效。
- `PATCH /api/users/me/profile`：修改当前用户昵称、头像和简介，需要 `x-session-id`。
- `GET /api/posts`：帖子列表。
- `POST /api/posts`：发布帖子，需要 `x-session-id`；服务端会转义 Markdown 渲染结果中的 HTML。
- `GET /api/posts/{post_id}`：帖子详情，并增加浏览计数。
- `PATCH /api/posts/{post_id}`：编辑自己的帖子；管理员可通过权限编辑任意帖子。
- `DELETE /api/posts/{post_id}`：删除自己的帖子；管理员可通过权限删除任意帖子。
- `GET /api/posts/{post_id}/comments`：评论列表。
- `POST /api/posts/{post_id}/comments`：发表评论或回复，需要 `x-session-id`。
- `DELETE /api/comments/{comment_id}`：删除自己的评论；管理员可通过权限删除任意评论。
- `POST /api/posts/{post_id}/like`：点赞或取消点赞，需要 `x-session-id`。
- `POST /api/posts/{post_id}/favorite`：收藏或取消收藏，需要 `x-session-id`。
- `POST /api/users/{user_id}/follow`：关注或取消关注，需要 `x-session-id`。
- `GET /api/categories`：分类列表。
- `GET /api/tags`：标签列表。
- `GET /api/search/posts`：按关键词、分类、标签搜索帖子。
- `GET /api/notifications`：查看当前用户站内通知，需要 `x-session-id`。
- `POST /api/notifications/read-all`：全部标记已读，需要 `x-session-id`。
- `POST /api/announcements`：发布公告，并写入目标用户通知；需要 `announcement:publish` 权限。
- `POST /api/files`：校验并记录图片上传元数据，需要 `x-session-id`，保留 RustFS 对象存储边界。
- `POST /api/reports`：提交帖子、评论或用户举报，需要 `x-session-id`。
- `POST /api/reports/{report_id}/resolve`：处理举报并记录审计日志；需要 `report:resolve` 权限。
- `GET /api/admin/users`：查看用户列表；需要 `user:view` 权限。
- `PATCH /api/admin/users/{user_id}/disabled`：禁用或解禁用户；需要 `user:disable` 权限。
- `PATCH /api/admin/posts/{post_id}/status`：下架、恢复或删除帖子；需要 `post:update` 权限。
- `DELETE /api/admin/comments/{comment_id}`：管理端删除评论；需要 `comment:delete` 权限。
- `POST /api/admin/categories`：创建分类；需要 `category:create` 权限。
- `POST /api/admin/tags`：创建标签；需要 `tag:create` 权限。
- `GET /api/admin/stats`：查看管理端统计；需要 `stats:view` 权限。
- `GET /api/admin/audit-logs`：查看审计日志；需要 `audit:view` 权限。
- `GET /api/ws/notifications`：WebSocket 通知入口，需要 `x-session-id`，连接后发送当前用户未读通知快照，并继续推送新通知。

## 事件

`ForumStore` 会在注册、关注、发帖、改帖、删帖、评论、回复、点赞、公告发布和通知创建时记录 PRD 建议的事件名，例如 `user.registered`、`post.created`、`post.commented`、`post.liked`、`announcement.published`、`notification.created`、`search.post.index` 和 `search.post.delete`。

`post::events::NatsEventPublisher` 提供 async-nats 发布边界：

```rust
let publisher = post::events::NatsEventPublisher::connect("nats://localhost:4222").await?;
publisher.publish(&event).await?;
```

## 搜索索引

`post::search::ElasticsearchPostIndexer` 提供官方 Elasticsearch Rust 客户端边界：

- `SearchIndexOperation::from_event`：把 `search.post.index` / `search.post.delete` 事件转换为索引或删除操作。
- `ElasticsearchPostIndexer::search_body`：生成覆盖标题、摘要、正文、标签、分类的 `multi_match` 查询体，并保留分类、标签过滤和分页。
- SSR 构建下可通过 `ElasticsearchPostIndexer::new("http://localhost:9200", "forum_posts")` 建立客户端并调用 `apply` / `search`。

## 对象存储

`post::storage::RustFsObjectStore` 提供 RustFS/S3 兼容上传边界：

- `ObjectUpload::from_file_request`：把上传请求和二进制内容转换为 bucket、object key、content type 和 body。
- object key 使用 `{user_id}/{sha256}/{filename}`，避免路径穿越并支持按 hash 去重。
- SSR 构建下通过 AWS S3 SDK 开启 path-style access，适配本地 RustFS endpoint。

示例：

```bash
curl -sS http://127.0.0.1:3000/api/posts

curl -sS -X POST http://127.0.0.1:3000/api/posts \
  -H 'Content-Type: application/json' \
  -H 'x-session-id: <session-id>' \
  -d '{"title":"API 发布测试","markdown":"# hello","summary":"通过 API 创建","category_name":"Leptos","tag_names":["api","rust"],"publish":true}'

curl -sS -X POST http://127.0.0.1:3000/api/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"new-author","password":"secret123","nickname":"新作者"}'
```

## 迁移

第一阶段 migration 位于 `migrations/202606100001_phase1.sql`。本地数据库启动后可使用 SQLx CLI 执行迁移：

```bash
sqlx migrate run
```

## 当前阶段边界

当前实现已覆盖论坛页面、核心领域模型、注册登录/session 校验、用户资料更新、帖子编辑删除、评论删除、分类标签管理、用户禁用、用户写操作强制登录、Argon2 密码 hash、RBAC 后端权限校验、JSON API、本地依赖环境、搜索、上传校验、RustFS/S3 对象上传客户端边界、关注发帖通知、公告通知、WebSocket 通知快照与新通知实时推送、NATS 事件模型与发布器边界、Elasticsearch 索引/查询客户端边界、举报处理、审计日志和管理端统计。

Redis、NATS、RustFS、Elasticsearch 已进入 Docker Compose；真实 PostgreSQL 仓储、后台 NATS 投递循环与消费者、multipart/binary 上传 API，以及把 NATS 搜索事件接入 Elasticsearch 后台消费者是后续集成点。现阶段对应能力以 `ForumStore` 内存仓储、Argon2 PHC 密码 hash、WebSocket 快照加实时通知广播、NATS outbox 事件、RustFS 与 Elasticsearch 客户端边界和稳定 API 边界验证 PRD 业务规则。
