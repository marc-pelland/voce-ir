// Tests for the .voce/ storage layer. Each test isolates VOCE_PROJECT_ROOT
// to a fresh temp directory so suites cannot collide and state never leaks
// across runs.

import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { atomicWriteFile, appendJsonlLine, readJsonlFile } from "./atomic.js";
import { readBrief, writeBrief } from "./brief.js";
import { getDecision, listDecisions, logDecision } from "./decisions.js";
import { listDrift, logDrift } from "./drift.js";
import { appendSession, latestIrSnapshot, listSessions, newSessionId, readSession } from "./session.js";
import { PATHS } from "./store.js";
import { validateDecision, validateDriftWarning, validateSessionEntry } from "./types.js";

let workspace: string;

beforeEach(() => {
  workspace = mkdtempSync(join(tmpdir(), "voce-memory-test-"));
  process.env.VOCE_PROJECT_ROOT = workspace;
});

afterEach(() => {
  rmSync(workspace, { recursive: true, force: true });
  delete process.env.VOCE_PROJECT_ROOT;
});

describe("atomicWriteFile", () => {
  it("writes content visible to readers", () => {
    const target = join(workspace, "atomic.txt");
    atomicWriteFile(target, "hello");
    expect(readFileSync(target, "utf8")).toBe("hello");
  });

  it("creates parent directories when missing", () => {
    const target = join(workspace, "nested", "deep", "file.txt");
    atomicWriteFile(target, "x");
    expect(readFileSync(target, "utf8")).toBe("x");
  });

  it("replaces existing content atomically", () => {
    const target = join(workspace, "atomic.txt");
    atomicWriteFile(target, "first");
    atomicWriteFile(target, "second");
    expect(readFileSync(target, "utf8")).toBe("second");
  });

  it("does not leave temp files behind on success", () => {
    const target = join(workspace, "atomic.txt");
    atomicWriteFile(target, "x");
    const siblings = readdirSync(workspace);
    expect(siblings.filter((f) => f.endsWith(".tmp"))).toEqual([]);
  });
});

describe("appendJsonlLine + readJsonlFile", () => {
  it("appends one line per call", () => {
    const target = join(workspace, "log.jsonl");
    appendJsonlLine(target, { a: 1 });
    appendJsonlLine(target, { a: 2 });
    const raw = readFileSync(target, "utf8");
    expect(raw).toBe(`{"a":1}\n{"a":2}\n`);
  });

  it("never rewrites earlier lines", () => {
    const target = join(workspace, "log.jsonl");
    appendJsonlLine(target, { a: 1 });
    const before = readFileSync(target, "utf8");
    appendJsonlLine(target, { a: 2 });
    const after = readFileSync(target, "utf8");
    expect(after.startsWith(before)).toBe(true);
  });

  it("returns malformed lines in errors[] without throwing", () => {
    const target = join(workspace, "log.jsonl");
    appendJsonlLine(target, { id: "ok", timestamp: new Date().toISOString(), summary: "s", rationale: "r" });
    // Inject a garbage line by hand.
    writeFileSync(target, readFileSync(target, "utf8") + "not json\n");
    appendJsonlLine(target, { id: "ok2", timestamp: new Date().toISOString(), summary: "s2", rationale: "r2" });
    const { entries, errors } = readJsonlFile(target, validateDecision);
    expect(entries).toHaveLength(2);
    expect(errors).toHaveLength(1);
    expect(errors[0]?.line).toBe(2);
  });

  it("returns empty when the file does not exist", () => {
    const { entries, errors } = readJsonlFile(join(workspace, "missing.jsonl"), validateDecision);
    expect(entries).toEqual([]);
    expect(errors).toEqual([]);
  });
});

describe("brief", () => {
  it("returns null when no brief exists", () => {
    expect(readBrief()).toBeNull();
  });

  it("round-trips markdown content with last_modified", () => {
    writeBrief("# Hello\n\nbody");
    const brief = readBrief();
    expect(brief?.content).toBe("# Hello\n\nbody");
    expect(brief?.last_modified).toBe(statSync(PATHS.brief()).mtime.toISOString());
  });

  it("replaces an existing brief atomically", () => {
    writeBrief("first");
    writeBrief("second");
    expect(readBrief()?.content).toBe("second");
  });
});

describe("decisions", () => {
  it("starts empty", () => {
    expect(listDecisions()).toEqual([]);
  });

  it("logs and lists decisions in order", () => {
    const a = logDecision({ summary: "A", rationale: "r1" });
    const b = logDecision({ summary: "B", rationale: "r2" });
    const all = listDecisions();
    expect(all.map((d) => d.id)).toEqual([a.id, b.id]);
  });

  it("auto-generates id and timestamp", () => {
    const d = logDecision({ summary: "x", rationale: "y" });
    expect(d.id).toMatch(/^[0-9a-f-]{36}$/);
    expect(new Date(d.timestamp).toISOString()).toBe(d.timestamp);
  });

  it("rejects empty summary", () => {
    expect(() => logDecision({ summary: "", rationale: "r" })).toThrow(/summary missing/);
  });

  it("filters by since", () => {
    const past = new Date(Date.now() - 60_000).toISOString();
    const future = new Date(Date.now() + 60_000).toISOString();
    logDecision({ summary: "old", rationale: "r", timestamp: past });
    logDecision({ summary: "new", rationale: "r" });
    expect(listDecisions({ since: future })).toEqual([]);
    expect(listDecisions({ since: past })).toHaveLength(2);
  });

  it("getDecision finds by id and returns null for misses", () => {
    const d = logDecision({ summary: "x", rationale: "y" });
    expect(getDecision(d.id)?.summary).toBe("x");
    expect(getDecision("unknown-id")).toBeNull();
  });

  it("supports supersedes / conflicts_with", () => {
    const a = logDecision({ summary: "old", rationale: "r" });
    const b = logDecision({
      summary: "new",
      rationale: "r2",
      supersedes: a.id,
      conflicts_with: a.id,
    });
    expect(b.supersedes).toBe(a.id);
    expect(b.conflicts_with).toBe(a.id);
  });
});

describe("drift", () => {
  it("logs and lists drift warnings, defaulting to pending", () => {
    logDrift({ decision_id: "d1", drift_description: "conflict A" });
    const all = listDrift();
    expect(all).toHaveLength(1);
    expect(all[0]?.resolution).toBe("pending");
  });

  it("rejects invalid resolution", () => {
    expect(() =>
      logDrift({
        decision_id: "d1",
        drift_description: "x",
        // @ts-expect-error — testing runtime validation
        resolution: "maybe",
      }),
    ).toThrow(/resolution must be one of/);
  });
});

describe("session", () => {
  it("appends and reads in order", () => {
    const id = newSessionId();
    appendSession(id, { role: "user", content: "hi" });
    appendSession(id, { role: "assistant", content: "hey" });
    const log = readSession(id);
    expect(log.map((e) => e.role)).toEqual(["user", "assistant"]);
  });

  it("listSessions reports counts and sorts most-recent-first", async () => {
    const a = newSessionId();
    appendSession(a, { role: "user", content: "first" });
    // Tiny delay so mtimes differ deterministically on fast filesystems.
    await new Promise((r) => setTimeout(r, 10));
    const b = newSessionId();
    appendSession(b, { role: "user", content: "second" });

    const sessions = listSessions();
    expect(sessions[0]?.id).toBe(b);
    expect(sessions[1]?.id).toBe(a);
    expect(sessions.find((s) => s.id === a)?.entry_count).toBe(1);
  });

  it("rejects an unknown role", () => {
    expect(() =>
      appendSession(newSessionId(), {
        // @ts-expect-error — testing runtime validation
        role: "bot",
        content: "x",
      }),
    ).toThrow(/role must be one of/);
  });

  it("preserves and surfaces ir_snapshot via latestIrSnapshot", () => {
    const id = newSessionId();
    appendSession(id, { role: "user", content: "make a hero" });
    appendSession(id, {
      role: "assistant",
      content: "draft 1",
      ir_snapshot: JSON.stringify({ version: 1 }),
    });
    appendSession(id, { role: "user", content: "swap headline" });
    appendSession(id, {
      role: "assistant",
      content: "draft 2",
      ir_snapshot: JSON.stringify({ version: 2 }),
    });
    appendSession(id, { role: "system", content: "validated" });

    expect(latestIrSnapshot(id)).toBe(JSON.stringify({ version: 2 }));
  });

  it("latestIrSnapshot returns null when no entry has one", () => {
    const id = newSessionId();
    appendSession(id, { role: "user", content: "hi" });
    appendSession(id, { role: "assistant", content: "hello" });
    expect(latestIrSnapshot(id)).toBeNull();
  });
});

describe("type validators (boundary cases)", () => {
  it("validateDecision rejects non-ISO timestamps", () => {
    expect(validateDecision({
      id: "x",
      timestamp: "not a date",
      summary: "s",
      rationale: "r",
    })).toMatch(/timestamp/);
  });

  it("validateDriftWarning rejects empty drift_description", () => {
    expect(validateDriftWarning({
      timestamp: new Date().toISOString(),
      decision_id: "d",
      drift_description: "",
      resolution: "pending",
    })).toMatch(/drift_description/);
  });

  it("validateSessionEntry rejects missing content", () => {
    expect(validateSessionEntry({
      timestamp: new Date().toISOString(),
      role: "user",
    })).toMatch(/content/);
  });
});
