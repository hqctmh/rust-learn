# Turn Store Axum 中转与聊天网页设计

## 目标

完善 `turn-store`，提供一个可本地运行的 Axum 服务。网页首次发送消息时创建 `conversation` 和首个 `turn`；后续消息复用同一 `conversation_id`，只创建新的 `turn`。每个 Turn 启动独立 Redis Stream，后台连接现有 mock agent SSE，将事件持久化为 `turn_response`，同时通过 Redis Stream 转发给网页。

本次同时交付一个由 Axum 同源托管的原生聊天网页，用于发起新对话、继续多轮对话并展示流式输出。

## 非目标

- 不实现用户登录、权限控制或多租户隔离。
- 不实现跨页面刷新的会话恢复、历史列表或断线续传。
- 不引入 React、Vue、Vite 或 Markdown 渲染器。
- 不改变 mock agent 的正文来源以及 fast/slow 分块语义。
- 不实现真正的上游任务取消；浏览器停止接收后，服务端继续完成持久化。

## 已选方案

采用两条明确的 POST SSE 接口：首次消息使用“创建 Conversation 并开始流”，后续消息使用“在已有 Conversation 下开始新 Turn”。这比带可选 `conversation_id` 的单接口含义清晰，也避免“先创建空 Conversation、再发送首条消息”造成两次请求和孤立数据。

每个 Turn 使用一个 Redis Stream，key 为 `turn:{turn_id}:events`。Redis 是运行期事件总线，PostgreSQL 是持久化事实源。网页使用 `fetch` 和 `ReadableStream` 解析 POST SSE，而不是仅支持 GET 的 `EventSource`。

## API 契约

### 首次消息

`POST /api/conversations/stream`

请求：

```json
{
  "conversation": {
    "doc_id": "demo-doc",
    "doc_type": "markdown",
    "user_id": 1,
    "title": "新对话",
    "type": "CHAT_EDIT",
    "inline_type": null
  },
  "turn": {
    "input_context": "请介绍这个设计",
    "document_content_version_id": 1
  },
  "speed": "fast"
}
```

`speed` 可为 `fast` 或 `slow`，缺省为 `fast`。Conversation 与 Turn 必须在同一 PostgreSQL 事务中创建。

### 后续消息

`POST /api/conversations/{conversation_id}/turns/stream`

请求：

```json
{
  "turn": {
    "input_context": "继续说明数据流",
    "document_content_version_id": 1
  },
  "speed": "fast"
}
```

服务端只允许复用 `deleted_at = 0` 的 Conversation。不存在或已删除时返回 `404`，不得创建 Turn。

### SSE 响应

两条接口使用完全相同的事件协议。第一条事件始终是服务端生成的 `turn_created`：

```text
id: 1710000000000-0
event: turn_created
data: {"type":"turn_created","conversation_id":"...","turn_id":"..."}

```

后续事件保留 mock agent 的事件名和 JSON data，并使用 Redis Stream entry ID 作为 SSE `id`。正常流以 `run_completed` 结束，失败流以 `error` 结束；二者是唯一终止事件。

建流前错误返回 JSON：请求校验失败为 `400`，Conversation 不存在为 `404`，PostgreSQL 或 Redis 初始化失败为 `500`。建流后的错误通过命名 SSE `error` 事件返回。

## 主数据流

1. 路由校验请求，首次消息在事务内创建 Conversation 和 Turn；后续消息校验 Conversation 后创建 Turn。
2. 根据 `turn_id` 生成 Redis Stream key。
3. 通过第一次 `XADD` 写入 `turn_created`，从而创建 Stream；每次写入都刷新 Stream TTL。
4. 为前端 SSE 建立独立的阻塞读连接，从 `0-0` 开始读取该 Stream。
5. 后台 Tokio task 根据 speed 连接 mock agent，解析上游 SSE。
6. 每个上游事件先持久化到 PostgreSQL，再 `XADD` 到 Redis Stream。网页能看到的上游事件因此已经落库。
7. Axum 将 Redis entry 映射为 SSE Event 并发送给网页。
8. 收到 `run_completed` 或 `error` 后结束读取并关闭 HTTP 响应。

浏览器断开或点击停止只会终止本地 fetch；后台 relay 继续消费上游并完成 PostgreSQL 持久化。Redis Stream 到期后自动删除。

## TurnResponse 持久化规则

追加策略由领域层集中决定，不散落在路由、仓储或前端中。第一版只有合法的 `text` 事件可追加；未来新增可追加类型时，只扩展该策略。

- 可追加事件：同一 `(turn_id, type)` 只有一条 `appendable = true` 的 `turn_response`。使用 PostgreSQL 部分唯一索引和 `INSERT ... ON CONFLICT ... DO UPDATE` 原子拼接 `response`。
- 非追加事件：每个事件各写一条 `appendable = false` 的记录，`response` 保存原始 JSON data。
- 格式错误的 `text`：不丢弃，按非追加事件保存原始 data。
- `turn_created`：这是 relay 自己生成的传输元数据，只进入 Redis Stream，不写 `turn_response`。
- 已生成的部分文本：发生错误时保留，不回滚。

## 代码边界

### 后端

- `src/main.rs`：加载可选 `.env`、配置、PostgreSQL、Redis、HTTP client，执行迁移并启动服务器。
- `src/app.rs`：定义 `AppState`，装配 API 与网页路由。
- `src/domain/model.rs`：Conversation、Turn、TurnResponse 以及首次/后续请求模型和校验。
- `src/domain/event.rs`：上游事件归一化、终止判断和追加策略。
- `src/repositories/turn_store.rs`：事务创建 Conversation + Turn、为已有 Conversation 创建 Turn、持久化事件。
- `src/infra/redis_stream.rs`：Redis Stream 写入、独立阻塞读取和 TTL。
- `src/infra/upstream.rs`：连接 mock agent SSE。
- `src/services/relay.rs`：消费上游、按“先 PostgreSQL 后 Redis”顺序处理事件，并发布终止错误。
- `src/routes/conversation.rs`：首次消息接口。
- `src/routes/turn.rs`：复用 Conversation 的后续消息接口。
- `src/routes/web.rs`：同源返回网页静态资源。

路由层共享一个内部的“创建 Stream 并启动 relay”函数，避免两条接口复制 SSE 读取循环，但首次和后续请求模型保持独立。

### 网页

- `static/index.html`：聊天布局、新对话按钮、速度选择、消息列表、输入区和运行详情。
- `static/styles.css`：响应式双栏/单栏布局、消息气泡、流式状态和错误态。
- `static/sse.js`：无 DOM 依赖的增量 SSE 分帧解析器。
- `static/app.js`：Conversation 状态、请求选择、事件 reducer、渲染、自动滚动和 AbortController。
- `static/sse.test.mjs`：使用 Node 内置测试框架验证跨网络 chunk 的 SSE 分帧。

网页把 `conversation_id` 只保存在当前页面内存中。收到首次 `turn_created` 后保存该 ID；后续发送调用复用接口。点击“新对话”时清空 ID 和消息区，下一次发送重新创建 Conversation。用户正在接收一个 Turn 时禁用再次发送，避免同一页面并行产生两个助手消息。

只有 `text.data.content` 追加到助手消息正文；其他事件进入可折叠运行详情。正文使用 `textContent` 渲染。收到 `run_completed` 或 `error` 时退出 loading；错误状态保留已经显示的部分正文。

## 错误与超时

- 上游连接失败、SSE 解析失败或在终止事件前断开：记录服务端详细日志，持久化一条非追加 `error` TurnResponse，并尽可能向 Redis Stream 发布脱敏后的 `error` 事件。
- PostgreSQL 事件持久化失败：不发布对应的正常事件，转入上述错误流程。
- Redis 写入失败：保留已经持久化的数据并记录错误；若无法发布终止事件，前端读循环通过空闲超时自行结束。
- Redis 读取失败：路由向前端发送 `error` 并关闭响应。
- 空闲超时：从最后一条业务事件开始计算，默认 60 秒，可由 `RELAY_IDLE_TIMEOUT_SECONDS` 配置；超时后发送 `error` 并结束响应。
- SSE keep-alive：默认每 10 秒发送注释帧，不计为业务活动。

## 配置与运行说明

保留环境变量配置，并新增不含凭据的 `.env.example` 与 `turn-store/README.md`。服务启动时可选加载当前目录 `.env`，但不覆盖已经导出的环境变量，也不改写用户现有 `.env`。

必需配置为 `DATABASE_URL`；其余提供本地默认值：

- `BIND_ADDR=127.0.0.1:3000`
- `REDIS_URL=redis://127.0.0.1/`
- `UPSTREAM_AGENT_URL=http://127.0.0.1:8787/events`
- `DATABASE_MAX_CONNECTIONS=10`
- `REDIS_STREAM_TTL_SECONDS=3600`
- `REDIS_XREAD_BLOCK_MS=15000`
- `RELAY_IDLE_TIMEOUT_SECONDS=60`

README 按 PostgreSQL、Redis、mock、Axum 的顺序给出启动命令，以及浏览器地址 `http://127.0.0.1:3000/`。

## 测试策略

实施遵循测试先行，每个行为先看到测试因缺少功能而失败，再写最小实现使其通过。

1. 领域测试：请求校验、事件名归一化、合法/异常 text 追加策略、终止事件和 Stream key。
2. PostgreSQL 集成测试：首次事务创建、复用 Conversation、已删除/不存在拒绝、同一 Turn 多个 text chunk 最终只有一条 appendable TurnResponse，以及非追加事件逐条保存。
3. Redis 集成测试：`XADD/XREAD` 顺序、生产者与阻塞消费者互不阻塞、Stream TTL。
4. HTTP 端到端测试：启动 Redis、mock 和 Axum，验证首个 `turn_created`、正文增量、`run_completed`、一个 Conversation 下多个 Turn，以及最终数据库内容。
5. 网页测试：Node 测试覆盖分帧残片、命名事件、多行 data 和 keep-alive；浏览器检查首次对话、继续对话、停止、新对话、自动滚动与错误显示。

## 验收条件

- 首次发送后数据库新增一个 Conversation 和一个 Turn。
- 连续发送两条后续消息后，Conversation 总数仍为一个，三个 Turn 都引用同一 `conversation_id`。
- 每个 Turn 的所有合法 text chunk 精确拼接到唯一一条 appendable text TurnResponse。
- 非追加事件保持逐条原始 JSON，不因 text 合并而丢失。
- 两条接口的首事件均为 `turn_created`，正常末事件均为 `run_completed`。
- mock 的 fast 仍按行输出，slow 仍按 5 至 10 个字符输出。
- 网页能展示连续流式正文，能停止本地接收并能开始全新的 Conversation。
- Rust 测试、mock TypeScript 类型检查、网页 SSE parser 测试、Rust 格式检查和构建全部通过。
