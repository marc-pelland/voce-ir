import { describe, expect, it } from "vitest";
import {
  formatReadinessReport,
  parseReadinessChoice,
  READINESS_PROMPT,
} from "./readiness-ui.js";

describe("formatReadinessReport", () => {
  it("includes the score in the header", () => {
    const out = formatReadinessReport({
      score: 80,
      ready: true,
      missing: [],
      blocking: [],
    });
    expect(out).toMatch(/80\/100/);
  });

  it("shows 'all checkpoints satisfied' when nothing is missing", () => {
    const out = formatReadinessReport({ score: 100, ready: true, missing: [], blocking: [] });
    expect(out).toMatch(/All discovery checkpoints satisfied/);
  });

  it("lists each missing item on its own line", () => {
    const out = formatReadinessReport({
      score: 40,
      ready: false,
      missing: ["empty states not specified", "tablet breakpoint not specified"],
      blocking: [],
    });
    expect(out).toMatch(/empty states not specified/);
    expect(out).toMatch(/tablet breakpoint not specified/);
  });

  it("calls out blocking items separately", () => {
    const out = formatReadinessReport({
      score: 0,
      ready: false,
      missing: [],
      blocking: ["user_intent"],
    });
    expect(out).toMatch(/Blocking: user_intent/);
  });
});

describe("parseReadinessChoice", () => {
  it.each([
    ["", "proceed"],
    ["y", "proceed"],
    ["Y", "proceed"],
    ["yes", "proceed"],
    ["YES", "proceed"],
    ["q", "ask-question"],
    ["question", "ask-question"],
    ["ask", "ask-question"],
    ["n", "abort"],
    ["no", "abort"],
    ["nope", "abort"],
    ["whatever", "abort"],
  ])("'%s' → %s", (input, expected) => {
    expect(parseReadinessChoice(input)).toBe(expected);
  });
});

describe("READINESS_PROMPT", () => {
  it("calls out all three options", () => {
    expect(READINESS_PROMPT).toMatch(/Y/);
    expect(READINESS_PROMPT).toMatch(/n/);
    expect(READINESS_PROMPT).toMatch(/q/);
  });
});
