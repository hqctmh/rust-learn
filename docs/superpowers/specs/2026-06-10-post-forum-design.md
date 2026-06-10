# Leptos 技术论坛系统设计

## 背景

`post/prd.md` 要求新增一个基于 Leptos 全栈模式的技术论坛系统，覆盖用户端、管理端、Markdown 发帖、图片上传、评论回复、点赞、关注、通知推送、公告、搜索、内容管理、RBAC、审计日志和 Docker Compose 依赖环境。

当前仓库是 Rust 学习仓库，已有 `sqlx-demo` 作为 Axum + SQLx + PostgreSQL 的极简接口示例。论坛系统不应混入该 demo，避免把学习代码和产品代码耦合在一起。

## 决策

新建独立 `post` Leptos 全栈项目，使用 Leptos SSR + Axum 实现主应用。第一阶段先用内存仓储跑通核心论坛链路，同时落地 PostgreSQL schema 和 SQLx 依赖边界；后续把 `ForumStore` 替换为 SQLx 仓储。DaisyUI/Tailwind 用作 UI 样式层。Redis、NATS、RustFS、Elasticsearch 先进入 Docker Compose 与配置边界，业务代码以接口/模块隔离，逐步接入真实缓存、事件、对象存储和搜索索引。

这是最合理的路径，因为 PRD 明确要求 Leptos 全栈项目，而现有 `sqlx-demo` 不是全栈结构，也没有管理端、SSR、资源构建和页面路由边界。

## 范围拆分

论坛系统按三个阶段推进。

第一阶段交付可运行主干：

- 演示登录/session 结构和后续 cookie 会话边界。
- 首页帖子列表，提供页面入口和 JSON API。
- 帖子发布、详情、草稿/发布状态模型和 Markdown 预览。
- Markdown 渲染后的 HTML 清洗，避免 XSS。
- 评论和二级回复的领域/API 链路。
- 帖子点赞/取消点赞的领域/API 链路。
- 收藏/取消收藏的领域/API 链路。
- 关注/取消关注的领域/API 链路。
- 通知表、未读通知列表和 WebSocket 推送接口骨架。
- 管理端登录、RBAC 基础模型、帖子和评论管理入口。
- Docker Compose 启动 PostgreSQL、Redis、NATS、RustFS、Elasticsearch。
- README 描述本地启动、迁移、运行和测试。

第二阶段补全业务深度：

- RustFS 真实上传、文件 hash 去重和 Markdown 图片插入。
- NATS 事件消费者，处理关注发帖、评论、回复、点赞、公告推送。
- Elasticsearch 帖子全文搜索、用户搜索、标签搜索和索引同步。
- 举报处理、公告发布与推送、分类标签后台管理。
- 审计日志、操作日志、系统统计。

第三阶段优化：

- 推荐算法、热门榜单、草稿自动保存增强、SEO、多端适配、内容审核流程。

## 技术约束

Leptos 文档结论：

- 采用 SSR + hydration，服务端先渲染 HTML，浏览器端再通过 WASM 激活交互。
- Leptos 可与 Axum 集成；server functions 可通过 context 和 Axum state 获取应用状态。
- Axum state 需要同时传入 Leptos routes context 和 `Router::with_state`，保证 server functions 和普通 handler 使用同一个应用状态。

SQLx 文档结论：

- PostgreSQL 连接通过 `PgPool` 管理。
- `query!`、`query_as!` 依赖 schema 做编译期 SQL 校验，开发时需要有效 `DATABASE_URL` 或离线元数据。
- 多表写入和计数更新使用事务封装，避免点赞、收藏、关注等唯一约束与计数不一致。

DaisyUI 文档结论：

- DaisyUI v5 通过 CSS 中 `@import "tailwindcss"; @plugin "daisyui";` 启用。
- 页面使用 DaisyUI 语义组件类，例如 `btn`、`card`、`navbar`、`tabs`、`badge`、`table`，再结合 Tailwind 工具类做布局。

## 架构

`post` 项目使用单体全栈结构：

- `src/app.rs`：Leptos 路由和页面壳。
- `src/pages/`：用户端页面和管理端页面。
- `src/components/`：导航、帖子卡片、Markdown 编辑器、评论树、通知入口、管理表格。
- `src/server/`：server functions，负责页面调用的服务端动作。
- `src/domain/`：领域服务，包含认证、帖子、评论、互动、关注、通知、RBAC、上传、搜索。
- `src/db/`：SQLx 查询和事务封装。
- `src/http/`：普通 Axum API 和 WebSocket handler。
- `migrations/`：PostgreSQL schema。
- `style/`：Tailwind/DaisyUI 样式入口。

`AppState` 持有：

- `PgPool`。
- 可选 Redis/NATS/RustFS/Elasticsearch 客户端包装。
- 会话配置。
- Leptos SSR 配置。

第一阶段中，Redis/NATS/RustFS/Elasticsearch 的运行环境和配置先可用；业务链路先由内存仓储跑通可交互 API，PostgreSQL migration 作为数据合同，避免在页面和领域边界未稳定前引入多系统一致性问题。

## 数据模型

核心表：

- `users`：账号、密码 hash、昵称、头像、简介、状态。
- `sessions`：cookie session，存储用户和过期时间。
- `roles`、`permissions`、`user_roles`、`role_permissions`：RBAC。
- `categories`、`tags`、`posts`、`post_contents`、`post_tags`：帖子和分类标签。
- `comments`：一级评论和二级回复，删除后保留占位。
- `post_likes`、`comment_likes`：点赞唯一约束。
- `post_favorites`：收藏唯一约束。
- `follows`：关注唯一约束，禁止关注自己。
- `notifications`：站内通知和未读状态。
- `announcements`、`announcement_reads`：公告和已读状态。
- `files`：上传文件元信息。
- `reports`：举报。
- `audit_logs`：管理端审计日志。

## 权限规则

- 未登录用户可以浏览公开内容、查看评论、使用基础搜索。
- 登录用户可以创建内容、评论、回复、点赞、收藏、关注、查看通知。
- 用户只能修改或删除自己的帖子和评论。
- 管理端接口必须校验 RBAC 权限，即使前端隐藏按钮也不能绕过后端校验。
- WebSocket 建连必须校验 session。
- 上传、点赞、收藏、关注、发帖和评论必须登录。

## 错误处理

服务端统一返回结构化错误：

- `401`：未登录或 session 失效。
- `403`：登录但无权限。
- `404`：资源不存在或不可见。
- `409`：唯一约束冲突，例如重复点赞、重复收藏、重复关注。
- `422`：请求字段校验失败。
- `500`：内部错误，日志记录详细原因，对用户返回通用消息。

Markdown 渲染和上传校验必须失败关闭：不允许不安全 HTML 和不合规 MIME 类型进入页面渲染。

## UI 设计

第一阶段 UI 是工作型论坛产品，不做营销页。首屏直接进入论坛首页：

- 顶部导航：站点名、搜索入口、分类入口、通知入口、登录/用户菜单。
- 主内容：帖子流，按推荐、最新、热门切换。
- 右侧栏：分类、标签、公告、热门作者。
- 发帖页：标题、摘要、分类、标签、Markdown 编辑区、实时预览、发布/草稿动作。
- 详情页：正文、作者信息、点赞收藏关注、评论树。
- 管理端：左侧菜单，右侧表格和操作按钮，保持高密度、清晰、可扫描。

视觉方向使用 DaisyUI 的简洁组件体系，颜色以浅色工作台为主，避免装饰性过强。交互优先完成真实流程，避免不可用的展示控件。

## 测试策略

核心业务先用 TDD：

- 认证：注册、登录、退出、session 过期。
- 帖子：创建、更新、删除、列表、详情、权限校验。
- 评论：一级评论、二级回复、删除占位。
- 点赞/收藏/关注：唯一约束、取消操作、禁止关注自己。
- RBAC：管理权限校验。

集成验证：

- `cargo test` 覆盖领域和 server functions。
- `cargo check` 覆盖 Rust 类型和 SQLx 查询。
- `cargo leptos build` 覆盖 SSR/WASM 构建。
- Docker Compose 健康检查覆盖基础设施可启动。
- 浏览器验证覆盖首页、发帖、详情、登录、管理入口。
- IDEA problem 检查修复 error 级问题。

## 验收映射

第一阶段完成后，应能证明：

- Docker Compose 可以启动 PRD 要求的依赖服务。
- 应用可以运行并打开首页。
- 用户/session 结构可用，演示登录 API 可创建 session。
- 未登录用户可以浏览帖子和评论。
- API 可以发布 Markdown 帖子、评论、回复、点赞、收藏、关注，Markdown HTML 会做转义。
- 通知有持久化数据和 WebSocket 入口。
- 管理端有登录、RBAC 模型和帖子/评论管理入口。
- README 有本地开发说明。

完整 PRD 完成需要继续交付第二、三阶段，尤其是 RustFS 真实上传、NATS 消费者、Elasticsearch 搜索、公告推送、举报、审计和统计。
