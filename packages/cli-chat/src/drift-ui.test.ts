import { describe, expect, it } from "vitest";
import {
  driftResolutionAsDecision,
  formatDriftReport,
  parseDriftChoice,
  DRIFT_PROMPT,
} from "./drift-ui.js";
import type { DriftReport } from "@voce-ir/mcp-server/memory";

const sample: DriftReport = {
  decision_id: "abcdef12-1234-5678-9abc-def123456789",
  decision_summary: "no Modal nodes",
  decision_timestamp: "2026-04-12T00:00:00.000Z",
  matched_terms: ["modal", "trap"],
  note: "v1 keyword heuristic",
};

describe("formatDriftReport", () => {
  it("includes the short id, date, and matched terms", () => {
    const out = formatDriftReport(sample);
    expect(out).toMatch(/abcdef12/);
    expect(out).toMatch(/2026-04-12/);
    expect(out).toMatch(/modal, trap/);
    expect(out).toMatch(/no Modal nodes/);
  });
});

describe("parseDriftChoice", () => {
  it.each([
    ["r", "revise"],
    ["revise", "revise"],
    ["s", "supersede"],
    ["supersede", "supersede"],
    ["c", "continue"],
    ["continue", "continue"],
    ["", null],
    ["x", null],
  ])("'%s' → %s", (input, expected) => {
    expect(parseDriftChoice(input)).toBe(expected);
  });
});

describe("driftResolutionAsDecision", () => {
  it("supersede produces a decision with the supersedes field set", () => {
    const out = driftResolutionAsDecision(sample, "supersede");
    expect(out.supersedes).toBe(sample.decision_id);
    expect(out.summary).toMatch(/Supersedes decision abcdef12/);
  });

  it("continue produces a decision with conflicts_with set, supersedes undefined", () => {
    const out = driftResolutionAsDecision(sample, "continue");
    expect(out.conflicts_with).toBe(sample.decision_id);
    expect(out.supersedes).toBeUndefined();
    expect(out.summary).toMatch(/Knowingly continued/);
  });
});

describe("DRIFT_PROMPT", () => {
  it("offers all three options", () => {
    expect(DRIFT_PROMPT).toMatch(/r/);
    expect(DRIFT_PROMPT).toMatch(/s/);
    expect(DRIFT_PROMPT).toMatch(/c/);
  });
});
