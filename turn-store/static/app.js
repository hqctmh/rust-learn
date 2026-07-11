import { createSseParser, parseAgentEventData } from "./sse.js";

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
  elements.runLog.replaceChildren();
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
      const payload = parseAgentEventData(event, data);
      logEvent(event, payload);

      if (event === "turn_created") {
        state.conversationId = payload.conversation_id;
        setConversationLabel(`Conversation ${state.conversationId.slice(0, 8)}`, true);
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
  elements.runLog.replaceChildren();
  setConversationLabel("尚未创建会话");
  elements.runPanel.classList.remove("collapsed");
  elements.runToggle.setAttribute("aria-expanded", "true");
  elements.runToggleHint.textContent = "收起";
  elements.prompt.focus();
});
elements.runToggle.addEventListener("click", () => {
  const open = elements.runPanel.classList.toggle("collapsed") === false;
  elements.runToggle.setAttribute("aria-expanded", String(open));
  elements.runToggleHint.textContent = open ? "收起" : "展开";
});
