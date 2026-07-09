export function createSseParser(onEvent) {
  let buffer = "";
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
      event: eventName,
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
      buffer += chunk;
      let newline = buffer.indexOf("\n");
      while (newline !== -1) {
        consumeLine(buffer.slice(0, newline));
        buffer = buffer.slice(newline + 1);
        newline = buffer.indexOf("\n");
      }
    },
    finish() {
      if (buffer !== "") {
        consumeLine(buffer);
        buffer = "";
      }
      dispatch();
    },
  };
}
