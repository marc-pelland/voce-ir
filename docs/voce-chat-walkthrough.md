# voce-chat — Walkthrough

`voce-chat` is the standalone conversational CLI for Voce IR. It speaks
to the model directly, runs the same 19 tools the MCP server exposes, and
persists every conversation to `.voce/sessions/<id>.jsonl` so a session
can be resumed days later. Sister tool to `voce-mcp` — the difference is
the transport: `voce-mcp` plugs into your IDE, `voce-chat` runs in a
terminal.

## Install

```bash
npm install -g @voce-ir/cli-chat
# or build from this repo:
cd packages/cli-chat && npm install && npm run build
```

`voce-chat` shells out to the `voce` CLI for `compile` / `validate` /
`preview`. Install via:

```bash
cargo install --path packages/validator --locked
```

Set your API key:

```bash
export ANTHROPIC_API_KEY=sk-ant-...
```

Without a key, `voce-chat` still loads slash commands and works on IR
files locally — `compile`, `validate`, `save`, `load`, `preview`, `diff`,
`undo`, `brief`, `decisions` — but the model is silent.

## Run

```bash
voce-chat                          # fresh session
voce-chat "build a contact form"   # fresh session, seeded with a prompt
voce-chat --resume                 # most recent session
voce-chat --resume <session-id>    # specific session
```

## A worked session

```
$ voce-chat

  Voce IR
  The code is gone. The experience remains.

  Model: claude-sonnet-4-20250514
  Type what you want to build, or /help for commands.

you > build me a wholesale order form for coffee roasters

voce What does the form actually need to capture? Roaster name, the
     coffees in question, quantities, delivery date — or are some of
     those out of scope?

you > roaster name, coffee selection (multi-select from a catalog), \
       quantity per item, requested ship date, and contact email

voce Got it. Where do submissions land — your inbox, a CRM, an
     ERP system?

…(several discovery turns)…

  ↳ voce_generation_readiness

Generation Readiness: 85/100

  ✓ All discovery checkpoints satisfied.
  ✗ project brief absent — voce_brief_set when the conversation produces a north star

  ↳ voce_generate_propose

Proceed with proposal? [Y/n/q for question] y

voce Here's the IR…
```

The `↳` indicators show tool calls in dim text so the conversation
stays readable. When the model wants to call `voce_generate_propose`,
the readiness score appears first; you decide whether to proceed.

## Slash commands

| Command | Effect |
|---|---|
| `/help` | List commands |
| `/show` | Print the current IR |
| `/save [file]` | Write IR to disk (default `generated.voce.json`) |
| `/load <file>` | Load IR from disk |
| `/ir <pasted>` | Load IR pasted inline |
| `/compile` | Compile IR to HTML, show first 20 lines + path |
| `/validate` | Run the validator, print per-pass diagnostics |
| `/preview` | Compile + open in your default browser |
| `/diff` | Compare current IR with the previous snapshot |
| `/undo` | Pop the IR-history stack |
| `/brief [md]` | Show or write the project brief |
| `/decisions [n]` | List recent decisions (default 5) |
| `/decision sum \| why` | Append a decision to the log |
| `/explain` | Have the model walk through the current IR |
| `/model [name]` | Print or switch model mid-session |
| `/cost` | Cumulative token usage |
| `/clear` | Reset in-memory conversation (session log persists) |
| `/exit` (`/quit`) | Exit |

## Multi-line input

End a line with `\` to continue on the next:

```
you > A list view with empty state, error state, and \
    > a loading skeleton. Filter by date and category, \
    > with a multi-select checkbox group.
```

The continuation prompt switches to `    > ` so you know more input is
expected. A line without trailing `\` submits the whole block.

## Ctrl+C behavior

| State | Effect |
|---|---|
| Multi-line in progress | Drop the pending lines, re-prompt |
| Model request in flight | Abort the request, return to prompt without crashing |
| Idle prompt | Exit cleanly |

You never lose the session log — it's been written incrementally on each
turn.

## Drift push-back

When the model calls `voce_check_drift` and a prior decision's terms
appear in the proposed IR, you'll see:

```
Hold on — decision [abcdef12] from 2026-04-12 may conflict with this IR.
  "no Modal nodes"
  matched terms: modal, trap

  v1 keyword heuristic — terms from this decision appear in the proposed IR.
  Read the decision and judge whether the conflict is real. False positives expected.

[r] revise IR  [s] supersede decision  [c] continue anyway:
```

Three-key resolution:

- **r** — the model reverts and tries again
- **s** — the prior decision is superseded; a new decision is logged with `supersedes` set
- **c** — you knowingly override; a new decision is logged with `conflicts_with` set

Both `s` and `c` write to `.voce/decisions.jsonl` so the audit trail is
complete.

## Memory — what gets persisted

```
.voce/
  brief.md              Authored via /brief or voce_brief_set
  decisions.jsonl       /decision and drift-resolution entries
  drift-warnings.jsonl  voce_check_drift output (when used by future sweeps)
  sessions/<id>.jsonl   Every turn — user, assistant, tool, system events
```

The session JSONL captures every tool call the model made:

```jsonl
{"timestamp":"2026-05-02T15:00:00Z","role":"user","content":"build a form"}
{"timestamp":"2026-05-02T15:00:01Z","role":"tool","tool":"voce_generate_start","content":"{\"input\":{...},\"result\":\"{...}\"}"}
{"timestamp":"2026-05-02T15:00:05Z","role":"assistant","content":"What fields..."}
```

`/explain` works because the model can read this ledger via
`voce_session_resume` and walk back through its own decisions.

## Performance

Cold start (banner ready, session resolved, system prompt assembled):

```
$ npm run bench:startup --silent
voce-chat cold start (5 runs):
  run 1: 77ms
  run 2: 88ms
  run 3: 76ms
  run 4: 76ms
  run 5: 75ms
  min 75ms · avg 78ms · max 88ms
  threshold: 500ms

OK: under 500ms.
```

The S66 acceptance criterion is 500ms on a developer laptop. CI gates
at 1000ms to absorb transient runner load.

## What this CLI intentionally does NOT do

- **No telemetry.** Nothing phones home. The only network traffic is to
  `api.anthropic.com` for the model — and only when you've set
  `ANTHROPIC_API_KEY`.
- **No auto-flip on `ready`.** The readiness score is advisory; the model
  decides when to call `voce_generate_propose`, you decide whether to
  proceed via [Y/n/q].
- **No automatic decision logging.** A decision lands in
  `.voce/decisions.jsonl` only when (a) the model called
  `voce_decisions_log`, (b) you typed `/decision`, or (c) you resolved a
  drift with `s` or `c`. Everything else is just conversation.

## Troubleshooting

**"Set ANTHROPIC_API_KEY to chat"** — you ran `voce-chat` without a key.
Slash commands still work; only the model is silent.

**Tools listed but `voce_compile` / `voce_validate` fail** — the `voce`
CLI isn't on PATH. Install via `cargo install --path packages/validator
--locked` or set `VOCE_BIN` to the binary path.

**Sessions not persisting between restarts** — check that `voce-chat` is
launched from your project root (so `process.cwd()` points at the
project with the `.voce/` directory). Override with the
`VOCE_PROJECT_ROOT` environment variable.

**Slow startup** — run `npm run bench:startup`. If you see numbers
significantly above 100ms on a modern laptop, the bundle has likely
picked up a heavy import; profile with `node --inspect-brk dist/index.js`.
