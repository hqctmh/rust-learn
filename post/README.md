# Leptos 技术论坛

这是基于 `prd.md` 的独立 Leptos 全栈论坛项目。当前主干已经覆盖首页聚合、注册登录、帖子流、Markdown 发帖、详情、评论、点赞、收藏、关注、通知与 WebSocket 实时推送、管理端 RBAC、审计日志、统计面板、文件上传、搜索和本地依赖环境。

## 技术栈

- Leptos SSR + Axum
- SQLx + PostgreSQL，运行时仓储使用 SQLx checked macros
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

可选运行时开关：

```bash
SEARCH_BACKEND=postgres
ELASTICSEARCH_SEARCH_INDEX=posts
HOME_SIDEBAR_CACHE_ENABLED=false
HOME_SIDEBAR_CACHE_TTL_SECONDS=60
RUST_LOG=post=info,tower_http=info,axum=info
```

通知 WebSocket 实时推送：

- 服务端路由：`/ws/notifications/{user_id}`
- 服务端推送 JSON：`{"type":"notification","push_id":"...","notification_id":"...","title":"...","body":"..."}`
- 客户端确认 JSON：`{"type":"ack","push_id":"..."}`

## 运行

安装 Leptos 相关工具后运行：

```bash
cargo leptos serve
```

普通 Rust 检查：

```bash
cargo check
cargo test
cargo leptos build
env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml
```

启动后访问：

- 首页：`http://127.0.0.1:3000`
- 发帖页：`http://127.0.0.1:3000/posts/new`
- 管理端：`http://127.0.0.1:3000/admin`

## API

当前 JSON API 会通过 `AppState` 优先使用 PostgreSQL 运行时仓储；没有数据库连接时保留 demo store fallback，便于本地界面和合同测试快速启动。

- `GET /api/home`：首页聚合数据，包含帖子表格流、分类统计、热门标签、公告、活跃作者、登录态和分页信息。
- `GET /api/search`：帖子、标签和用户搜索；默认 PostgreSQL 后端，可切换 Elasticsearch。
- `POST /api/login`：登录或创建演示 session。
- `POST /api/register`：注册账号并创建 session。
- `POST /api/logout`：退出登录。
- `GET /api/posts`：帖子列表。
- `POST /api/posts`：发布帖子，服务端会转义 Markdown 渲染结果中的 HTML。
- `GET /api/posts/{post_id}`：帖子详情，并增加浏览计数。
- `GET /api/posts/{post_id}/comments?page=1&page_size=20`：分页评论列表。
- `POST /api/posts/{post_id}/comments`：发表评论或回复。
- `POST /api/posts/{post_id}/like`：点赞或取消点赞。
- `POST /api/posts/{post_id}/favorite`：收藏或取消收藏。
- `POST /api/users/{user_id}/follow`：关注或取消关注。
- `POST /api/files/binary`：图片二进制上传，写入 RustFS 并保存文件元信息。
- `GET /api/notifications`：通知中心数据。
- `GET /ws/notifications/{user_id}`：WebSocket 实时通知连接。
- `GET /api/admin/dashboard`：管理端首页数据，包含 RBAC 菜单、统计、审计、治理队列。

示例：

```bash
curl -sS http://127.0.0.1:3000/api/posts

curl -sS -X POST http://127.0.0.1:3000/api/posts \
  -H 'Content-Type: application/json' \
  -d '{"title":"API 发布测试","markdown":"# hello","summary":"通过 API 创建","category_name":"Leptos","tag_names":["api","rust"],"publish":true}'
```

## 迁移

数据库 migration 位于 `migrations/`，包含核心业务表和 `integration_outbox`。本地数据库启动后可使用 SQLx CLI 执行迁移：

```bash
sqlx migrate run
```

## 当前阶段边界

当前实现已经具备 PostgreSQL 运行时仓储、Redis 首页侧栏缓存边界、RustFS 对象上传、NATS/Redis/Elasticsearch `integration_outbox`、WebSocket 通知推送和 HTTP tracing。默认搜索后端仍是 PostgreSQL，本地或集成环境可通过 `SEARCH_BACKEND=elasticsearch` 和 `ELASTICSEARCH_SEARCH_INDEX=posts` 切换到 Elasticsearch。

外部服务 live e2e 测试默认 `ignored`，需要 PostgreSQL、Redis、NATS、RustFS 和 Elasticsearch 都可访问后手动运行。
