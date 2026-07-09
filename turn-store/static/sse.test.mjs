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
