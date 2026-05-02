// Drift detector v1 tests. The detector is intentionally conservative —
// these tests pin its observed behavior so future refactors don't silently
// change false-positive / false-negative rates.

import { describe, expect, it } from "vitest";
import { detectDrift } from "./drift-check.js";
import type { Decision } from "./types.js";

function decision(partial: Partial<Decision>): Decision {
  return {
    id: "test-id",
    timestamp: "2026-05-02T00:00:00.000Z",
    summary: "summary",
    rationale: "rationale",
    ...partial,
  };
}

describe("detectDrift", () => {
  it("returns empty when there are no decisions", () => {
    const result = detectDrift("{}", []);
    expect(result.drift).toEqual([]);
    expect(result.decisions_referenced).toEqual([]);
  });

  it("returns empty when nothing in IR matches", () => {
    const ir = JSON.stringify({ root: { children: [] } });
    const decisions: Decision[] = [
      decision({ id: "d1", summary: "Avoid Modal nodes", rationale: "Modals trap focus" }),
    ];
    expect(detectDrift(ir, decisions).drift).toEqual([]);
  });

  it("flags a decision when one of its terms appears in the IR", () => {
    const ir = JSON.stringify({ value_type: "Modal", title: "Confirm" });
    const decisions: Decision[] = [
      decision({ id: "d1", summary: "Avoid Modal nodes", rationale: "Modals trap focus" }),
    ];
    const result = detectDrift(ir, decisions);
    expect(result.drift).toHaveLength(1);
    expect(result.drift[0]?.decision_id).toBe("d1");
    expect(result.drift[0]?.matched_terms).toContain("modal");
    expect(result.decisions_referenced).toEqual(["d1"]);
  });

  it("ignores stopwords like 'always', 'never', 'should'", () => {
    // The IR and decision both contain "always" / "never" / "should", but no
    // domain terms in common. Stopword filtering should keep these from
    // generating a false-positive match.
    const ir = JSON.stringify({ note: "always never should within between" });
    const decisions: Decision[] = [
      decision({
        id: "d1",
        summary: "always never should",
        rationale: "within between further",
      }),
    ];
    expect(detectDrift(ir, decisions).drift).toEqual([]);
  });

  it("ignores tokens shorter than 5 characters", () => {
    // "form" is 4 chars — below the threshold, so a decision about "forms"
    // (5 chars) won't match an IR that only contains "form".
    const ir = JSON.stringify({ value_type: "form" });
    const decisions: Decision[] = [
      decision({ id: "d1", summary: "no abc def ghi", rationale: "irrelevant" }),
    ];
    expect(detectDrift(ir, decisions).drift).toEqual([]);
  });

  it("matches case-insensitively", () => {
    const ir = JSON.stringify({ value_type: "MODAL" });
    const decisions: Decision[] = [
      decision({ id: "d1", summary: "no modal nodes", rationale: "x x" }),
    ];
    expect(detectDrift(ir, decisions).drift).toHaveLength(1);
  });

  it("deduplicates terms within a single decision", () => {
    const ir = JSON.stringify({ value_type: "Modal", child: { value_type: "Modal" } });
    const decisions: Decision[] = [
      decision({
        id: "d1",
        summary: "modal modal modal",
        rationale: "Modal Modal",
      }),
    ];
    const result = detectDrift(ir, decisions);
    expect(result.drift).toHaveLength(1);
    expect(result.drift[0]?.matched_terms.filter((t) => t === "modal")).toHaveLength(1);
  });

  it("emits a heuristic note in every report so callers know it's v1", () => {
    const ir = JSON.stringify({ value_type: "Modal" });
    const decisions: Decision[] = [decision({ id: "d1", summary: "no modal", rationale: "x" })];
    const result = detectDrift(ir, decisions);
    expect(result.drift[0]?.note).toMatch(/keyword heuristic|review/i);
  });

  it("returns multiple reports when multiple decisions match", () => {
    const ir = JSON.stringify({ a: "modal", b: "carousel" });
    const decisions: Decision[] = [
      decision({ id: "d1", summary: "no modal", rationale: "x" }),
      decision({ id: "d2", summary: "no carousel", rationale: "y" }),
    ];
    const result = detectDrift(ir, decisions);
    expect(result.drift).toHaveLength(2);
    expect(result.decisions_referenced.sort()).toEqual(["d1", "d2"]);
  });
});
