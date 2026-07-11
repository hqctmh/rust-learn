import { createSseParser, parseAgentEventData } from "./sse.js";
import { createRunEventQueue } from "./run-events.js";

const MAX_RUN_EVENTS = 200;
const RUN_LOG_FLUSH_INTERVAL_MS = 100;

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
  conversationStatus: document.querySelector("#conversation-status"),
  statusDot: document.querySelector("#status-dot"),
  workspace: document.querySelector(".workspace"),
  runPanel: document.querySelector("#run-panel"),
  runToggle: document.querySelector("#run-toggle"),
  runToggleHint: document.querySelector(".run-toggle-hint"),
  runLog: document.querySelector("#run-log"),
};

const state = {
  conversationId: null,
  controller: null,
  streaming: false,
  draftDocId: crypto.randomUUID(),
  runLogTimer: null,
};

const runEventQueue = createRunEventQueue(MAX_RUN_EVENTS);

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
  elements.statusDot.classList.toggle("streaming", active);
}

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

function createRunEventItem({ event, detail, dateTime, timeLabel }) {
  const item = document.createElement("li");
  item.className = "run-event";

  const time = document.createElement("time");
  time.className = "run-event-time";
  time.dateTime = dateTime;
  time.textContent = timeLabel;

  const name = document.createElement("code");
  name.className = "run-event-name";
  name.textContent = event;

  const detailNode = document.createElement("p");
  detailNode.className = "run-event-detail";
  detailNode.textContent = detail;

  item.append(time, name, detailNode);
  return item;
}

function flushRunEvents() {
  state.runLogTimer = null;
  if (elements.runPanel.classList.contains("collapsed")) return;

  const entries = runEventQueue.drain();
  if (entries.length === 0) return;

  const fragment = document.createDocumentFragment();
  for (const entry of entries) fragment.append(createRunEventItem(entry));
  elements.runLog.append(fragment);

  while (elements.runLog.childElementCount > MAX_RUN_EVENTS) {
    elements.runLog.firstElementChild.remove();
  }
  elements.runLog.scrollTop = elements.runLog.scrollHeight;
}

function scheduleRunEventFlush() {
  if (elements.runPanel.classList.contains("collapsed") || state.runLogTimer !== null) return;
  state.runLogTimer = setTimeout(flushRunEvents, RUN_LOG_FLUSH_INTERVAL_MS);
}

function resetRunEvents() {
  if (state.runLogTimer !== null) clearTimeout(state.runLogTimer);
  state.runLogTimer = null;
  runEventQueue.clear();
  elements.runLog.replaceChildren();
  elements.runPanel.hidden = true;
  elements.workspace.classList.remove("has-run-panel", "run-panel-collapsed");
}

function logEvent(event, data) {
  const now = new Date();
  elements.runPanel.hidden = false;
  elements.workspace.classList.add("has-run-panel");
  runEventQueue.push({
    event,
    detail: eventDetail(event, data),
    dateTime: now.toISOString(),
    timeLabel: now.toLocaleTimeString("zh-CN", { hour12: false }),
  });
  scheduleRunEventFlush();
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
  let pendingText = "";
  let textFrame = null;
  const flushText = () => {
    textFrame = null;
    if (pendingText.length === 0) return;
    assistant.bubble.append(pendingText);
    pendingText = "";
    scrollToBottom();
  };
  const appendText = (content) => {
    pendingText += content;
    if (textFrame === null) textFrame = requestAnimationFrame(flushText);
  };
  resetRunEvents();
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
      const payload = parseAgentEventData(event, data);
      logEvent(event, payload);

      if (event === "turn_created") {
        state.conversationId = payload.conversation_id;
        setConversationLabel(`Conversation ${state.conversationId.slice(0, 8)}`, true);
      } else if (event === "text" && typeof payload.content === "string") {
        appendText(payload.content);
      } else if (event === "run_completed") {
        terminal = true;
      } else if (event === "error") {
        terminal = true;
        throw new Error(payload.message || "流式响应失败");
      }
    });
    if (!terminal) throw new Error("SSE 在终止事件前结束");
  } catch (error) {
    if (textFrame !== null) cancelAnimationFrame(textFrame);
    flushText();
    if (error.name === "AbortError") {
      assistant.bubble.textContent ||= "已停止接收；服务端仍会完成本轮处理。";
    } else {
      assistant.article.classList.add("error");
      assistant.bubble.textContent ||= error.message;
      logEvent("error", error.message);
    }
  } finally {
    if (textFrame !== null) cancelAnimationFrame(textFrame);
    flushText();
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
  resetRunEvents();
  setConversationLabel("尚未创建会话");
  elements.runPanel.classList.remove("collapsed");
  elements.runToggle.setAttribute("aria-expanded", "true");
  elements.runToggleHint.textContent = "收起";
  elements.prompt.focus();
});
elements.runToggle.addEventListener("click", () => {
  const open = elements.runPanel.classList.toggle("collapsed") === false;
  elements.workspace.classList.toggle("run-panel-collapsed", !open);
  elements.runToggle.setAttribute("aria-expanded", String(open));
  elements.runToggleHint.textContent = open ? "收起" : "展开";
  if (open) scheduleRunEventFlush();
});
