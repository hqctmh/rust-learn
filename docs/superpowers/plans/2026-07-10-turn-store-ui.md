# Turn Store UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 Turn Store 流式对话页实现为已确认的双栏编辑器界面，并保持现有 SSE、会话复用和停止接收行为不变。

**Architecture:** 保留无构建步骤的原生 HTML/CSS/JavaScript 结构。`index.html` 只负责语义化页面骨架，`styles.css` 实现 Figma 风格视觉与三档响应式布局，`app.js` 继续管理会话与流式状态，并把纯文本运行日志升级为结构化事件时间线。

**Tech Stack:** HTML5、CSS、原生 JavaScript ES Modules、Node.js 内置测试运行器、Axum 静态文件服务。

## Global Constraints

- 只修改 `turn-store/static/index.html`、`turn-store/static/styles.css`、必要的 `turn-store/static/app.js`，并新增 `turn-store/static/ui.test.mjs`。
- 不修改 Rust 路由、数据库、Redis、mock agent 或 API 契约。
- 不新增前端框架、图标包、网络字体、npm 依赖或构建步骤。
- 使用纯白画布、黑色文字、`#e6e6e6` 分隔线、`#c8e6cd` 检查器、`#c5b0f4` 用户消息和 `#f4ecd6` Agent 消息；不使用渐变和投影。
- 保留 `aria-live`、表单标签、Enter 发送、Shift+Enter 换行、`aria-expanded` 和禁用状态。
- 不覆盖或提交用户已有的 `AGENTS.md`、`sqlx-demo/Cargo.toml`、`turn-store/.env.local` 改动。

---

### Task 1: 建立页面语义结构

**Files:**
- Create: `turn-store/static/ui.test.mjs`
- Modify: `turn-store/static/index.html:9-53`

**Interfaces:**
- Consumes: 现有 `app.js` 使用的元素 ID：`composer`、`prompt`、`speed`、`send`、`stop`、`new-chat`、`messages`、`empty-state`、`conversation-label`、`run-panel`、`run-toggle`、`run-log`。
- Produces: 新增 `conversation-status`、`status-dot` 和语义化的 `aside#run-panel`；后续 CSS 与 JavaScript 直接依赖这些选择器。

- [ ] **Step 1: 写页面结构失败测试**

在 `turn-store/static/ui.test.mjs` 写入：

```js
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
    "composer", "prompt", "speed", "send", "stop", "new-chat",
    "messages", "empty-state", "conversation-label", "run-panel",
    "run-toggle", "run-log",
  ]) {
    assert.match(html, new RegExp(`id="${id}"`));
  }
});
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cd turn-store && node --test static/ui.test.mjs`

Expected: FAIL，提示缺少 `class="workspace"` 或 `aside#run-panel`。

- [ ] **Step 3: 实现语义化双栏骨架**

把 `index.html` 的 `<body>` 内容替换为以下结构，保留现有脚本引用：

```html
<body>
  <main class="shell">
    <header class="topbar">
      <div class="brand">
        <p class="eyebrow">AXUM · REDIS STREAM · SSE</p>
        <h1>Turn Store</h1>
      </div>
      <div class="conversation-status" aria-live="polite">
        <span id="status-dot" class="status-dot" aria-hidden="true"></span>
        <span id="conversation-status">尚未创建会话</span>
      </div>
      <button id="new-chat" type="button">新对话</button>
    </header>

    <div class="workspace">
      <div class="conversation-pane">
        <section id="messages" class="messages" aria-live="polite">
          <div id="empty-state" class="empty-state">
            <p class="eyebrow">STREAMING WORKSPACE</p>
            <h2>开始一段流式对话</h2>
            <p>首条消息创建会话，后续消息复用同一会话。</p>
          </div>
        </section>

        <section class="composer-wrap">
          <form id="composer" class="composer">
            <label class="sr-only" for="prompt">消息</label>
            <textarea
              id="prompt"
              rows="3"
              placeholder="输入消息，Enter 发送，Shift + Enter 换行"
              required
            ></textarea>
            <div class="composer-actions">
              <label class="speed-field" for="speed">
                <span>输出模式</span>
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
      </div>

      <aside id="run-panel" class="run-panel" aria-label="运行详情" hidden>
        <button id="run-toggle" class="run-toggle" type="button" aria-expanded="true">
          <span>运行详情</span>
          <span class="run-toggle-hint" aria-hidden="true">收起</span>
        </button>
        <div class="run-heading">
          <p class="eyebrow">EVENT STREAM</p>
          <h2>事件流</h2>
        </div>
        <ol id="run-log" class="run-log" role="list"></ol>
      </aside>
    </div>
  </main>
  <script type="module" src="/app.js"></script>
</body>
```

- [ ] **Step 4: 运行结构测试并确认通过**

Run: `cd turn-store && node --test static/ui.test.mjs`

Expected: 2 tests PASS。

- [ ] **Step 5: 提交页面结构**

```bash
git add turn-store/static/index.html turn-store/static/ui.test.mjs
git commit -m "界面：重构 Turn Store 页面结构"
```

### Task 2: 把运行日志升级为事件时间线

**Files:**
- Modify: `turn-store/static/ui.test.mjs`
- Modify: `turn-store/static/app.js:3-59,108-189`

**Interfaces:**
- Consumes: `aside#run-panel`、`ol#run-log`、`#conversation-status`、`#conversation-label`、`#status-dot`。
- Produces: `.run-event` 列表项、`.run-event-time`、`.run-event-name`、`.run-event-detail`；CSS 任务据此渲染时间线。

- [ ] **Step 1: 写事件时间线源码契约失败测试**

追加到 `ui.test.mjs`：

```js
test("运行事件使用结构化列表项并同步会话状态", async () => {
  const app = await read("app.js");

  assert.match(app, /createElement\("li"\)/);
  assert.match(app, /className = "run-event"/);
  assert.match(app, /runLog\.append\(item\)/);
  assert.match(app, /conversationStatus\.textContent/);
  assert.match(app, /statusDot\.classList\.toggle\("active"/);
});
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cd turn-store && node --test static/ui.test.mjs`

Expected: FAIL，提示 `app.js` 尚未创建结构化 `li.run-event`。

- [ ] **Step 3: 实现状态同步和事件列表项**

在 `elements` 中新增：

```js
conversationStatus: document.querySelector("#conversation-status"),
statusDot: document.querySelector("#status-dot"),
runToggleHint: document.querySelector(".run-toggle-hint"),
```

用以下函数替换原 `logEvent`，并新增状态同步函数：

```js
function setConversationLabel(label, active = false) {
  elements.conversationLabel.textContent = label;
  elements.conversationStatus.textContent = label;
  elements.statusDot.classList.toggle("active", active);
}

function eventDetail(event, data) {
  if (event === "text" && typeof data?.content === "string") {
    return data.content.length > 80 ? `${data.content.slice(0, 80)}…` : data.content;
  }
  if (typeof data === "string") return data;
  return JSON.stringify(data);
}

function logEvent(event, data) {
  elements.runPanel.hidden = false;

  const item = document.createElement("li");
  item.className = "run-event";

  const time = document.createElement("time");
  time.className = "run-event-time";
  time.dateTime = new Date().toISOString();
  time.textContent = new Date().toLocaleTimeString("zh-CN", { hour12: false });

  const name = document.createElement("code");
  name.className = "run-event-name";
  name.textContent = event;

  const detail = document.createElement("p");
  detail.className = "run-event-detail";
  detail.textContent = eventDetail(event, data);

  item.append(time, name, detail);
  elements.runLog.append(item);
  elements.runLog.scrollTop = elements.runLog.scrollHeight;
}
```

把 `turn_created` 分支改为：

```js
state.conversationId = payload.conversation_id;
setConversationLabel(`Conversation ${state.conversationId.slice(0, 8)}`, true);
```

把提交前日志清空和“新对话”清理改为：

```js
elements.runLog.replaceChildren();
```

在 `setStreaming(active)` 末尾加入：

```js
elements.statusDot.classList.toggle("streaming", active);
```

把折叠按钮处理改为：

```js
elements.runToggle.addEventListener("click", () => {
  const open = elements.runPanel.classList.toggle("collapsed") === false;
  elements.runToggle.setAttribute("aria-expanded", String(open));
  elements.runToggleHint.textContent = open ? "收起" : "展开";
});
```

“新对话”处理中调用：

```js
setConversationLabel("尚未创建会话");
elements.runPanel.classList.remove("collapsed");
elements.runToggle.setAttribute("aria-expanded", "true");
elements.runToggleHint.textContent = "收起";
```

- [ ] **Step 4: 运行 UI 和 SSE 测试**

Run: `cd turn-store && node --test static/ui.test.mjs static/sse.test.mjs`

Expected: 16 tests PASS（3 个 UI 契约测试 + 13 个 SSE 测试）。

- [ ] **Step 5: 提交事件时间线**

```bash
git add turn-store/static/app.js turn-store/static/ui.test.mjs
git commit -m "界面：增加运行事件时间线"
```

### Task 3: 实现 Figma 风格和响应式布局

**Files:**
- Modify: `turn-store/static/ui.test.mjs`
- Modify: `turn-store/static/styles.css:1-60`

**Interfaces:**
- Consumes: Task 1 的结构类名和 Task 2 的事件类名。
- Produces: 桌面双栏、平板折叠面板和移动端单栏三种布局；不改变 JavaScript 接口。

- [ ] **Step 1: 写视觉令牌和响应式失败测试**

追加到 `ui.test.mjs`：

```js
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
```

- [ ] **Step 2: 运行测试并确认失败**

Run: `cd turn-store && node --test static/ui.test.mjs`

Expected: FAIL，指出旧样式包含 gradient、box-shadow 且缺少目标色板。

- [ ] **Step 3: 替换基础令牌和桌面布局**

以以下令牌和布局规则重写 `styles.css`，其余组件规则继续遵守同一令牌：

```css
:root {
  color-scheme: light;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  color: #000;
  background: #fff;
  --ink: #000;
  --canvas: #fff;
  --hairline: #e6e6e6;
  --soft: #f7f7f5;
  --mint: #c8e6cd;
  --lilac: #c5b0f4;
  --cream: #f4ecd6;
  --coral: #f3c9b6;
  --success: #1ea64a;
}

* { box-sizing: border-box; }
body { margin: 0; min-height: 100vh; background: var(--canvas); color: var(--ink); }
button, textarea, select { font: inherit; }
button { min-height: 44px; border: 1px solid var(--ink); border-radius: 999px; padding: 0.65rem 1.25rem; background: var(--ink); color: #fff; font-weight: 700; cursor: pointer; }
button.secondary { background: #fff; color: #000; }
button:disabled { cursor: not-allowed; opacity: 0.42; }
button:focus-visible, textarea:focus-visible, select:focus-visible { outline: 3px solid var(--lilac); outline-offset: 2px; }

.shell { min-height: 100vh; display: grid; grid-template-rows: auto 1fr; }
.topbar { min-height: 108px; display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; gap: 2rem; padding: 1.5rem 3rem; border-bottom: 1px solid var(--hairline); }
.topbar > #new-chat { justify-self: end; }
.brand h1 { margin: 0.2rem 0 0; font-size: clamp(2rem, 3vw, 3rem); line-height: 1; letter-spacing: -0.04em; }
.eyebrow { margin: 0; font: 700 0.72rem/1.2 ui-monospace, SFMono-Regular, Menlo, monospace; letter-spacing: 0.12em; }
.conversation-status { display: flex; align-items: center; gap: 0.5rem; font: 0.78rem ui-monospace, SFMono-Regular, Menlo, monospace; }
.status-dot { width: 0.5rem; height: 0.5rem; border-radius: 50%; background: var(--hairline); }
.status-dot.active, .status-dot.streaming { background: var(--success); }

.workspace { min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr) minmax(22.5rem, 28vw); }
.conversation-pane { min-width: 0; min-height: 0; display: grid; grid-template-rows: 1fr auto; }
.messages { min-height: 0; overflow-y: auto; padding: 3rem clamp(2rem, 7vw, 7rem); }
.empty-state { min-height: 55vh; display: grid; place-content: center; justify-items: center; text-align: center; }
.empty-state h2 { margin: 0.7rem 0; font-size: clamp(2rem, 4vw, 4rem); letter-spacing: -0.05em; }
.empty-state > p:last-child { margin: 0; color: #606060; }

.message { display: grid; gap: 0.5rem; margin: 2rem 0; }
.message.user { justify-items: end; }
.role { font: 700 0.72rem ui-monospace, SFMono-Regular, Menlo, monospace; letter-spacing: 0.08em; text-transform: uppercase; }
.bubble { max-width: min(44rem, 88%); margin: 0; padding: 1.4rem 1.6rem; border-radius: 0.75rem; background: var(--cream); white-space: pre-wrap; overflow-wrap: anywhere; line-height: 1.65; }
.user .bubble { background: var(--lilac); }
.message.error .bubble { background: var(--coral); }
.cursor::after { content: ""; display: inline-block; width: 0.55rem; height: 1rem; margin-left: 0.3rem; vertical-align: -0.12rem; background: #000; animation: pulse 0.9s infinite; }
@keyframes pulse { 50% { opacity: 0.2; } }

.composer-wrap { padding: 1rem clamp(2rem, 4vw, 4rem) 2rem; border-top: 1px solid var(--hairline); background: #fff; }
.composer { border: 1px solid #000; border-radius: 0.75rem; padding: 1rem; }
.composer textarea { width: 100%; min-height: 5rem; resize: vertical; border: 0; outline: 0; padding: 0.4rem; background: transparent; color: inherit; line-height: 1.5; }
.composer-actions { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; font-size: 0.8rem; }
.speed-field { display: flex; align-items: center; gap: 0.6rem; }
.speed-field select { border: 1px solid var(--hairline); border-radius: 999px; padding: 0.55rem 0.8rem; background: #fff; }
#conversation-label { margin-right: auto; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }

.run-panel { min-height: 0; overflow: hidden; border-left: 1px solid var(--hairline); background: var(--mint); display: grid; grid-template-rows: auto auto 1fr; }
.run-panel[hidden] { display: none; }
.run-toggle { width: 100%; display: flex; justify-content: space-between; border: 0; border-radius: 0; padding: 1.5rem 2rem; background: transparent; color: #000; }
.run-heading { padding: 1rem 2rem 0; }
.run-heading h2 { margin: 0.5rem 0 1.5rem; font-size: 1.6rem; }
.run-log { min-height: 0; margin: 0; padding: 0 2rem 2rem 3.4rem; overflow-y: auto; list-style: none; }
.run-event { position: relative; display: grid; grid-template-columns: auto 1fr; gap: 0.35rem 0.8rem; padding: 0 0 1.6rem; }
.run-event::before { content: ""; position: absolute; left: -1.45rem; top: 0.25rem; width: 0.55rem; height: 0.55rem; border: 2px solid #000; border-radius: 50%; background: var(--mint); }
.run-event::after { content: ""; position: absolute; left: -1.12rem; top: 1rem; bottom: 0; width: 1px; background: #000; }
.run-event:last-child::after { display: none; }
.run-event-time, .run-event-name { font: 0.72rem ui-monospace, SFMono-Regular, Menlo, monospace; }
.run-event-detail { grid-column: 1 / -1; margin: 0; color: #39483c; font: 0.75rem/1.5 ui-monospace, SFMono-Regular, Menlo, monospace; overflow-wrap: anywhere; }
.run-panel.collapsed .run-heading, .run-panel.collapsed .run-log { display: none; }

.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
```

- [ ] **Step 4: 增加平板和移动端规则**

在 `styles.css` 末尾加入：

```css
@media (max-width: 959px) {
  .topbar { grid-template-columns: 1fr auto; }
  .conversation-status { grid-column: 1 / -1; grid-row: 2; }
  .workspace { grid-template-columns: 1fr; grid-template-rows: minmax(0, 1fr) auto; }
  .run-panel { border-left: 0; border-top: 1px solid var(--hairline); max-height: 20rem; }
  .run-panel.collapsed { max-height: 4.5rem; }
}

@media (max-width: 639px) {
  .topbar { min-height: auto; gap: 1rem; padding: 1.1rem 1rem; }
  .brand h1 { font-size: 1.8rem; }
  .topbar > #new-chat { padding-inline: 1rem; }
  .messages { padding: 1.5rem 1rem 8rem; }
  .empty-state { min-height: 42vh; }
  .bubble { max-width: 96%; }
  .composer-wrap { padding: 0.75rem 1rem 1rem; }
  .composer-actions { align-items: stretch; }
  .speed-field { width: 100%; justify-content: space-between; }
  #conversation-label { width: 100%; order: 3; }
  #stop, #send { flex: 1; }
}
```

- [ ] **Step 5: 运行 UI 契约测试**

Run: `cd turn-store && node --test static/ui.test.mjs`

Expected: 5 tests PASS。

- [ ] **Step 6: 提交视觉样式**

```bash
git add turn-store/static/styles.css turn-store/static/ui.test.mjs
git commit -m "界面：应用 Turn Store 编辑器视觉"
```

### Task 4: 回归验证和视觉对照

**Files:**
- Verify: `turn-store/static/index.html`
- Verify: `turn-store/static/styles.css`
- Verify: `turn-store/static/app.js`
- Verify: `turn-store/static/ui.test.mjs`
- Reference: `docs/superpowers/specs/assets/2026-07-10-turn-store-ui-option-2.png`

**Interfaces:**
- Consumes: 前三项任务的完整页面。
- Produces: 通过自动化测试、响应式检查和视觉对照的最终实现。

- [ ] **Step 1: 运行完整静态测试**

Run: `cd turn-store && node --test static/ui.test.mjs static/sse.test.mjs`

Expected: 18 tests PASS（5 个 UI 测试 + 13 个 SSE 测试）。

- [ ] **Step 2: 运行 Rust 格式和相关回归测试**

Run: `cd turn-store && cargo fmt --check && cargo test --test relay_policy_test`

Expected: 命令退出码为 0，`relay_policy_test` 全部 PASS。

- [ ] **Step 3: 启动页面并检查桌面视口**

Run: `python3 -m http.server 4173 --directory turn-store/static`

在 in-app Browser 打开 `http://127.0.0.1:4173/`，使用 1440×1024 视口截图。检查顶栏、空状态、输入区、纯白画布、黑色按钮以及无渐变/无阴影。静态服务器不处理 POST，因此此步只验证初始视觉状态。

- [ ] **Step 4: 检查响应式布局**

在同一浏览器依次使用 959×900 和 639×844 视口，确认检查器改为整行面板、顶栏不溢出、输入操作可点击、页面无横向滚动。每次切换视口后重新加载页面并截图。

- [ ] **Step 5: 做选中方案视觉对照**

把 1440×1024 页面截图与 `docs/superpowers/specs/assets/2026-07-10-turn-store-ui-option-2.png` 放入同一次视觉比较，逐项检查：主栏/检查器比例、顶栏高度、字体层级、薄荷色检查器、消息色块、边框、圆角、输入区和留白。若存在可见偏差，只调整 `styles.css`，然后重复截图与比较。

- [ ] **Step 6: 验证 Git 范围并提交修正**

Run: `git status --short && git diff --check`

Expected: 只有本计划列出的静态页面文件发生变化；用户原有改动保持未暂存。

如视觉验证产生修正：

```bash
git add turn-store/static/index.html turn-store/static/styles.css turn-store/static/app.js turn-store/static/ui.test.mjs
git commit -m "修复：校准 Turn Store 页面视觉"
```
