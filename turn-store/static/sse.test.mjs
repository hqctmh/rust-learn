import assert from "node:assert/strict";
import test from "node:test";

import * as sse from "./sse.js";

const { createSseParser } = sse;

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

test("支持跨分块的 CRLF 行结束符", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("id: 2-0\r");
  parser.push("\nevent: text\r\ndata: hello\r\n\r");
  parser.push("\n");
  parser.finish();

  assert.deepEqual(events, [
    { id: "2-0", event: "text", data: "hello" },
  ]);
});

test("支持纯 CR 行结束符", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("event: text\rdata: value\r\r");
  parser.finish();

  assert.deepEqual(events, [
    { id: "", event: "text", data: "value" },
  ]);
});

test("finish 派发没有末尾空行的事件", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("event: status\ndata: tail");
  parser.finish();

  assert.deepEqual(events, [
    { id: "", event: "status", data: "tail" },
  ]);
});

test("事件 ID 会继承、空 ID 会清空且含 NUL 的 ID 会被忽略", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("id: first\ndata: one\n\ndata: two\n\n");
  parser.push("id:\ndata: three\n\nid: ignored\0id\ndata: four\n\n");
  parser.finish();

  assert.deepEqual(events, [
    { id: "first", event: "message", data: "one" },
    { id: "first", event: "message", data: "two" },
    { id: "", event: "message", data: "three" },
    { id: "", event: "message", data: "four" },
  ]);
});

test("空 event 字段回退为 message", () => {
  const events = [];
  const parser = createSseParser((event) => events.push(event));

  parser.push("event:\ndata: value\n\n");
  parser.finish();

  assert.deepEqual(events, [
    { id: "", event: "message", data: "value" },
  ]);
});

test("拒绝非法 JSON 和非对象事件 data", () => {
  assert.throws(
    () => sse.parseAgentEventData("text", "{"),
    /text 事件 data 不是有效 JSON/,
  );
  assert.throws(
    () => sse.parseAgentEventData("text", "null"),
    /text 事件 data 必须是 JSON 对象/,
  );
  assert.throws(
    () => sse.parseAgentEventData("text", "[]"),
    /text 事件 data 必须是 JSON 对象/,
  );
});

test("校验 turn_created 的 conversation_id", () => {
  assert.throws(
    () => sse.parseAgentEventData("turn_created", "{}"),
    /turn_created\.conversation_id 必须是字符串/,
  );
  assert.throws(
    () => sse.parseAgentEventData("turn_created", '{"conversation_id":1}'),
    /turn_created\.conversation_id 必须是字符串/,
  );
});

test("校验 text 的 content", () => {
  assert.throws(
    () => sse.parseAgentEventData("text", "{}"),
    /text\.content 必须是字符串/,
  );
  assert.throws(
    () => sse.parseAgentEventData("text", '{"content":null}'),
    /text\.content 必须是字符串/,
  );
});

test("校验 error 的 message", () => {
  assert.throws(
    () => sse.parseAgentEventData("error", "{}"),
    /error\.message 必须是字符串/,
  );
  assert.throws(
    () => sse.parseAgentEventData("error", '{"message":false}'),
    /error\.message 必须是字符串/,
  );
});

test("返回通过校验的事件 payload", () => {
  assert.deepEqual(
    sse.parseAgentEventData(
      "turn_created",
      '{"conversation_id":"conversation-1","turn_id":"turn-1"}',
    ),
    { conversation_id: "conversation-1", turn_id: "turn-1" },
  );
  assert.deepEqual(sse.parseAgentEventData("text", '{"content":"ok"}'), {
    content: "ok",
  });
  assert.deepEqual(sse.parseAgentEventData("error", '{"message":"bad"}'), {
    message: "bad",
  });
});

test("非关键事件名不会命中对象原型字段", () => {
  assert.deepEqual(
    sse.parseAgentEventData("constructor", '{"type":"constructor"}'),
    { type: "constructor" },
  );
});
