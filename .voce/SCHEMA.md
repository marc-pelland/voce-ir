# `.voce/` Memory Contract

This file pins the on-disk layout and entry shapes for the Voce memory store.
Changes here are breaking. Day 2 of S65 introduced this; subsequent days build
on it.

## Layout

```
.voce/
  brief.md              Project north star — markdown, hand-edited or set via voce_brief_set.
  decisions.jsonl       Append-only log. One Decision per line.
  drift-warnings.jsonl  Append-only log. One DriftWarning per line.
  user-profile.md       Optional. User preferences and recurring patterns.
  sessions/             Per-session conversation logs. <session-id>.jsonl, append-only.
                        Gitignored — these are working state, not durable record.
  SCHEMA.md             This file.
```

## Invariants

1. **Append-only.** `decisions.jsonl`, `drift-warnings.jsonl`, and every
   `sessions/*.jsonl` are written only by `appendJsonlLine`. No code path
   rewrites or truncates them. Supersession is expressed via a new entry
   with `supersedes: <prior-id>`, never by mutating the prior entry.
2. **Atomic full-file writes.** `brief.md` and `user-profile.md` are
   replaced via `atomicWriteFile` (write-tmp + fsync + rename). A reader
   sees either the prior version or the new version, never a torn write.
3. **One JSON per line.** `JSON.stringify` never emits unescaped newlines,
   so a `\n` in a JSONL file always terminates an entry. The append helper
   guards this explicitly.
4. **Validation on read.** `readJsonlFile` validates each entry against
   the type predicate. Invalid lines are returned in `errors[]`, not
   silently dropped, and never crash the read.

## Entry shapes

### Decision (`decisions.jsonl`)

```ts
{
  id: string;              // UUID v4
  timestamp: string;       // ISO-8601 (matches new Date().toISOString())
  summary: string;         // Non-empty
  rationale: string;       // Non-empty
  supersedes?: string;     // id of a prior decision this replaces
  conflicts_with?: string; // id of a decision this knowingly overrides
}
```

### DriftWarning (`drift-warnings.jsonl`)

```ts
{
  timestamp: string;          // ISO-8601
  decision_id: string;        // References Decision.id
  drift_description: string;  // Non-empty, human-readable
  resolution: "pending" | "accepted" | "overridden";
}
```

### SessionEntry (`sessions/<id>.jsonl`)

```ts
{
  timestamp: string;                                      // ISO-8601
  role: "user" | "assistant" | "system" | "tool";
  content: string;
  tool?: string;                                          // When role === "tool"
}
```

## Legacy

Pre-S65 scaffolding created `.voce/decisions/`, `.voce/memory/`, and
`.voce/snapshots/` as empty directories alongside `.voce/brief.yaml.template`.
None of those held real data. The S65 Day 2 reset chose `brief.md` (markdown,
human-friendly) over `brief.yaml` (structured, machine-friendly), and a single
`decisions.jsonl` over a directory of YAML files. The legacy directories are
kept on disk but unused — they can be removed in a future cleanup once we are
confident no external tooling references them.

## Path resolution

`PATHS` in `packages/mcp-server/src/memory/store.ts` is the single source of
truth for filenames. The base directory is `process.env.VOCE_PROJECT_ROOT` if
set, otherwise `process.cwd()`. Tests override the env var to point at a
temp directory; the MCP server inherits cwd from its host (the project root
the agent is operating against).
