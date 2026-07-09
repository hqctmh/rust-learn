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
