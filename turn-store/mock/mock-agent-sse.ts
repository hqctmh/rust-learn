import { randomUUID } from "node:crypto";
import { readFile } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";

type Speed = "fast" | "slow";

type AgentEvent =
  | { type: "run_created"; id: string; speed: Speed; content: string }
  | { type: "message_created"; id: string; role: "assistant"; content: string }
  | { type: "status"; id: string; stage: string; content: string }
  | { type: "thinking"; id: string; content: string }
  | { type: "step_started"; id: string; step: string; content: string }
  | { type: "tool_call"; id: string; name: string; args: Record<string, string>; content: string }
  | { type: "tool_result"; id: string; name: string; content: string; bytes: number }
  | { type: "step_finished"; id: string; step: string; content: string }
  | { type: "text"; id: string; index: number; content: string }
  | { type: "checkpoint"; id: string; index: number; content: string }
  | { type: "usage"; id: string; input_tokens: number; output_tokens: number }
  | { type: "message_completed"; id: string; content: string }
  | { type: "run_completed"; id: string; content: string }
  | { type: "error"; id: string; message: string };

const PORT = Number(process.env.PORT ?? 8787);
const DESIGN_FILE = new URL("./DESIGN-figma.md", import.meta.url);

const DELAY_MS: Record<Speed, number> = {
  fast: 8,
  slow: 8,
};

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function getSpeed(url: URL): Speed {
  const speed = url.searchParams.get("speed");
  if (speed === "slow" || url.pathname.endsWith("/slow")) {
    return "slow";
  }
  return "fast";
}

async function buildEvents(speed: Speed): Promise<AgentEvent[]> {
  const id = randomUUID();
  const designContent = await readFile(DESIGN_FILE, "utf8");
  const textChunks = chunkText(designContent, speed);
  const textEvents = textChunks.map<AgentEvent>((content, index) => ({
    type: "text",
    id,
    index,
    content,
  }));

  return [
    { type: "run_created", id, speed, content: "mock agent run created" },
    { type: "message_created", id, role: "assistant", content: "assistant message created" },
    { type: "status", id, stage: "queued", content: "request queued" },
    { type: "status", id, stage: "started", content: "mock agent started" },
    { type: "thinking", id, content: "分析请求，准备读取 Figma 设计说明" },
    { type: "step_started", id, step: "load_design_file", content: "loading DESIGN-figma.md" },
    {
      type: "tool_call",
      id,
      name: "read_design_file",
      args: { path: "turn-store/mock/DESIGN-figma.md" },
      content: "read file content",
    },
    {
      type: "tool_result",
      id,
      name: "read_design_file",
      content: "DESIGN-figma.md loaded",
      bytes: Buffer.byteLength(designContent, "utf8"),
    },
    { type: "step_finished", id, step: "load_design_file", content: "design file ready" },
    { type: "thinking", id, content: `开始按 ${describeTextMode(speed)} 流式返回正文` },
    { type: "checkpoint", id, index: 0, content: "text stream begin" },
    ...withCheckpoints(textEvents, id),
    { type: "checkpoint", id, index: textEvents.length, content: "text stream end" },
    {
      type: "usage",
      id,
      input_tokens: 256,
      output_tokens: textChunks.length,
    },
    { type: "message_completed", id, content: "assistant message completed" },
    { type: "run_completed", id, content: "mock agent run completed" },
  ];
}

function chunkText(content: string, speed: Speed): string[] {
  if (speed === "fast") {
    return content.split(/\r?\n/).map((line, index, lines) => (index === lines.length - 1 ? line : `${line}\n`));
  }

  const chars = Array.from(content);
  const chunks: string[] = [];

  for (let i = 0; i < chars.length; ) {
    const size = 5 + (i % 6);
    chunks.push(chars.slice(i, i + size).join(""));
    i += size;
  }

  return chunks;
}

function describeTextMode(speed: Speed): string {
  return speed === "fast" ? "一行一条 text 事件" : "5-10 个字符一条 text 事件";
}

function withCheckpoints(textEvents: AgentEvent[], id: string): AgentEvent[] {
  const events: AgentEvent[] = [];

  for (const event of textEvents) {
    events.push(event);

    if (event.type === "text" && event.index > 0 && event.index % 200 === 0) {
      events.push({
        type: "checkpoint",
        id,
        index: event.index,
        content: `streamed ${event.index} text chunks`,
      });
    }
  }

  return events;
}

function writeSse(res: ServerResponse, event: AgentEvent): void {
  res.write(`event: ${event.type}\n`);
  res.write(`data: ${JSON.stringify(event)}\n\n`);
}

async function streamAgent(res: ServerResponse, speed: Speed): Promise<void> {
  res.writeHead(200, {
    "Content-Type": "text/event-stream; charset=utf-8",
    "Cache-Control": "no-cache, no-transform",
    Connection: "keep-alive",
    "X-Accel-Buffering": "no",
  });

  for (const event of await buildEvents(speed)) {
    writeSse(res, event);
    await sleep(DELAY_MS[speed]);
  }

  res.end();
}

const server = createServer((req, res) => {
  const url = new URL(req.url ?? "/", `http://${req.headers.host ?? "localhost"}`);

  if (url.pathname === "/health") {
    res.writeHead(200, { "Content-Type": "application/json; charset=utf-8" });
    res.end(JSON.stringify({ ok: true }));
    return;
  }

  if (url.pathname === "/events" || url.pathname === "/events/fast" || url.pathname === "/events/slow") {
    void streamAgent(res, getSpeed(url)).catch((error: unknown) => {
      if (!res.headersSent) {
        res.writeHead(500, {
          "Content-Type": "text/event-stream; charset=utf-8",
          "Cache-Control": "no-cache, no-transform",
          Connection: "keep-alive",
        });
      }
      const message = error instanceof Error ? error.message : "unknown error";
      writeSse(res, { type: "error", id: "mock-agent", message });
      res.end();
    });
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json; charset=utf-8" });
  res.end(
    JSON.stringify({
      error: "not found",
      endpoints: ["/health", "/events?speed=fast", "/events?speed=slow", "/events/fast", "/events/slow"],
    }),
  );
});

server.listen(PORT, () => {
  console.log(`mock agent sse server listening on http://127.0.0.1:${PORT}`);
  console.log(`fast: curl -N http://127.0.0.1:${PORT}/events?speed=fast`);
  console.log(`slow: curl -N http://127.0.0.1:${PORT}/events?speed=slow`);
});
