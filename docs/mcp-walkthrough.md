# Voce MCP — Walkthrough

The Voce MCP server (`@voce-ir/mcp-server`, binary `voce-mcp`) exposes the
Voce IR pipeline + the `.voce/` memory store + a five-phase generation
workflow to any MCP-compatible client. Once wired up, your client (Claude
Code, Cursor, Cline, Continue.dev, Claude Desktop, …) inherits Voce's
conversational pillars without client-side prompting.

## What you get

**Pipeline tools** — validate, compile, inspect IR; consult schema; pull
reference examples.

**Memory tools** — read and write the project brief; log decisions with
rationale; resume prior sessions; check a proposed IR for drift against
recorded decisions.

**Generation workflow (5 phases)** — `start` → `answer` (×N) → `propose`
→ `refine` → `finalize`. Discovery is mandatory: `propose` blocks if
readiness < 70, `finalize` blocks on missing pillars or validation
errors. The agent cannot skip steps.

**Quality gates as standalone tools** — `voce_generation_readiness`,
`voce_feature_completeness`. Same logic the workflow enforces, callable
directly so you can probe state without mutating it.

**Resources** — `voce://brief`, `voce://decisions`, `voce://drift-warnings`,
`voce://status`. Live reads from the `.voce/` directory.

## Install

```bash
npm install -g @voce-ir/mcp-server
# or, from this repo:
cd packages/mcp-server && npm install && npm run build
```

The server expects the `voce` CLI on `PATH` (it shells out for validate +
compile). Install via:

```bash
cargo install --path packages/validator --locked
```

## Wiring it up

### Claude Code

Edit `~/.claude/settings.json` (or `.claude/settings.json` in your project):

```json
{
  "mcpServers": {
    "voce-ir": {
      "command": "voce-mcp",
      "args": []
    }
  }
}
```

Restart Claude Code. The 19 Voce tools should appear in the next session.
List them with `/mcp` — you should see all `voce_*` tools.

### Cursor

Cursor reads MCP servers from `~/.cursor/mcp.json` (global) or
`.cursor/mcp.json` (per-project):

```json
{
  "mcpServers": {
    "voce-ir": {
      "command": "voce-mcp"
    }
  }
}
```

### Cline

Cline reads MCP servers from VS Code settings. Open Settings → Extensions →
Cline → MCP Servers, or edit `settings.json` directly:

```json
{
  "cline.mcpServers": {
    "voce-ir": {
      "command": "voce-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

### Continue.dev

Add to `~/.continue/config.json`:

```json
{
  "experimental": {
    "modelContextProtocolServers": [
      {
        "transport": {
          "type": "stdio",
          "command": "voce-mcp"
        }
      }
    ]
  }
}
```

### Claude Desktop

Edit `~/Library/Application Support/Claude/claude_desktop_config.json`
(macOS) or the equivalent on your platform:

```json
{
  "mcpServers": {
    "voce-ir": {
      "command": "voce-mcp"
    }
  }
}
```

## Worked example — generating a contact form

A complete session, in the order the calling agent should invoke tools.

### 1. Open a session

```
voce_generate_start({
  user_intent: "I want a contact form for my consulting site"
})
→ { session_id: "8a3c…", state: { phase: "discovering" }, readiness: { score: 20, ready: false } }
```

### 2. Discover (one question at a time)

```
voce_generate_answer({
  session_id: "8a3c…",
  question: "What information do you need to collect?",
  answer: "Name, email, project description, optional budget range",
  ready: false
})
→ { state: { discovery_turns: 1 }, readiness: { score: 40 } }

voce_generate_answer({
  session_id: "8a3c…",
  question: "Where do submissions go — email, CRM, database?",
  answer: "Send to my Postmark inbox plus log to Airtable",
  ready: false
})
→ { state: { discovery_turns: 2 }, readiness: { score: 40 } }

voce_generate_answer({
  session_id: "8a3c…",
  question: "Do you want spam protection? CAPTCHA, honeypot, both?",
  answer: "Honeypot is fine — I don't want CAPTCHA friction",
  ready: false
})
→ { state: { discovery_turns: 3 }, readiness: { score: 60 } }

voce_generate_answer({
  session_id: "8a3c…",
  question: "Anything else — confirmation message style, success URL?",
  answer: "Inline 'thanks, I'll respond within 2 days' message; stay on page",
  ready: true
})
→ { state: { discovery_turns: 4, ready: true }, readiness: { score: 100 } }
```

### 3. Propose

The agent now generates the IR (using its own model context — the server
doesn't call Anthropic) and submits:

```
voce_generate_propose({
  session_id: "8a3c…",
  ir_json: "{\"value_type\":\"ViewRoot\",…}"
})
→ {
    ok: true,
    state: { phase: "proposed", has_proposal: true },
    readiness: { score: 100, ready: true },
    completeness: { complete: true, missing_pillars: [] }
  }
```

If the IR is missing a pillar (no `semantic_node_id`, no `error_state`,
etc.) the response carries `completeness.complete: false` with a list. Fix
with `voce_generate_refine`.

### 4. Refine (optional, can be called repeatedly)

```
voce_generate_refine({
  session_id: "8a3c…",
  feedback: "Move the budget field below the description",
  ir_json: "{\"value_type\":\"ViewRoot\",…(updated)…}"
})
→ { state: { phase: "proposed" }, completeness: { complete: true } }
```

### 5. Finalize

```
voce_generate_finalize({ session_id: "8a3c…" })
→ {
    ok: true,
    state: { phase: "finalized" },
    ir_json: "…",
    validation: { ok: true, …pass details… },
    html: "<!DOCTYPE html>…",
    deployment_hints: [
      "voce-adapter-vercel — git-push deploy with Edge runtime",
      "voce-adapter-netlify — Netlify Functions + CDN",
      "voce-adapter-cloudflare — Workers + Pages",
      "voce-adapter-static — single-file HTML for any static host"
    ]
  }
```

If validation fails OR a pillar is missing, `ok: false` and the response
explains what to do — you can't accidentally finalize broken IR.

## Memory at a glance

Every session writes to `.voce/`:

```
.voce/
  brief.md              # Hand-edited or set via voce_brief_set
  decisions.jsonl       # Append-only, one Decision per line
  drift-warnings.jsonl  # Append-only, one DriftWarning per line
  sessions/<id>.jsonl   # Append-only conversation logs (gitignored)
  SCHEMA.md             # Pinned contract — see this file before extending
```

Use `voce_decisions_log` whenever the conversation produces a durable
choice ("we chose Postmark over Sendgrid because…"). Use `voce_check_drift`
on a proposed IR before finalizing — it surfaces decisions whose terms
appear in the IR so you can judge real conflicts.

`voce_session_resume` returns the most recent `ir_snapshot` from a session,
so a resumed conversation picks up exactly where the prior one left off —
even after restarts or crashes.

## What this server intentionally does NOT do

- **It does not call Anthropic.** The calling agent has the model context;
  generation lives there. The MCP server orchestrates state, gates
  readiness/completeness, and runs the deterministic Voce pipeline. No
  `ANTHROPIC_API_KEY` required.
- **It does not auto-flip `ready`.** Discovery completion is the agent's
  judgment call. The readiness score is advisory; `ready: true` is the
  agent's commitment.
- **Drift detection v1 is a heuristic, not a guarantee.** It substring-matches
  decision terms against IR text — false positives expected, semantic
  conflicts that share no surface vocabulary slip through. Use it as an
  aid, not a gate. v2 will ship a real rule engine.

## Troubleshooting

**`voce-mcp` not found** — `npm install -g @voce-ir/mcp-server`, or invoke
the binary directly: `node /path/to/packages/mcp-server/dist/index.js`.

**Tools listed but calls fail with `voce: command not found`** — the
server shells out to the `voce` CLI for validate + compile. Install via
`cargo install --path packages/validator --locked`.

**`voce_generate_propose` blocks with score 40** — that's working as
intended. Continue discovery with more `voce_generate_answer` calls and
pass `ready: true` on the last one.

**Sessions not persisting between restarts** — check that the MCP server is
launched from your project root (so `process.cwd()` points at the project
with the `.voce/` directory). Override with the `VOCE_PROJECT_ROOT`
environment variable in the client config.
