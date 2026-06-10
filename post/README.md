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

- `POST /api/login`：登录或创建演示 session。
- `GET /api/posts`：帖子列表。
- `POST /api/posts`：发布帖子，服务端会转义 Markdown 渲染结果中的 HTML。
- `GET /api/posts/{post_id}`：帖子详情，并增加浏览计数。
- `GET /api/posts/{post_id}/comments`：评论列表。
- `POST /api/posts/{post_id}/comments`：发表评论或回复。
- `POST /api/posts/{post_id}/like`：点赞或取消点赞。
- `POST /api/posts/{post_id}/favorite`：收藏或取消收藏。
- `POST /api/users/{user_id}/follow`：关注或取消关注。

示例：

```bash
curl -sS http://127.0.0.1:3000/api/posts

curl -sS -X POST http://127.0.0.1:3000/api/posts \
  -H 'Content-Type: application/json' \
  -d '{"title":"API 发布测试","markdown":"# hello","summary":"通过 API 创建","category_name":"Leptos","tag_names":["api","rust"],"publish":true}'
```

## 迁移

第一阶段 migration 位于 `migrations/202606100001_phase1.sql`。本地数据库启动后可使用 SQLx CLI 执行迁移：

```bash
sqlx migrate run
```

## 当前阶段边界

第一阶段先保证论坛页面、核心领域模型、JSON API 和本地依赖环境可运行。Redis、NATS、RustFS、Elasticsearch 已进入 Docker Compose；真实 PostgreSQL 仓储、缓存、事件消费、对象存储上传和全文索引在后续阶段接入。
