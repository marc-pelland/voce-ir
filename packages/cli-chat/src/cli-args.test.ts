import { describe, expect, it } from "vitest";
import { parseArgs } from "./cli-args.js";

describe("parseArgs", () => {
  it("returns null resume + empty prompt when argv is empty", () => {
    expect(parseArgs([])).toEqual({ resume: null, initialPrompt: "" });
  });

  it("collects positional args as the initial prompt", () => {
    expect(parseArgs(["build", "a", "nav", "bar"])).toEqual({
      resume: null,
      initialPrompt: "build a nav bar",
    });
  });

  it("--resume with no argument means 'auto' (most recent)", () => {
    expect(parseArgs(["--resume"])).toEqual({ resume: "auto", initialPrompt: "" });
  });

  it("--resume <id> takes the next non-flag token as the session id", () => {
    expect(parseArgs(["--resume", "abc-123", "make", "it", "blue"])).toEqual({
      resume: "abc-123",
      initialPrompt: "make it blue",
    });
  });

  it("--resume followed by another flag stays in 'auto' mode", () => {
    expect(parseArgs(["--resume", "--something-else"])).toEqual({
      resume: "auto",
      initialPrompt: "--something-else",
    });
  });
});
