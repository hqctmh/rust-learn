export function parseAgentEventData(event, data) {
  let payload;
  try {
    payload = JSON.parse(data);
  } catch {
    throw new Error(`${event} 事件 data 不是有效 JSON`);
  }

  if (payload === null || typeof payload !== "object" || Array.isArray(payload)) {
    throw new Error(`${event} 事件 data 必须是 JSON 对象`);
  }

  let requiredStringField;
  if (event === "turn_created") requiredStringField = "conversation_id";
  if (event === "text") requiredStringField = "content";
  if (event === "error") requiredStringField = "message";
  if (requiredStringField && typeof payload[requiredStringField] !== "string") {
    throw new Error(`${event}.${requiredStringField} 必须是字符串`);
  }

  return payload;
}

export function createSseParser(onEvent) {
  let lineBuffer = "";
  let skipLfAfterCr = false;
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
      event: eventName || "message",
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
      for (const character of chunk) {
        if (skipLfAfterCr) {
          skipLfAfterCr = false;
          if (character === "\n") continue;
        }

        if (character === "\r") {
          consumeLine(lineBuffer);
          lineBuffer = "";
          skipLfAfterCr = true;
        } else if (character === "\n") {
          consumeLine(lineBuffer);
          lineBuffer = "";
        } else {
          lineBuffer += character;
        }
      }
    },
    finish() {
      if (lineBuffer !== "") {
        consumeLine(lineBuffer);
        lineBuffer = "";
      }
      skipLfAfterCr = false;
      dispatch();
    },
  };
}
