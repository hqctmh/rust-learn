# Turn Store Relay Chat Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 完成可复用 Conversation 的 Axum/Redis Stream/mock SSE 中转服务，并交付同源的流式聊天网页。

**Architecture:** 首次 POST 在事务内创建 Conversation 与 Turn，后续 POST 根据路径中的 `conversation_id` 只创建 Turn。每个 Turn 使用独立 Redis Stream；后台 relay 先持久化上游事件，再写入 Redis，前端 POST SSE 从 Stream 读取。Axum 同源托管原生 HTML/CSS/JavaScript，浏览器使用 `fetch` 和增量 SSE parser 显示文本。

**Tech Stack:** Rust 1.97、Axum 0.8、Tokio、SQLx/PostgreSQL、redis-rs 1.x、reqwest、eventsource-stream、原生 JavaScript、Node 25 内置测试框架。

## Global Constraints

- 首次消息使用 `POST /api/conversations/stream`，后续消息使用 `POST /api/conversations/{conversation_id}/turns/stream`。
- 只有发起新对话时创建 Conversation；同一网页会话的后续消息必须复用 `conversation_id`。
- Redis Stream key 固定为 `turn:{turn_id}:events`，每次 XADD 刷新 TTL。
- 每个上游事件先写 PostgreSQL，再写 Redis Stream。
- 第一版只有合法 `text` 事件可追加；同一 `(turn_id, type)` 只能有一条 `appendable = true` 的 TurnResponse。
- `run_completed` 和 `error` 是唯一终止事件；默认空闲超时为 60 秒，keep-alive 为 10 秒。
- mock 的 fast 保持一行一个 text，slow 保持每个 text 5 至 10 个字符。
- 网页使用纯文本渲染，不引入前端框架、Markdown 渲染、认证、历史恢复或真正的上游取消。
- 不覆盖 `turn-store/.env`，不提交仓库中与本功能无关的现有改动。

---

## File Structure

- Modify: `turn-store/Cargo.toml` — 增加可选 `.env` 加载依赖。
- Modify: `turn-store/Cargo.lock` — 锁定依赖解析结果。
- Modify: `turn-store/src/main.rs` — 加载 `.env`、注入 relay 空闲超时。
- Modify: `turn-store/src/app.rs` — 扩展 `AppState` 并装配 API/网页路由。
- Modify: `turn-store/src/config.rs` — 增加 `RELAY_IDLE_TIMEOUT_SECONDS`。
- Modify: `turn-store/src/error.rs` — 增加 `404` 构造器。
- Modify: `turn-store/src/domain/model.rs` — 拆分首次与后续请求模型。
- Modify: `turn-store/src/repositories/turn_store.rs` — 为已有 Conversation 创建 Turn。
- Create: `turn-store/src/routes/conversation.rs` — 首次消息入口。
- Modify: `turn-store/src/routes/turn.rs` — 后续消息入口。
- Create: `turn-store/src/routes/stream.rs` — 共享 Redis-to-SSE 读取循环与空闲截止时间。
- Create: `turn-store/src/routes/web.rs` — 同源静态资源路由。
- Modify: `turn-store/src/routes/mod.rs` — 装配新路由。
- Modify: `turn-store/tests/relay_policy_test.rs` — 请求模型契约。
- Modify: `turn-store/tests/relay_repository_test.rs` — Conversation 复用与追加持久化测试。
- Create: `turn-store/tests/http_stream_test.rs` — 两条 POST SSE 路由集成测试。
- Create: `turn-store/tests/runtime_docs_test.rs` — 本地运行配置契约。
- Create: `turn-store/static/package.json` — 将静态目录声明为 ES module。
- Create: `turn-store/static/index.html` — 聊天页面结构。
- Create: `turn-store/static/styles.css` — 响应式聊天样式。
- Create: `turn-store/static/sse.js` — 无 DOM 的增量 SSE parser。
- Create: `turn-store/static/sse.test.mjs` — parser 单元测试。
- Create: `turn-store/static/app.js` — 多轮会话与流式渲染。
- Create: `turn-store/.env.example` — 无凭据配置示例。
- Create: `turn-store/README.md` — PostgreSQL、Redis、mock、Axum 启动与测试说明。

### Task 1: Split First-Turn and Follow-Up Request Contracts

**Files:**
- Modify: `turn-store/src/domain/model.rs`
- Modify: `turn-store/src/routes/turn.rs`
- Modify: `turn-store/tests/relay_policy_test.rs`

**Interfaces:**
- Produces: `CreateConversationStreamRequest { conversation, turn, speed }`。
- Produces: `CreateTurnStreamRequest { turn, speed }`。
- Produces: `ConversationInput::validate()` 与 `TurnInput::validate()`，供两个请求模型复用。

- [ ] **Step 1: Write the failing request-contract tests**

在 `turn-store/tests/relay_policy_test.rs` 增加：

```rust
use turn_store::domain::model::{
    ConversationInput, CreateConversationStreamRequest, CreateTurnStreamRequest, TurnInput,
    UpstreamSpeed,
};

fn valid_turn() -> TurnInput {
    TurnInput {
        input_context: "继续说明".to_string(),
        document_content_version_id: 1,
    }
}

#[test]
fn first_turn_request_validates_conversation_and_turn() {
    let request = CreateConversationStreamRequest {
        conversation: ConversationInput {
            doc_id: "doc-1".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "新对话".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        turn: valid_turn(),
        speed: UpstreamSpeed::Fast,
    };

    assert_eq!(request.validate(), Ok(()));
}

#[test]
fn follow_up_request_does_not_require_conversation_metadata() {
    let request = CreateTurnStreamRequest {
        turn: valid_turn(),
        speed: UpstreamSpeed::Slow,
    };

    assert_eq!(request.validate(), Ok(()));
}

#[test]
fn follow_up_request_rejects_empty_input_context() {
    let request = CreateTurnStreamRequest {
        turn: TurnInput {
            input_context: "   ".to_string(),
            document_content_version_id: 1,
        },
        speed: UpstreamSpeed::Fast,
    };

    assert_eq!(
        request.validate(),
        Err("turn.input_context 不能为空".to_string())
    );
}
```

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cd turn-store && cargo test --test relay_policy_test follow_up_request`

Expected: FAIL because the old `CreateTurnStreamRequest` still requires `conversation`, and `CreateConversationStreamRequest` does not exist.

- [ ] **Step 3: Implement the two request models and shared validation**

将 `turn-store/src/domain/model.rs` 中请求与校验部分改为：

```rust
impl ConversationInput {
    pub fn validate(&self) -> Result<(), String> {
        required(&self.doc_id, "conversation.doc_id")?;
        max_chars(&self.doc_id, "conversation.doc_id", 64)?;
        required(&self.doc_type, "conversation.doc_type")?;
        max_chars(&self.doc_type, "conversation.doc_type", 50)?;
        required(&self.title, "conversation.title")?;
        max_chars(&self.title, "conversation.title", 255)?;
        required(&self.r#type, "conversation.type")?;
        max_chars(&self.r#type, "conversation.type", 50)?;
        if let Some(inline_type) = &self.inline_type {
            max_chars(inline_type, "conversation.inline_type", 50)?;
        }
        Ok(())
    }
}

impl TurnInput {
    pub fn validate(&self) -> Result<(), String> {
        required(&self.input_context, "turn.input_context")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversationStreamRequest {
    pub conversation: ConversationInput,
    pub turn: TurnInput,
    #[serde(default)]
    pub speed: UpstreamSpeed,
}

impl CreateConversationStreamRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.conversation.validate()?;
        self.turn.validate()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTurnStreamRequest {
    pub turn: TurnInput,
    #[serde(default)]
    pub speed: UpstreamSpeed,
}

impl CreateTurnStreamRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.turn.validate()
    }
}
```

保留现有 `required`、`max_chars`、`UpstreamSpeed` 和领域结构。为使本任务能独立编译，在旧 `turn-store/src/routes/turn.rs` 中只做兼容性改名：把导入和 handler 参数中的 `CreateTurnStreamRequest` 改为 `CreateConversationStreamRequest`；路由路径和现有行为此时不变，Task 4 再用最终的后续 Turn handler 替换该文件。

- [ ] **Step 4: Run the focused tests and verify GREEN**

Run: `cd turn-store && cargo test --test relay_policy_test`

Expected: PASS，包含原有 5 个策略测试和本任务新增的 3 个请求测试。

- [ ] **Step 5: Commit this contract**

```bash
git add turn-store/src/domain/model.rs turn-store/src/routes/turn.rs turn-store/tests/relay_policy_test.rs
git commit -m "功能：拆分首次和后续流式请求模型"
```

### Task 2: Reuse Conversation When Creating Follow-Up Turns

**Files:**
- Modify: `turn-store/src/repositories/turn_store.rs`
- Modify: `turn-store/tests/relay_repository_test.rs`

**Interfaces:**
- Consumes: `TurnInput` from Task 1。
- Produces: `create_turn_for_conversation(db, conversation_id, turn) -> sqlx::Result<Turn>`。
- Guarantees: 不存在或 `deleted_at != 0` 的 Conversation 返回 `sqlx::Error::RowNotFound`，且不写 Turn。

- [ ] **Step 1: Write failing PostgreSQL integration tests**

在 `turn-store/tests/relay_repository_test.rs` 增加导入和测试：

```rust
use uuid::Uuid;
use turn_store::repositories::turn_store::create_turn_for_conversation;

#[sqlx::test]
async fn follow_up_turns_reuse_one_conversation(db: PgPool) -> sqlx::Result<()> {
    let (conversation, first_turn) = create_conversation_and_turn(
        &db,
        &ConversationInput {
            doc_id: "doc-multi".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "多轮对话".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        &TurnInput {
            input_context: "第一问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;

    let second_turn = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "第二问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;
    let third_turn = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "第三问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;

    let conversation_count = sqlx::query_scalar::<_, i64>(
        "select count(*) from conversation where id = $1",
    )
    .bind(conversation.id)
    .fetch_one(&db)
    .await?;
    let turn_count = sqlx::query_scalar::<_, i64>(
        "select count(*) from \"turn\" where conversation_id = $1",
    )
    .bind(conversation.id)
    .fetch_one(&db)
    .await?;

    assert_eq!(conversation_count, 1);
    assert_eq!(turn_count, 3);
    assert_eq!(first_turn.conversation_id, conversation.id);
    assert_eq!(second_turn.conversation_id, conversation.id);
    assert_eq!(third_turn.conversation_id, conversation.id);
    Ok(())
}

#[sqlx::test]
async fn follow_up_turn_rejects_missing_or_deleted_conversation(
    db: PgPool,
) -> sqlx::Result<()> {
    let missing = create_turn_for_conversation(
        &db,
        Uuid::nil(),
        &TurnInput {
            input_context: "不会写入".to_string(),
            document_content_version_id: 1,
        },
    )
    .await;
    assert!(matches!(missing, Err(sqlx::Error::RowNotFound)));

    let (conversation, _) = create_conversation_and_turn(
        &db,
        &ConversationInput {
            doc_id: "doc-deleted".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "已删除".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        &TurnInput {
            input_context: "第一问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;
    sqlx::query("update conversation set deleted_at = 1 where id = $1")
        .bind(conversation.id)
        .execute(&db)
        .await?;

    let deleted = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "不会写入".to_string(),
            document_content_version_id: 1,
        },
    )
    .await;
    assert!(matches!(deleted, Err(sqlx::Error::RowNotFound)));
    Ok(())
}
```

- [ ] **Step 2: Run the repository test and verify RED**

Run: `cd turn-store && DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres cargo test --test relay_repository_test follow_up_turn`

Expected: FAIL because `create_turn_for_conversation` is not defined.

- [ ] **Step 3: Implement one atomic INSERT…SELECT**

在 `turn-store/src/repositories/turn_store.rs` 增加：

```rust
pub async fn create_turn_for_conversation(
    db: &PgPool,
    conversation_id: Uuid,
    turn: &TurnInput,
) -> sqlx::Result<Turn> {
    sqlx::query_as::<_, Turn>(
        r#"
        insert into "turn" (conversation_id, input_context, document_content_version_id)
        select id, $2, $3
        from conversation
        where id = $1 and deleted_at = 0
        returning id, conversation_id, input_context, document_content_version_id,
                  created_at, updated_at, deleted_at
        "#,
    )
    .bind(conversation_id)
    .bind(&turn.input_context)
    .bind(turn.document_content_version_id)
    .fetch_one(db)
    .await
}
```

该单条 SQL 同时完成有效性检查和写入，避免“先 SELECT、再 INSERT”之间 Conversation 被删除的竞态。

- [ ] **Step 4: Run all repository tests and verify GREEN**

Run: `cd turn-store && DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres cargo test --test relay_repository_test`

Expected: PASS，验证 Conversation 数量保持 1、Turn 数量变为 3，缺失/删除两种情况均拒绝写入。

- [ ] **Step 5: Commit the repository behavior**

```bash
git add turn-store/src/repositories/turn_store.rs turn-store/tests/relay_repository_test.rs turn-store/migrations/20260710000000_turn_response_appendable.sql
git commit -m "功能：支持复用对话创建轮次"
```

### Task 3: Add a Testable SSE Idle Deadline

**Files:**
- Create: `turn-store/src/routes/stream.rs`
- Modify: `turn-store/src/routes/mod.rs`

**Interfaces:**
- Produces: `IdleDeadline::new(now, timeout)`、`reset(now)`、`remaining(now)`。
- Guarantees: 业务事件重置截止时间；keep-alive 和空 XREAD 不重置。

- [ ] **Step 1: Add a failing unit test before the helper exists**

先在 `turn-store/src/routes/mod.rs` 增加 `pub mod stream;`，再创建 `turn-store/src/routes/stream.rs`，仅写测试：

```rust
#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::IdleDeadline;

    #[test]
    fn idle_deadline_only_moves_after_business_activity() {
        let start = Instant::now();
        let mut deadline = IdleDeadline::new(start, Duration::from_secs(60));

        assert_eq!(
            deadline.remaining(start + Duration::from_secs(15)),
            Some(Duration::from_secs(45))
        );
        deadline.reset(start + Duration::from_secs(20));
        assert_eq!(
            deadline.remaining(start + Duration::from_secs(50)),
            Some(Duration::from_secs(30))
        );
        assert_eq!(deadline.remaining(start + Duration::from_secs(81)), None);
    }
}
```

- [ ] **Step 2: Run the unit test and verify RED**

Run: `cd turn-store && cargo test idle_deadline_only_moves_after_business_activity`

Expected: FAIL because `IdleDeadline` does not exist.

- [ ] **Step 3: Implement the pure deadline helper above the test module**

```rust
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct IdleDeadline {
    timeout: Duration,
    deadline: Instant,
}

impl IdleDeadline {
    pub fn new(now: Instant, timeout: Duration) -> Self {
        Self {
            timeout,
            deadline: now + timeout,
        }
    }

    pub fn reset(&mut self, now: Instant) {
        self.deadline = now + self.timeout;
    }

    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline.checked_duration_since(now)
    }
}
```

- [ ] **Step 4: Run the unit test and verify GREEN**

Run: `cd turn-store && cargo test idle_deadline_only_moves_after_business_activity`

Expected: PASS.

- [ ] **Step 5: Commit the timeout primitive**

```bash
git add turn-store/src/routes/mod.rs turn-store/src/routes/stream.rs
git commit -m "功能：增加流式读取空闲截止时间"
```

### Task 4: Implement the Two Axum POST SSE Routes

**Files:**
- Modify: `turn-store/Cargo.toml`
- Modify: `turn-store/Cargo.lock`
- Modify: `turn-store/src/main.rs`
- Modify: `turn-store/src/app.rs`
- Modify: `turn-store/src/config.rs`
- Modify: `turn-store/src/error.rs`
- Create: `turn-store/src/routes/conversation.rs`
- Modify: `turn-store/src/routes/turn.rs`
- Modify: `turn-store/src/routes/stream.rs`
- Modify: `turn-store/src/routes/mod.rs`
- Create: `turn-store/tests/http_stream_test.rs`

**Interfaces:**
- Consumes: request contracts from Task 1 and repository function from Task 2。
- Produces: `POST /api/conversations/stream`。
- Produces: `POST /api/conversations/{conversation_id}/turns/stream`。
- Produces: `start_turn_stream(state, conversation_id, turn_id, speed) -> Result<TurnSse, AppError>`。
- Produces: `AppState.relay_idle_timeout: Duration`。

- [ ] **Step 1: Write an end-to-end route test that initially receives 404/compile failure**

创建 `turn-store/tests/http_stream_test.rs`：

```rust
use std::{convert::Infallible, env, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::sse::{Event, Sse},
    routing::get,
};
use futures_util::stream;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use turn_store::{
    app::{AppState, build_app},
    infra::{redis_stream::RedisStream, upstream::UpstreamClient},
    services::relay::RelayService,
};

async fn start_test_upstream() -> String {
    let app = Router::new().route(
        "/events",
        get(|| async {
            let events = stream::iter([
                Ok::<_, Infallible>(Event::default().event("text").data(
                    json!({"type":"text","content":"你好"}).to_string(),
                )),
                Ok::<_, Infallible>(Event::default().event("text").data(
                    json!({"type":"text","content":"，世界"}).to_string(),
                )),
                Ok::<_, Infallible>(Event::default().event("run_completed").data(
                    json!({"type":"run_completed","content":"完成"}).to_string(),
                )),
            ]);
            Sse::new(events)
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{address}/events")
}

async fn test_app(db: PgPool) -> Router {
    let redis_url = env::var("TEST_REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6380/15".to_string());
    let redis_stream = RedisStream::connect(&redis_url, 60, 50).await.unwrap();
    let upstream = UpstreamClient::new(reqwest::Client::new(), start_test_upstream().await);
    let relay_service = RelayService::new(db.clone(), redis_stream.clone(), upstream);
    build_app(AppState {
        db,
        redis_stream,
        relay_service,
        relay_idle_timeout: Duration::from_secs(5),
    })
}

async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn event_data(body: &str, event_name: &str) -> Value {
    for block in body.split("\n\n") {
        let name = block
            .lines()
            .find_map(|line| line.strip_prefix("event:").map(str::trim));
        if name == Some(event_name) {
            let data = block
                .lines()
                .find_map(|line| line.strip_prefix("data:").map(str::trim))
                .unwrap();
            return serde_json::from_str(data).unwrap();
        }
    }
    panic!("没有找到 SSE 事件 {event_name}: {body}");
}

#[sqlx::test]
async fn first_and_follow_up_routes_reuse_conversation(db: PgPool) -> sqlx::Result<()> {
    let app = test_app(db.clone()).await;
    let first_request = json!({
        "conversation": {
            "doc_id": "web-doc",
            "doc_type": "markdown",
            "user_id": 1,
            "title": "网页对话",
            "type": "CHAT_EDIT",
            "inline_type": null
        },
        "turn": {
            "input_context": "第一问",
            "document_content_version_id": 1
        },
        "speed": "fast"
    });
    let (first_status, first_body) =
        post_json(app.clone(), "/api/conversations/stream", first_request).await;
    assert_eq!(first_status, StatusCode::OK);
    let created = event_data(&first_body, "turn_created");
    let conversation_id = created["conversation_id"].as_str().unwrap();
    assert!(first_body.contains("event:run_completed") || first_body.contains("event: run_completed"));

    let follow_up_request = json!({
        "turn": {
            "input_context": "第二问",
            "document_content_version_id": 1
        },
        "speed": "slow"
    });
    let uri = format!("/api/conversations/{conversation_id}/turns/stream");
    let (second_status, second_body) = post_json(app, &uri, follow_up_request).await;
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(
        event_data(&second_body, "turn_created")["conversation_id"].as_str(),
        Some(conversation_id)
    );

    let conversation_count =
        sqlx::query_scalar::<_, i64>("select count(*) from conversation")
            .fetch_one(&db)
            .await?;
    let turn_count = sqlx::query_scalar::<_, i64>("select count(*) from \"turn\"")
        .fetch_one(&db)
        .await?;
    let text_rows = sqlx::query_as::<_, (String, i64)>(
        "select response, count(*) over () from turn_response where type = 'text' and appendable order by created_at",
    )
    .fetch_all(&db)
    .await?;

    assert_eq!(conversation_count, 1);
    assert_eq!(turn_count, 2);
    assert_eq!(text_rows.len(), 2);
    assert!(text_rows.iter().all(|(response, count)| response == "你好，世界" && *count == 2));
    Ok(())
}

#[sqlx::test]
async fn follow_up_route_returns_404_for_missing_conversation(db: PgPool) {
    let app = test_app(db).await;
    let request = json!({
        "turn": {
            "input_context": "不会创建",
            "document_content_version_id": 1
        }
    });
    let (status, _) = post_json(
        app,
        "/api/conversations/00000000-0000-0000-0000-000000000000/turns/stream",
        request,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
```

- [ ] **Step 2: Start isolated Redis and verify RED**

Run:

```bash
docker run --rm -d --name turn-store-test-redis -p 6380:6379 redis:7-alpine
cd turn-store
DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test --test http_stream_test
```

Expected: FAIL because the two new routes and `AppState.relay_idle_timeout` are not implemented.

- [ ] **Step 3: Add config, state, error, and startup wiring**

在 `turn-store/Cargo.toml` 增加：

```toml
dotenvy = "0.15"
```

在 `Config` 增加字段并在 `from_env` 中解析：

```rust
pub relay_idle_timeout_seconds: u64,
```

```rust
relay_idle_timeout_seconds: parse_env("RELAY_IDLE_TIMEOUT_SECONDS", 60)?,
```

同时把 `bind_addr` 的本地缺省值改为：

```rust
bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
```

在 `turn-store/src/app.rs` 顶部增加 `use std::time::Duration;`，并在 `AppState` 增加：

```rust
pub relay_idle_timeout: Duration,
```

在 `main` 的配置读取前和 `AppState` 构造中增加：

```rust
dotenvy::dotenv().ok();
```

```rust
relay_idle_timeout: Duration::from_secs(config.relay_idle_timeout_seconds),
```

在 `AppError` 增加：

```rust
pub fn not_found(message: impl Into<String>) -> Self {
    Self {
        status: StatusCode::NOT_FOUND,
        message: message.into(),
    }
}
```

- [ ] **Step 4: Implement the shared Redis-to-SSE stream**

把 `turn-store/src/routes/stream.rs` 顶部的 `std` 导入替换为下面的组合导入，并在 `IdleDeadline` 后增加其余代码：

```rust
use std::{
    convert::Infallible,
    pin::Pin,
    time::{Duration, Instant},
};

use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::Stream;
use serde_json::json;
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    app::AppState,
    domain::{
        event::AgentEvent,
        model::{Turn, UpstreamSpeed},
    },
    error::AppError,
    infra::redis_stream::StreamEntry,
};

pub type TurnEventStream =
    Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send + 'static>>;
pub type TurnSse = Sse<TurnEventStream>;

pub async fn start_turn_stream(
    state: AppState,
    conversation_id: Uuid,
    turn_id: Uuid,
    speed: UpstreamSpeed,
) -> Result<TurnSse, AppError> {
    let stream_key = Turn::stream_key(turn_id);
    let created = AgentEvent::turn_created(conversation_id, turn_id);
    state.redis_stream.append(&stream_key, &created).await?;
    let mut reader = state.redis_stream.reader().await?;
    state
        .relay_service
        .spawn(turn_id, stream_key.clone(), speed);
    let idle_timeout = state.relay_idle_timeout;

    let output = async_stream::stream! {
        let mut last_id = "0-0".to_string();
        let mut idle = IdleDeadline::new(Instant::now(), idle_timeout);

        'read: loop {
            let Some(remaining) = idle.remaining(Instant::now()) else {
                let data = AgentEvent::error("等待上游事件超时").data;
                yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                break;
            };

            match timeout(remaining, reader.read_after(&stream_key, &last_id)).await {
                Err(_) => {
                    let data = AgentEvent::error("等待上游事件超时").data;
                    yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                    break;
                }
                Ok(Err(error)) => {
                    eprintln!("读取 Redis Stream {stream_key} 失败: {error}");
                    let data = json!({
                        "type": "error",
                        "message": "读取 Redis Stream 失败"
                    })
                    .to_string();
                    yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                    break;
                }
                Ok(Ok(entries)) => {
                    if !entries.is_empty() {
                        idle.reset(Instant::now());
                    }
                    for entry in entries {
                        let StreamEntry { id, event, data } = entry;
                        last_id = id.clone();
                        let terminal = AgentEvent::from_sse(&event, &data).is_terminal();
                        yield Ok::<Event, Infallible>(
                            Event::default().id(id).event(event).data(data)
                        );
                        if terminal {
                            break 'read;
                        }
                    }
                }
            }
        }
    };

    let output: TurnEventStream = Box::pin(output);
    Ok(Sse::new(output).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    ))
}
```

- [ ] **Step 5: Implement the first-message and follow-up handlers**

创建 `turn-store/src/routes/conversation.rs`：

```rust
use axum::{Json, Router, extract::State, routing::post};

use crate::{
    app::AppState,
    domain::model::CreateConversationStreamRequest,
    error::AppError,
    repositories::turn_store::create_conversation_and_turn,
    routes::stream::{TurnSse, start_turn_stream},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/conversations/stream", post(create_conversation_stream))
}

async fn create_conversation_stream(
    State(state): State<AppState>,
    Json(request): Json<CreateConversationStreamRequest>,
) -> Result<TurnSse, AppError> {
    request.validate().map_err(AppError::bad_request)?;
    let (conversation, turn) = create_conversation_and_turn(
        &state.db,
        &request.conversation,
        &request.turn,
    )
    .await?;
    start_turn_stream(state, conversation.id, turn.id, request.speed).await
}
```

将 `turn-store/src/routes/turn.rs` 替换为：

```rust
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use uuid::Uuid;

use crate::{
    app::AppState,
    domain::model::CreateTurnStreamRequest,
    error::AppError,
    repositories::turn_store::create_turn_for_conversation,
    routes::stream::{TurnSse, start_turn_stream},
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/conversations/{conversation_id}/turns/stream",
        post(create_turn_stream),
    )
}

async fn create_turn_stream(
    Path(conversation_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<CreateTurnStreamRequest>,
) -> Result<TurnSse, AppError> {
    request.validate().map_err(AppError::bad_request)?;
    let turn = create_turn_for_conversation(&state.db, conversation_id, &request.turn)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => {
                AppError::not_found("Conversation 不存在或已删除")
            }
            error => error.into(),
        })?;
    start_turn_stream(state, conversation_id, turn.id, request.speed).await
}
```

将 `turn-store/src/routes/mod.rs` 的模块与 router 装配改为：

```rust
pub mod conversation;
pub mod stream;
pub mod turn;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(conversation::router())
        .merge(turn::router())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
```

- [ ] **Step 6: Run route tests and verify GREEN**

Run:

```bash
cd turn-store
DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test --test http_stream_test
```

Expected: PASS，首次与后续请求均输出 `turn_created → text → run_completed`，数据库为 1 个 Conversation、2 个 Turn、2 条合并后的 text TurnResponse；缺失 Conversation 返回 404。

- [ ] **Step 7: Run all Rust tests before commit**

Run: `cd turn-store && DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres TEST_REDIS_URL=redis://127.0.0.1:6380/15 cargo test`

Expected: PASS with zero failed tests.

- [ ] **Step 8: Commit the backend HTTP flow**

```bash
git add turn-store/Cargo.toml turn-store/Cargo.lock turn-store/src turn-store/tests/http_stream_test.rs
git commit -m "功能：实现多轮对话流式接口"
```

### Task 5: Build and Serve the Streaming Chat Page

**Files:**
- Create: `turn-store/static/package.json`
- Create: `turn-store/static/sse.test.mjs`
- Create: `turn-store/static/sse.js`
- Create: `turn-store/static/index.html`
- Create: `turn-store/static/styles.css`
- Create: `turn-store/static/app.js`
- Create: `turn-store/src/routes/web.rs`
- Modify: `turn-store/src/routes/mod.rs`
- Modify: `turn-store/tests/http_stream_test.rs`

**Interfaces:**
- Produces: `createSseParser(onEvent)` with `push(chunk)` and `finish()`。
- Produces: `GET /`、`GET /styles.css`、`GET /sse.js`、`GET /app.js`。
- Consumes: 两条 POST SSE API；首次 `turn_created` 保存 Conversation ID。

- [ ] **Step 1: Write failing incremental-parser tests**

创建 `turn-store/static/package.json`：

```json
{
  "private": true,
  "type": "module"
}
```

创建 `turn-store/static/sse.test.mjs`：

```javascript
import assert from "node:assert/strict";
import test from "node:test";

import { createSseParser } from "./sse.js";

test("跨网络分块仍能还原命名 SSE 事件", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("id: 1-0\nevent: te");
  parser.push("xt\ndata: {\"content\":\"你");
  parser.push("好\"}\n\n");
  parser.finish();

  assert.deepEqual(events, [
    { id: "1-0", event: "text", data: '{"content":"你好"}' },
  ]);
});

test("忽略 keep-alive 并按换行拼接多行 data", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push(": keep-alive\n\nevent: status\ndata: first\ndata: second\n\n");
  parser.finish();

  assert.deepEqual(events, [
    { id: "", event: "status", data: "first\nsecond" },
  ]);
});
```

- [ ] **Step 2: Run parser tests and verify RED**

Run: `cd turn-store && node --test static/sse.test.mjs`

Expected: FAIL with module-not-found for `static/sse.js`.

- [ ] **Step 3: Implement the parser**

创建 `turn-store/static/sse.js`：

```javascript
export function createSseParser(onEvent) {
  let buffer = "";
  let eventName = "message";
  let eventId = "";
  let dataLines = [];

  function dispatch() {
    if (dataLines.length === 0) {
      eventName = "message";
      return;
    }
    onEvent({
      id: eventId,
      event: eventName,
      data: dataLines.join("\n"),
    });
    eventName = "message";
    dataLines = [];
  }

  function consumeLine(rawLine) {
    const line = rawLine.endsWith("\r") ? rawLine.slice(0, -1) : rawLine;
    if (line === "") {
      dispatch();
      return;
    }
    if (line.startsWith(":")) {
      return;
    }

    const separator = line.indexOf(":");
    const field = separator === -1 ? line : line.slice(0, separator);
    let value = separator === -1 ? "" : line.slice(separator + 1);
    if (value.startsWith(" ")) {
      value = value.slice(1);
    }

    if (field === "event") eventName = value;
    if (field === "id" && !value.includes("\0")) eventId = value;
    if (field === "data") dataLines.push(value);
  }

  return {
    push(chunk) {
      buffer += chunk;
      let newline = buffer.indexOf("\n");
      while (newline !== -1) {
        consumeLine(buffer.slice(0, newline));
        buffer = buffer.slice(newline + 1);
        newline = buffer.indexOf("\n");
      }
    },
    finish() {
      if (buffer !== "") {
        consumeLine(buffer);
        buffer = "";
      }
      dispatch();
    },
  };
}
```

- [ ] **Step 4: Run parser tests and verify GREEN**

Run: `cd turn-store && node --test static/sse.test.mjs`

Expected: 2 tests PASS.

- [ ] **Step 5: Add a failing web-route assertion**

在 `turn-store/tests/http_stream_test.rs` 增加：

```rust
#[sqlx::test]
async fn app_serves_chat_page_and_javascript(db: PgPool) {
    let app = test_app(db).await;
    let page = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let page_body = to_bytes(page.into_body(), 1024 * 1024).await.unwrap();
    assert!(String::from_utf8_lossy(&page_body).contains("Turn Store Agent"));

    let script = app
        .oneshot(Request::builder().uri("/app.js").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(script.status(), StatusCode::OK);
    assert_eq!(
        script.headers()[CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
}
```

Run: `cd turn-store && DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres TEST_REDIS_URL=redis://127.0.0.1:6380/15 cargo test --test http_stream_test app_serves_chat_page`

Expected: FAIL because `/` and `/app.js` are not registered.

- [ ] **Step 6: Create the chat HTML**

创建 `turn-store/static/index.html`：

```html
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Turn Store Agent</title>
    <link rel="stylesheet" href="/styles.css" />
  </head>
  <body>
    <main class="shell">
      <header class="topbar">
        <div>
          <p class="eyebrow">AXUM · REDIS STREAM · SSE</p>
          <h1>Turn Store Agent</h1>
        </div>
        <button id="new-chat" class="secondary" type="button">新对话</button>
      </header>

      <section id="messages" class="messages" aria-live="polite">
        <div id="empty-state" class="empty-state">
          <span class="orb">✦</span>
          <h2>开始一段流式对话</h2>
          <p>首条消息创建 Conversation，后续消息复用同一会话。</p>
        </div>
      </section>

      <section class="composer-wrap">
        <div id="run-panel" class="run-panel" hidden>
          <button id="run-toggle" type="button" aria-expanded="false">运行详情</button>
          <pre id="run-log"></pre>
        </div>
        <form id="composer" class="composer">
          <textarea
            id="prompt"
            rows="3"
            placeholder="输入消息，Enter 发送，Shift + Enter 换行"
            required
          ></textarea>
          <div class="composer-actions">
            <label>
              输出模式
              <select id="speed">
                <option value="fast">快速 · 按行</option>
                <option value="slow">慢速 · 5-10 字</option>
              </select>
            </label>
            <span id="conversation-label">尚未创建会话</span>
            <button id="stop" class="secondary" type="button" hidden>停止接收</button>
            <button id="send" type="submit">发送</button>
          </div>
        </form>
      </section>
    </main>
    <script type="module" src="/app.js"></script>
  </body>
</html>
```

- [ ] **Step 7: Create the responsive CSS**

创建 `turn-store/static/styles.css`：

```css
:root {
  color-scheme: dark;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #07110f;
  color: #eaf7f2;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  min-height: 100vh;
  background:
    radial-gradient(circle at 15% 0%, rgba(55, 214, 157, 0.16), transparent 30rem),
    radial-gradient(circle at 95% 30%, rgba(66, 153, 225, 0.12), transparent 28rem),
    #07110f;
}
button, textarea, select { font: inherit; }
button {
  border: 0;
  border-radius: 999px;
  padding: 0.72rem 1.15rem;
  background: #54e0ae;
  color: #052017;
  font-weight: 750;
  cursor: pointer;
}
button:disabled { cursor: not-allowed; opacity: 0.45; }
button.secondary { background: #172925; color: #d7eee6; border: 1px solid #2d4941; }
.shell { width: min(1040px, 100%); min-height: 100vh; margin: 0 auto; display: grid; grid-template-rows: auto 1fr auto; }
.topbar { display: flex; justify-content: space-between; align-items: center; gap: 1rem; padding: 1.4rem clamp(1rem, 4vw, 2.5rem); border-bottom: 1px solid rgba(126, 181, 162, 0.16); }
.topbar h1 { margin: 0.15rem 0 0; font-size: clamp(1.35rem, 3vw, 2rem); }
.eyebrow { margin: 0; color: #73c9a9; font-size: 0.72rem; font-weight: 800; letter-spacing: 0.15em; }
.messages { min-height: 0; overflow-y: auto; padding: 2rem clamp(1rem, 4vw, 2.5rem) 10rem; }
.empty-state { min-height: 48vh; display: grid; place-content: center; justify-items: center; text-align: center; color: #99b8ad; }
.empty-state h2 { color: #eaf7f2; margin-bottom: 0.25rem; }
.orb { display: grid; place-items: center; width: 3.3rem; aspect-ratio: 1; border-radius: 50%; background: #173c31; color: #6ce4b8; box-shadow: 0 0 3rem rgba(84, 224, 174, 0.2); }
.message { display: grid; gap: 0.35rem; margin: 1.2rem 0; }
.message.user { justify-items: end; }
.message .role { color: #7fa397; font-size: 0.78rem; font-weight: 700; }
.bubble { max-width: min(46rem, 88%); margin: 0; padding: 0.9rem 1.05rem; border-radius: 1.15rem; white-space: pre-wrap; overflow-wrap: anywhere; line-height: 1.65; background: #12231f; border: 1px solid #213a33; }
.user .bubble { background: #215c49; border-color: #2a725b; }
.message.error .bubble { border-color: #8e4d50; color: #ffc9c9; }
.cursor::after { content: ""; display: inline-block; width: 0.55rem; height: 1rem; margin-left: 0.3rem; vertical-align: -0.12rem; background: #65e6b6; animation: pulse 0.9s infinite; }
@keyframes pulse { 50% { opacity: 0.2; } }
.composer-wrap { position: sticky; bottom: 0; padding: 0.8rem clamp(1rem, 4vw, 2.5rem) 1.5rem; background: linear-gradient(transparent, #07110f 22%); }
.composer { padding: 0.85rem; border-radius: 1.2rem; background: rgba(14, 31, 26, 0.96); border: 1px solid #29473e; box-shadow: 0 1.4rem 4rem rgba(0, 0, 0, 0.36); }
.composer textarea { width: 100%; resize: none; border: 0; outline: 0; padding: 0.55rem; background: transparent; color: inherit; line-height: 1.5; }
.composer-actions { display: flex; align-items: center; gap: 0.7rem; flex-wrap: wrap; color: #8caaa0; font-size: 0.8rem; }
.composer-actions label { display: flex; align-items: center; gap: 0.45rem; }
.composer-actions select { border: 1px solid #2d4941; border-radius: 999px; padding: 0.45rem 0.7rem; background: #10231e; color: #dff4ec; }
#conversation-label { margin-right: auto; }
.run-panel { margin-bottom: 0.55rem; border: 1px solid #29473e; border-radius: 0.9rem; background: rgba(12, 27, 23, 0.96); }
.run-panel button { width: 100%; text-align: left; border-radius: 0.9rem; background: transparent; color: #9bc1b3; }
.run-panel pre { display: none; max-height: 10rem; margin: 0; padding: 0 1rem 0.9rem; overflow: auto; color: #7fa397; white-space: pre-wrap; font-size: 0.72rem; }
.run-panel.open pre { display: block; }
@media (max-width: 640px) {
  .topbar { align-items: flex-start; }
  .messages { padding-bottom: 12rem; }
  .bubble { max-width: 94%; }
  #conversation-label { width: 100%; order: 3; }
}
```

- [ ] **Step 8: Implement browser state and streaming rendering**

创建 `turn-store/static/app.js`：

```javascript
import { createSseParser } from "./sse.js";

const elements = {
  form: document.querySelector("#composer"),
  prompt: document.querySelector("#prompt"),
  speed: document.querySelector("#speed"),
  send: document.querySelector("#send"),
  stop: document.querySelector("#stop"),
  newChat: document.querySelector("#new-chat"),
  messages: document.querySelector("#messages"),
  empty: document.querySelector("#empty-state"),
  conversationLabel: document.querySelector("#conversation-label"),
  runPanel: document.querySelector("#run-panel"),
  runToggle: document.querySelector("#run-toggle"),
  runLog: document.querySelector("#run-log"),
};

const state = {
  conversationId: null,
  controller: null,
  streaming: false,
  draftDocId: crypto.randomUUID(),
};

function scrollToBottom() {
  elements.messages.scrollTop = elements.messages.scrollHeight;
}

function addMessage(role, content = "") {
  elements.empty.hidden = true;
  const article = document.createElement("article");
  article.className = `message ${role}`;
  const label = document.createElement("span");
  label.className = "role";
  label.textContent = role === "user" ? "你" : "Agent";
  const bubble = document.createElement("p");
  bubble.className = "bubble";
  bubble.textContent = content;
  article.append(label, bubble);
  elements.messages.append(article);
  scrollToBottom();
  return { article, bubble };
}

function setStreaming(active) {
  state.streaming = active;
  elements.send.disabled = active;
  elements.prompt.disabled = active;
  elements.speed.disabled = active;
  elements.stop.hidden = !active;
  elements.newChat.disabled = active;
}

function logEvent(event, data) {
  elements.runPanel.hidden = false;
  const line = `[${event}] ${typeof data === "string" ? data : JSON.stringify(data)}`;
  elements.runLog.textContent += `${line}\n`;
  elements.runLog.scrollTop = elements.runLog.scrollHeight;
}

function requestFor(prompt) {
  const turn = {
    input_context: prompt,
    document_content_version_id: 1,
  };
  if (state.conversationId) {
    return {
      url: `/api/conversations/${state.conversationId}/turns/stream`,
      body: { turn, speed: elements.speed.value },
    };
  }
  return {
    url: "/api/conversations/stream",
    body: {
      conversation: {
        doc_id: `web-${state.draftDocId}`,
        doc_type: "markdown",
        user_id: 1,
        title: prompt.slice(0, 40),
        type: "CHAT_EDIT",
        inline_type: null,
      },
      turn,
      speed: elements.speed.value,
    },
  };
}

async function consumeSse(response, onEvent) {
  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: `HTTP ${response.status}` }));
    throw new Error(error.error || `HTTP ${response.status}`);
  }
  if (!response.body) throw new Error("浏览器不支持流式响应");

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  const parser = createSseParser(onEvent);
  while (true) {
    const { value, done } = await reader.read();
    if (done) break;
    parser.push(decoder.decode(value, { stream: true }));
  }
  parser.push(decoder.decode());
  parser.finish();
}

async function send(prompt) {
  const assistant = addMessage("assistant");
  assistant.bubble.classList.add("cursor");
  elements.runLog.textContent = "";
  elements.runPanel.hidden = true;
  const request = requestFor(prompt);
  state.controller = new AbortController();
  setStreaming(true);

  try {
    const response = await fetch(request.url, {
      method: "POST",
      headers: { Accept: "text/event-stream", "Content-Type": "application/json" },
      body: JSON.stringify(request.body),
      signal: state.controller.signal,
    });
    let terminal = false;
    await consumeSse(response, ({ event, data }) => {
      let payload;
      try {
        payload = JSON.parse(data);
      } catch {
        payload = data;
      }
      logEvent(event, payload);

      if (event === "turn_created") {
        state.conversationId = payload.conversation_id;
        elements.conversationLabel.textContent = `Conversation ${state.conversationId.slice(0, 8)}`;
      } else if (event === "text" && typeof payload.content === "string") {
        assistant.bubble.textContent += payload.content;
        scrollToBottom();
      } else if (event === "run_completed") {
        terminal = true;
      } else if (event === "error") {
        terminal = true;
        throw new Error(payload.message || "流式响应失败");
      }
    });
    if (!terminal) throw new Error("SSE 在终止事件前结束");
  } catch (error) {
    if (error.name === "AbortError") {
      assistant.bubble.textContent ||= "已停止接收；服务端仍会完成本轮处理。";
    } else {
      assistant.article.classList.add("error");
      assistant.bubble.textContent ||= error.message;
      logEvent("error", error.message);
    }
  } finally {
    assistant.bubble.classList.remove("cursor");
    state.controller = null;
    setStreaming(false);
    elements.prompt.focus();
  }
}

elements.form.addEventListener("submit", (event) => {
  event.preventDefault();
  const prompt = elements.prompt.value.trim();
  if (!prompt || state.streaming) return;
  addMessage("user", prompt);
  elements.prompt.value = "";
  void send(prompt);
});

elements.prompt.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    elements.form.requestSubmit();
  }
});

elements.stop.addEventListener("click", () => state.controller?.abort());
elements.newChat.addEventListener("click", () => {
  state.conversationId = null;
  state.draftDocId = crypto.randomUUID();
  elements.messages.replaceChildren(elements.empty);
  elements.empty.hidden = false;
  elements.runPanel.hidden = true;
  elements.runLog.textContent = "";
  elements.conversationLabel.textContent = "尚未创建会话";
  elements.prompt.focus();
});
elements.runToggle.addEventListener("click", () => {
  const open = elements.runPanel.classList.toggle("open");
  elements.runToggle.setAttribute("aria-expanded", String(open));
});
```

- [ ] **Step 9: Serve the files from Axum**

创建 `turn-store/src/routes/web.rs`：

```rust
use axum::{
    Router,
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse, Response},
    routing::get,
};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/sse.js", get(sse_script))
        .route("/app.js", get(app_script))
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn styles() -> Response {
    asset("text/css; charset=utf-8", include_str!("../../static/styles.css"))
}

async fn sse_script() -> Response {
    asset("text/javascript; charset=utf-8", include_str!("../../static/sse.js"))
}

async fn app_script() -> Response {
    asset("text/javascript; charset=utf-8", include_str!("../../static/app.js"))
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(CONTENT_TYPE, content_type)], body).into_response()
}
```

在 `turn-store/src/routes/mod.rs` 增加 `pub mod web;`，并在 router builder 末尾增加 `.merge(web::router())`。

- [ ] **Step 10: Run JS and Rust web tests**

Run:

```bash
cd turn-store
node --test static/sse.test.mjs
DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test --test http_stream_test
```

Expected: Node 2/2 PASS；Rust HTTP tests all PASS including `/` and `/app.js` content type.

- [ ] **Step 11: Commit the chat page**

```bash
git add turn-store/static turn-store/src/routes turn-store/tests/http_stream_test.rs
git commit -m "功能：增加流式聊天网页"
```

### Task 6: Add Reproducible Runtime Documentation and Verify the Real Mock

**Files:**
- Create: `turn-store/tests/runtime_docs_test.rs`
- Create: `turn-store/.env.example`
- Create: `turn-store/README.md`

**Interfaces:**
- Produces: 可复制的本地配置与启动顺序。
- Verifies: mock TypeScript、Rust、Node parser、真实 mock SSE 和浏览器页面。

- [ ] **Step 1: Write a failing documentation contract test**

创建 `turn-store/tests/runtime_docs_test.rs`：

```rust
#[test]
fn runtime_docs_cover_required_services_and_environment() {
    let env = include_str!("../.env.example");
    for name in [
        "DATABASE_URL",
        "REDIS_URL",
        "UPSTREAM_AGENT_URL",
        "RELAY_IDLE_TIMEOUT_SECONDS",
    ] {
        assert!(env.contains(name), "缺少环境变量 {name}");
    }

    let readme = include_str!("../README.md");
    for command in [
        "docker run",
        "npm run dev",
        "cargo run",
        "node --test static/sse.test.mjs",
    ] {
        assert!(readme.contains(command), "README 缺少命令 {command}");
    }
}
```

- [ ] **Step 2: Run the test and verify RED**

Run: `cd turn-store && cargo test --test runtime_docs_test`

Expected: FAIL because `.env.example` and `README.md` do not exist.

- [ ] **Step 3: Add the environment example**

创建 `turn-store/.env.example`：

```dotenv
BIND_ADDR=127.0.0.1:3000
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55432/turn_store
DATABASE_MAX_CONNECTIONS=10
REDIS_URL=redis://127.0.0.1:6379/
REDIS_STREAM_TTL_SECONDS=3600
REDIS_XREAD_BLOCK_MS=15000
RELAY_IDLE_TIMEOUT_SECONDS=60
UPSTREAM_AGENT_URL=http://127.0.0.1:8787/events
```

- [ ] **Step 4: Add exact local-run documentation**

创建 `turn-store/README.md`：

````markdown
# Turn Store

Axum 中转服务：创建或复用 Conversation，为每个 Turn 建立 Redis Stream，连接 mock agent SSE，并把事件持久化到 PostgreSQL 后转发给网页。

## 本地启动

在仓库根目录执行：

```bash
docker run --rm -d --name turn-store-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=turn_store \
  -p 55432:5432 postgres:17-alpine

docker run --rm -d --name turn-store-redis \
  -p 6379:6379 redis:7-alpine
```

启动 mock agent：

```bash
cd turn-store/mock
npm install
npm run dev
```

另开终端启动 Axum：

```bash
cd turn-store
cp .env.example .env.local
set -a
source .env.local
set +a
cargo run
```

浏览器打开 <http://127.0.0.1:3000/>。首次发送创建 Conversation，后续发送复用页面保存的 `conversation_id`；“新对话”会清空该 ID。

## 检查

```bash
cd turn-store/mock && npm run check
cd turn-store && node --test static/sse.test.mjs
cd turn-store && cargo fmt --check
cd turn-store && cargo test --test relay_policy_test
```

需要 PostgreSQL 和测试 Redis 的完整检查：

```bash
docker run --rm -d --name turn-store-test-redis -p 6380:6379 redis:7-alpine
cd turn-store
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55432/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test
```

停止本地依赖：

```bash
docker stop turn-store-test-redis turn-store-redis turn-store-postgres
```
````

- [ ] **Step 5: Run the documentation test and verify GREEN**

Run: `cd turn-store && cargo test --test runtime_docs_test`

Expected: PASS.

- [ ] **Step 6: Run all static and compilation checks**

Run:

```bash
cd turn-store/mock && npm run check
cd turn-store && node --test static/sse.test.mjs
cd turn-store && cargo fmt --check
cd turn-store && cargo clippy --all-targets --all-features -- -D warnings
cd turn-store && cargo build
```

Expected: every command exits 0 with no warnings promoted to errors.

- [ ] **Step 7: Run the complete Rust test suite with real PostgreSQL and Redis**

Run:

```bash
cd turn-store
DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test
```

Expected: zero failed tests.

- [ ] **Step 8: Exercise the real mock service through Axum**

启动 mock 与 Axum 后执行：

```bash
curl -fsS http://127.0.0.1:8787/health
curl -fsS http://127.0.0.1:3000/health
curl -N -X POST http://127.0.0.1:3000/api/conversations/stream \
  -H 'Accept: text/event-stream' \
  -H 'Content-Type: application/json' \
  --data '{"conversation":{"doc_id":"e2e-doc","doc_type":"markdown","user_id":1,"title":"端到端验证","type":"CHAT_EDIT","inline_type":null},"turn":{"input_context":"验证真实 mock","document_content_version_id":1},"speed":"fast"}'
```

Expected: health 均返回 `{"ok":true}`；SSE 首事件为 `turn_created`，包含若干 `text`，末事件为 `run_completed`。

- [ ] **Step 9: Verify the browser interaction**

使用 `browser:control-in-app-browser` 打开 `http://127.0.0.1:3000/` 并逐项验证：

1. 首次发送后出现用户消息、增量助手正文和 Conversation 短 ID。
2. 第二次发送沿用同一短 ID并产生新的 Turn 流。
3. “停止接收”终止浏览器 fetch，页面保留已显示内容。
4. “新对话”清空 ID 和消息；下一次发送获得不同 Conversation ID。
5. slow 模式持续输出且最终正常结束。

- [ ] **Step 10: Commit runtime documentation**

```bash
git add turn-store/.env.example turn-store/README.md turn-store/tests/runtime_docs_test.rs
git commit -m "文档：补充 turn-store 本地运行与验证说明"
```

- [ ] **Step 11: Stop temporary services**

Run: `docker stop turn-store-test-redis turn-store-redis turn-store-postgres`

Expected: containers started by this plan are stopped; no source files change.

## Final Verification Checklist

- [ ] `git diff --check` reports no whitespace errors.
- [ ] `cd turn-store/mock && npm run check` exits 0.
- [ ] `cd turn-store && node --test static/sse.test.mjs` exits 0 with 2 passing tests.
- [ ] `cd turn-store && cargo fmt --check` exits 0.
- [ ] `cd turn-store && cargo clippy --all-targets --all-features -- -D warnings` exits 0.
- [ ] `cd turn-store && DATABASE_URL=postgres://post:post@127.0.0.1:5433/postgres TEST_REDIS_URL=redis://127.0.0.1:6380/15 cargo test` exits 0.
- [ ] Real mock curl starts with `turn_created` and ends with `run_completed`.
- [ ] Database assertions prove one reused Conversation, multiple Turns, and one appendable text TurnResponse per Turn.
- [ ] Browser verification covers first turn, follow-up, stop, new conversation, fast, and slow.
- [ ] `git status --short` is reviewed so unrelated pre-existing changes remain untouched.
