import { describe, expect, it } from "vitest";
import { buildSystemPrompt, SYSTEM_PROMPT_PILLARS } from "./prompt.js";

describe("buildSystemPrompt", () => {
  it("encodes all six pillars verbatim", () => {
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/ONE QUESTION AT A TIME/);
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/BUILD A PROJECT PROFILE/);
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/FULL-STACK COMPLETENESS/);
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/PUSH BACK CONCISELY/);
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/SHARE EXPERTISE PROACTIVELY/);
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/EARN THE RIGHT TO PROPOSE/);
  });

  it("calls out the readiness 70 gate explicitly", () => {
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/readiness score is ≥\s*70|readiness.*≥\s*70/);
  });

  it("instructs the model to consult voce_check_drift before proposing", () => {
    expect(SYSTEM_PROMPT_PILLARS).toMatch(/voce_check_drift/);
  });

  it("inserts the brief content when present", () => {
    const prompt = buildSystemPrompt({
      brief: { content: "# My App\n\nB2B SaaS for X", last_modified: "2026-05-02T00:00:00.000Z" },
      recentDecisions: [],
    });
    expect(prompt).toMatch(/B2B SaaS for X/);
    expect(prompt).toMatch(/Current project brief/);
  });

  it("notes when no brief exists yet", () => {
    const prompt = buildSystemPrompt({ brief: null, recentDecisions: [] });
    expect(prompt).toMatch(/No brief yet/);
  });

  it("inserts decision summaries with their short ids", () => {
    const prompt = buildSystemPrompt({
      brief: null,
      recentDecisions: [
        {
          id: "abcdef12-3456-7890-abcd-ef1234567890",
          timestamp: "2026-05-02T00:00:00.000Z",
          summary: "Postmark for transactional",
          rationale: "Lower bounce rate vs Sendgrid",
        },
      ],
    });
    expect(prompt).toMatch(/abcdef12/);
    expect(prompt).toMatch(/Postmark for transactional/);
    expect(prompt).toMatch(/Lower bounce rate/);
  });

  it("token budget — prompt body stays under 4 KB (≤ ~1 K tokens including brief/decisions)", () => {
    // The pillars block alone is ~700 tokens; the brief + decisions blocks
    // are dynamic but capped at "20 most recent" — this guards against
    // someone bloating the static portion.
    const bytes = Buffer.byteLength(SYSTEM_PROMPT_PILLARS, "utf8");
    expect(bytes).toBeLessThan(4000);
  });
});
