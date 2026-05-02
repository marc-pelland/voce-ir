import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { logAssistantTurn, logSystemEvent, logUserTurn, resolveSession } from "./session-manager.js";
import { readSession } from "@voce-ir/mcp-server/memory";

let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-cli-chat-test-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("resolveSession", () => {
  it("opens a fresh session when resume is null", () => {
    const r = resolveSession({ resume: null });
    expect(r.resumed).toBe(false);
    expect(r.history).toEqual([]);
    expect(r.id).toMatch(/^[0-9a-f-]{36}$/);
  });

  it("'auto' on a clean workspace falls back to a fresh session", () => {
    const r = resolveSession({ resume: "auto" });
    expect(r.resumed).toBe(false);
    expect(r.history).toEqual([]);
  });

  it("'auto' picks the most-recent prior session", async () => {
    const first = resolveSession({ resume: null });
    logUserTurn(first.id, "first session");
    // Tiny delay so mtimes differ deterministically.
    await new Promise((res) => setTimeout(res, 10));
    const second = resolveSession({ resume: null });
    logUserTurn(second.id, "second session");

    const resumed = resolveSession({ resume: "auto" });
    expect(resumed.id).toBe(second.id);
    expect(resumed.resumed).toBe(true);
    expect(resumed.history).toHaveLength(1);
    expect(resumed.history[0]?.content).toBe("second session");
  });

  it("explicit id loads that session's history", () => {
    const a = resolveSession({ resume: null });
    logUserTurn(a.id, "hello");
    logAssistantTurn(a.id, "hi back");

    const resumed = resolveSession({ resume: a.id });
    expect(resumed.resumed).toBe(true);
    expect(resumed.id).toBe(a.id);
    expect(resumed.history.map((h) => h.content)).toEqual(["hello", "hi back"]);
  });

  it("explicit id that's not on disk falls back to a fresh session", () => {
    const r = resolveSession({ resume: "definitely-not-an-id" });
    expect(r.resumed).toBe(false);
    expect(r.id).not.toBe("definitely-not-an-id");
  });
});

describe("session log writers", () => {
  it("logUserTurn appends a user entry to the session", () => {
    const r = resolveSession({ resume: null });
    logUserTurn(r.id, "build a nav");
    const entries = readSession(r.id);
    expect(entries).toHaveLength(1);
    expect(entries[0]).toMatchObject({ role: "user", content: "build a nav" });
  });

  it("logAssistantTurn carries an ir_snapshot when provided", () => {
    const r = resolveSession({ resume: null });
    logAssistantTurn(r.id, "Here's the IR", JSON.stringify({ value_type: "ViewRoot" }));
    const entries = readSession(r.id);
    expect(entries[0]?.ir_snapshot).toBe(JSON.stringify({ value_type: "ViewRoot" }));
  });

  it("logAssistantTurn omits ir_snapshot when not provided", () => {
    const r = resolveSession({ resume: null });
    logAssistantTurn(r.id, "no IR this turn");
    const entries = readSession(r.id);
    expect(entries[0]?.ir_snapshot).toBeUndefined();
  });

  it("logSystemEvent records system role entries", () => {
    const r = resolveSession({ resume: null });
    logSystemEvent(r.id, "voce-chat started");
    const entries = readSession(r.id);
    expect(entries[0]).toMatchObject({ role: "system", content: "voce-chat started" });
  });

  it("turns from an explicit-id resume show up in the loaded history", () => {
    const first = resolveSession({ resume: null });
    logUserTurn(first.id, "hi");
    logAssistantTurn(first.id, "hello");
    logUserTurn(first.id, "build a thing");

    const resumed = resolveSession({ resume: first.id });
    expect(resumed.history.map((h) => h.role)).toEqual(["user", "assistant", "user"]);
  });
});
