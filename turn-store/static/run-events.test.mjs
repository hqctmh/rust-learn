import assert from "node:assert/strict";
import test from "node:test";

import { createRunEventQueue } from "./run-events.js";

test("运行事件队列只保留最近的事件", () => {
  const queue = createRunEventQueue(3);

  queue.push({ event: "one" });
  queue.push({ event: "two" });
  queue.push({ event: "three" });
  queue.push({ event: "four" });

  assert.deepEqual(queue.drain(), [
    { event: "two" },
    { event: "three" },
    { event: "four" },
  ]);
});

test("读取事件后清空队列", () => {
  const queue = createRunEventQueue(2);
  queue.push({ event: "one" });

  assert.deepEqual(queue.drain(), [{ event: "one" }]);
  assert.deepEqual(queue.drain(), []);

  queue.push({ event: "two" });
  queue.clear();
  assert.deepEqual(queue.drain(), []);
});
