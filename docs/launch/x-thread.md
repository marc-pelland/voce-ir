# X / Twitter Launch Thread

*10 posts. Each ≤ 280 chars. Standalone hooks, threaded narrative.
Post screenshots / short clips at the marked positions.*

---

## 1/10 — the hook

Most AI coding tools assume the AI's output is source code a human
will eventually maintain.

Voce IR starts from the other assumption: AI emits typed binary IR, a
compiler emits the artifact, **and there is no human-readable code in
the pipeline.**

🧵

> [281 chars — trim "the artifact" → "the output" if needed]
> [no media; the thesis is the hook]

## 2/10 — what that buys you

"Accessibility is a compile error" is a literal claim, not a slogan.

Missing `SemanticNode` on an interactive node? Build stops.
WCAG 2.2 AA contrast computed at compile time. Heading hierarchy
enforced. ReducedMotion mandatory.

391 tests enforce 3 pillars.

> [media: screenshot of `voce validate` rejecting a missing-semantic
> IR with the A11Y001 hint + docs URL]

## 3/10 — the compiled output

Compiles to a single HTML file. Zero runtime dependencies. The supply
chain attack surface for what ships to your users is, literally, zero.

Per-script SHA-256 CSP. Spring physics solved at compile time to CSS
`linear()`. Every byte serves the user.

> [media: a 7.6KB compiled landing page running in browser dev tools,
> showing zero JS deps + Lighthouse 100/100/100/100]

## 4/10 — one IR, seven targets

Same IR document → DOM, Hybrid (DOM+WASM), WebGPU, iOS SwiftUI,
Android Compose, Email HTML, WASM state machines.

No per-platform style drift. Conformance kit verifies semantic
parity across targets. Email forms degrade by medium — and we say
so honestly.

> [media: side-by-side screenshot, same fixture rendered in DOM
> preview / SwiftUI preview / Email client]

## 5/10 — the agent contract

No source text → the agent contract IS the interface. So it has to be
complete.

6 schema-locked envelopes:
• `voce skills` — capabilities
• `voce graph` — semantic IR graph
• `voce doctor` — health
• `voce fix --plan` — repair plan
• `voce conformance run`
• `voce compile --perf-report`

> [media: `voce skills --json` output, pretty-printed]

## 6/10 — drift-gated in CI

The schemas are derived from the live structs via `schemars`. Change
a struct, the schema regenerates, the diff lands in code review, the
version bumps by policy.

The committed schema can't silently drift from live shape. Six
contract versions, six drift gates.

> [media: screenshot of a CI run where the drift gate caught a
> change, with the diff inline]

## 7/10 — convergent auto-repair

`voce fix <file> --until-clean --plan` runs a multi-step fix loop:

  validate → apply → re-validate → repeat

Until clean, or non-progress detected (the IR doesn't change after a
patch). Output is a contract-versioned plan an agent can drive
headlessly.

> [media: JSON output of a fix-plan with 3 steps + converges: true]

## 8/10 — anti-vibe-coding

5-phase generation workflow with quality gates:

discovery → design → propose → refine → finalize

Discovery agent asks one question at a time. Readiness ≥ 70 before
generation. Validator must pass before finalize. No TODOs. No
half-implementations. The validator decides done, not the assistant.

> [media: voce-chat REPL showing the readiness score rising as the
> assistant gathers context, then unlocking propose]

## 9/10 — two surfaces, your choice

Standalone REPL:
  `npm install -g @voce-ir/cli-chat && voce-chat`

Or plug into Claude Code / Cursor / any MCP client:
  `npm install -g @voce-ir/mcp-server`

22 MCP tools. Same store, same gates, same contract.

> [media: terminal showing voce-chat alongside Claude Code with the
> Voce MCP server connected, listing tools]

## 10/10 — try it

v1.0.0+ today. 15 Rust crates, 4 TS packages, 7 compile targets, 52
validation rules.

Repo: github.com/marc-pelland/voce-ir
Site & playground: voce-ir.xyz
Blog: voce-ir.xyz/blog/introducing-voce-ir

The code is gone. The experience remains.

> [media: the project logo / a clean banner if one exists]

---

## Notes for the launch operator

- **Spacing:** 60–90 sec between posts. Don't auto-schedule the
  whole thread instantly — gives early replies room to surface.
- **Pin tweet 1.** Quote-tweet your own pinned tweet later in the
  week with a single follow-up screenshot to extend the curve.
- **Media absolutely matters on X.** A thread without media gets
  ~30% the reach of one with even mediocre screenshots. The marked
  positions are non-optional.
- **Don't tag big accounts in the original thread.** Reply-tag in
  the quote-tweet phase if a reaction warrants it.
- **The "anti-vibe-coding" framing is sharper than it sounds.** Some
  audiences read it as a swipe at v0/Cursor/Replit. It's not — it's
  describing a different stance toward AI-authored output. Don't
  shy from the term, but be ready to clarify in replies that it's
  about *the workflow*, not other tools.
