import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const root = new URL("./", import.meta.url);
const read = (name) => readFile(new URL(name, root), "utf8");

test("页面提供双栏对话区和语义化运行检查器", async () => {
  const html = await read("index.html");

  assert.match(html, /class="workspace"/);
  assert.match(html, /<aside[^>]+id="run-panel"/);
  assert.match(html, /id="conversation-status"/);
  assert.match(html, /id="status-dot"/);
  assert.match(html, /id="run-log"[^>]+role="list"/);
});

test("页面保留现有交互元素 ID", async () => {
  const html = await read("index.html");
  for (const id of [
    "composer",
    "prompt",
    "speed",
    "send",
    "stop",
    "new-chat",
    "messages",
    "empty-state",
    "conversation-label",
    "run-panel",
    "run-toggle",
    "run-log",
  ]) {
    assert.match(html, new RegExp(`id="${id}"`));
  }
});

test("运行事件使用结构化列表项并同步会话状态", async () => {
  const app = await read("app.js");

  assert.match(app, /createElement\("li"\)/);
  assert.match(app, /className = "run-event"/);
  assert.match(app, /runLog\.append\(item\)/);
  assert.match(app, /conversationStatus\.textContent/);
  assert.match(app, /statusDot\.classList\.toggle\("active"/);
});

test("样式使用确认的色板且不包含渐变或投影", async () => {
  const css = await read("styles.css");

  for (const color of ["#e6e6e6", "#c8e6cd", "#c5b0f4", "#f4ecd6"]) {
    assert.match(css, new RegExp(color, "i"));
  }
  assert.doesNotMatch(css, /gradient\(/i);
  assert.doesNotMatch(css, /box-shadow\s*:/i);
});

test("样式覆盖 960px 和 640px 响应式断点", async () => {
  const css = await read("styles.css");
  assert.match(css, /@media \(max-width: 959px\)/);
  assert.match(css, /@media \(max-width: 639px\)/);
});
