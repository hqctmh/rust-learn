export function createRunEventQueue(limit) {
  if (!Number.isInteger(limit) || limit < 1) {
    throw new RangeError("limit must be a positive integer");
  }

  let entries = [];

  return {
    push(entry) {
      entries.push(entry);
      if (entries.length > limit) {
        entries.splice(0, entries.length - limit);
      }
    },
    drain() {
      const drained = entries;
      entries = [];
      return drained;
    },
    clear() {
      entries = [];
    },
  };
}
