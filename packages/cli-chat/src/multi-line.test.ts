import { describe, expect, it } from "vitest";
import { clearPending, createAccumulator, feedLine } from "./multi-line.js";

describe("multi-line accumulator", () => {
  it("submits a single line that doesn't end with backslash", () => {
    const acc = createAccumulator();
    expect(feedLine(acc, "hello")).toEqual({ kind: "submit", text: "hello" });
    expect(acc.pending).toEqual([]);
  });

  it("ignores blank lines when nothing is pending", () => {
    const acc = createAccumulator();
    expect(feedLine(acc, "")).toEqual({ kind: "empty" });
  });

  it("queues lines that end with backslash and submits when one doesn't", () => {
    const acc = createAccumulator();
    expect(feedLine(acc, "first line \\")).toEqual({ kind: "continue" });
    expect(feedLine(acc, "second line")).toEqual({
      kind: "submit",
      text: "first line \nsecond line",
    });
  });

  it("supports a third line by chaining backslashes", () => {
    const acc = createAccumulator();
    feedLine(acc, "line one\\");
    feedLine(acc, "line two\\");
    const r = feedLine(acc, "line three");
    expect(r).toEqual({ kind: "submit", text: "line one\nline two\nline three" });
  });

  it("clearPending drops pending state and reports whether anything was cleared", () => {
    const acc = createAccumulator();
    expect(clearPending(acc)).toBe(false);
    feedLine(acc, "queued\\");
    expect(clearPending(acc)).toBe(true);
    expect(clearPending(acc)).toBe(false);
  });

  it("submits an empty line when pending is non-empty (terminator)", () => {
    // Trailing backslash + blank line: backslash means "continue", blank
    // should NOT start submission. The accumulator keeps pending and emits
    // 'submit' only when a non-backslash line comes in.
    const acc = createAccumulator();
    feedLine(acc, "first \\");
    const r = feedLine(acc, "");
    expect(r).toEqual({ kind: "submit", text: "first \n" });
  });
});
