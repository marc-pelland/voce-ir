// Tiny argv parser — voce-chat's surface is small, no need for yargs.

export interface CliArgs {
  /** null = fresh session, "auto" = most recent, string = explicit id. */
  resume: string | "auto" | null;
  /** Verbatim initial prompt assembled from the non-flag positional args. */
  initialPrompt: string;
}

export function parseArgs(argv: readonly string[]): CliArgs {
  const args = [...argv];
  let resume: CliArgs["resume"] = null;
  const positional: string[] = [];

  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    if (a === "--resume") {
      const next = args[i + 1];
      if (next !== undefined && !next.startsWith("--")) {
        resume = next;
        i += 1;
      } else {
        resume = "auto";
      }
    } else {
      positional.push(a);
    }
  }

  return { resume, initialPrompt: positional.join(" ") };
}
