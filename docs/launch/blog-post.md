# Introducing Voce IR: AI-Native UI Without Human-Readable Code

*Draft for voce-ir.xyz/blog. ~1900 words. Tone: technical, specific,
principled — no hype, no jargon-without-payoff. Edit liberally before
publishing.*

---

Most AI coding tools share an assumption so obvious it disappears: the
output the AI produces is **source code**, and a human will eventually
read it, edit it, blame-merge it, and keep it alive. The whole
toolchain — IDE, language server, version control, code review —
exists to make code legible.

Voce IR starts from the other assumption. The AI produces a typed
binary intermediate representation. A compiler turns it into the
artifact the end-user touches — HTML, native UI, an email. **There is
no human-readable code in the pipeline.** And there is no plan to add
one.

This isn't a stylistic preference. It changes what AI authoring can
guarantee, what an agent contract actually means, and which classes of
bugs simply cannot exist. Today, after about a year of building, Voce
IR is at v1.0.0+ with 15 Rust crates, 4 TypeScript packages, 391
tests, 7 compile targets, and a 6-envelope agent contract. We're
preparing v1.1.0 and an open-source launch. This post is the case for
why the source-text-free path is worth the bet.

## The three non-negotiable pillars

Voce sits on three principles that override any other consideration:

**Stability.** Security is a compile error, not configuration. CSRF is
required on mutating actions. Auth-required routes need redirects.
HTTPS is enforced. JSON-LD that breaks out of script context is
rejected. The compiled output has *zero* runtime dependencies — no
framework, no router, no client-side state library — so the supply
chain attack surface for what ships to your users is, literally, zero.

**Experience.** Spring physics solved at compile time via an ODE
solver to CSS `linear()`. Per-script SHA-256 hashes for a hardened
CSP. Every byte in the output serves the user. The reference DOM
target produces a single HTML file that loads in milliseconds.

**Accessibility.** Missing a `SemanticNode` on an interactive element
is not a warning — it's a validation **error**, and the build stops.
Heading hierarchy enforced. `ReducedMotion` mandatory on every
animation. Color contrast computed at compile time against WCAG 2.2
AA (no JS audit, no runtime cost). Touch targets checked against SC
2.5.8's 24×24 floor.

These aren't claims; they're tests. The validator ships nine passes
covering 52 stable diagnostic codes — STR (structural), REF
(references), STA (state machines), A11Y, SEC, SEO, FRM (forms),
I18N, MOT (motion) — each carrying a hint, a fix-confidence rating,
and (for 17 codes) a JSON Patch auto-fix.

## "Accessibility is a compile error" — what that actually means

Consider a `Surface` with an `href` (a link or button) whose only
child is an icon — say, a gear glyph. In a typical AI-generated React
app, the assistant might forget the `aria-label`. The icon ships. A
screen-reader user hears "link" and nothing else.

In Voce, that same shape produces an `A11Y006` validation error:
"Link or button has no accessible text content." The build refuses to
complete. The hint names the specific fix. The compiler, separately,
auto-synthesizes an `aria-label` from a descendant MediaNode's alt
when one is present — so an icon link with `<img alt="Settings">`
inside a Voce `Surface` href ends up emitting:

```html
<a href="/settings" class="voce-btn" aria-label="Settings">
  <img src="/icons/gear.svg" alt="Settings">
</a>
```

with the `aria-label` promoted explicitly, not left to per-client
img-alt fallback heuristics.

This is what "pillar enforcement" looks like in practice: an
implementation detail that's invisible in most AI-generated code, made
*structurally* impossible to ship wrong.

## Why no source code? (Three reasons.)

**1. The agent contract becomes the entire interface — and that
forces it to be complete.**

When an AI agent works on a Rust or TypeScript project and hits a
wall, it can fall back to reading the source. That escape hatch is
silently load-bearing: every "AI-native" tool relies on it to fill
gaps in its own contract. Voce can't. There's no source text to read.

That sounds limiting until you realize what it forces: the contract
has to actually be complete. So we made it one. Six versioned,
schema-locked, machine-consumable envelopes:

- **`voce skills --json`** — what this build can do (passes, codes,
  node types, targets, CLI surface), all reflected from single
  sources of truth, never hand-maintained.
- **`voce graph <file> --json`** — the IR's semantic graph:
  composition, typed reference edges with resolved/dangling status,
  state-machine reachability via BFS.
- **`voce doctor --json`** — toolchain + `.voce/` project health
  with stable `DOC-*` check IDs.
- **`voce fix <file> --plan`** — a convergent multi-step repair
  plan (validate → apply → re-validate → repeat) with explicit
  `converges` and `residual_codes` so an agent can drive auto-repair
  headlessly.
- **`voce compile --perf-report`** — per-phase compile timing.
- **`voce conformance run --target dom --json`** — cross-target
  semantic equivalence at Core / Standard / Full level.

Every envelope carries a `contract_version`. Every schema lives under
`docs/schema/contract/v1/`. Every schema is *derived from the live
struct* via `schemars` and **gated in CI**: change the struct, the
schema regenerates, the diff lands in code review, the version bumps
by policy. The committed schema can't silently drift from the live
shape because the test won't let it.

And there's a contract-completeness test — seven representative
agent-task scenarios, plus a corpus-walking drift gate that asserts
**every diagnostic code emitted at runtime is declared in
`skills.diagnostic_codes`**. A new code added to a pass that forgets
its `CodeMeta` entry fails CI loudly. The agent's view of the world
is never out of sync with the validator's reality.

**2. One IR, every platform — without per-platform style drift.**

Voce ships seven compile targets: DOM (single-file HTML), Hybrid (DOM
+ WebAssembly), WebGPU, native iOS via SwiftUI, native Android via
Jetpack Compose, Email HTML (tables, Outlook-safe), and WASM (state
machines as WebAssembly text). One IR document compiles to all of
them.

This already exists for AI-generated source code in some form — point
GPT-5 at "make me an iOS app" and it'll produce SwiftUI. But every
provider has different output preferences, and a multi-target project
becomes a stylistic Tower of Babel.

In Voce, the IR is a single JSON shape. Every provider emits the same
artifact. The compile targets are the ones that diverge — and they do
so under a **typed conformance contract**. Email legitimately can't
do JS gestures? That's declared, tested, and surfaced honestly as
"Pass at Standard level, Gestures unavailable by medium." WebGPU
paints on the GPU behind an HTML shell? That's `NotApplicable` for
HTML-scraped semantics — not a fake-pass, not a buried failure.

We built a `voce conformance run --target X --level Y` command that
any third-party compiler can be certified against. The cross-target
parity test for the in-tree compilers runs on every PR.

**3. Conversational, not consultative.**

The AI bridge implements a five-phase generation workflow: discovery
→ design → propose → refine → finalize. Each phase has a quality
gate. The discovery agent asks one question at a time. The readiness
score must be ≥ 70 before generation runs. The generated IR has to
pass the validator before it can be finalized. There's a project
brief checked into `.voce/brief.md`, a decision log in append-only
`.voce/decisions.jsonl`, and drift detection that surfaces conflicts
between an incoming proposal and prior decisions.

This is the part of Voce that's hardest to describe in a screenshot.
"Anti-vibe-coding" is the working name for it, and it's enforced by
the pipeline, not the assistant's good intentions. No TODOs.
No half-implementations. No silent placeholders. The assistant can't
declare the work done — the validator does.

## How to actually use it

Two surfaces, both shipped today:

### Standalone REPL: `voce-chat`

```bash
npm install -g @voce-ir/cli-chat
voce-chat
```

A self-contained conversational terminal — sister to running `claude`
or `gemini`. Anthropic SDK tool-use loop, persistent `.voce/` memory,
18 slash commands, multi-line input, ~78 ms cold start.

### MCP for your existing client

```bash
npm install -g @voce-ir/mcp-server
```

```jsonc
// Claude Code config
{ "mcpServers": { "voce-ir": { "command": "voce-mcp-server" } } }
```

Plug Voce into Claude Code, Cursor, Continue, or any
[Model Context Protocol](https://modelcontextprotocol.io) client.
The same 22 tools become first-class for your assistant — validate,
compile, inspect, generate, brief, decisions, drift, the full
workflow, skills, graph, doctor.

Both share the same store, the same gates, the same agent contract.
Pick whichever fits your flow.

### Or the bare CLI

```bash
cargo install voce-validator
voce validate page.voce.json       # 9 passes, 52 rules
voce compile page.voce.json        # zero-runtime HTML output
voce fix page.voce.json --until-clean --plan  # convergent auto-repair plan
voce conformance run --target dom  # certify a target against the corpus
voce deploy page.voce.json --adapter cloudflare
```

## What we didn't build (and why)

We didn't build a language. We didn't build a runtime. We didn't
build a hosted "AI app studio." We deliberately ship a typed IR, a
compiler that emits to existing platforms, and the agent surface
between them.

A few things we explicitly *won't* do:

- **Train a model.** Voce is provider-agnostic. The IR is a typed
  JSON contract every provider can emit. Two sprints in the roadmap
  (S92/S93, prompted by [Kilo Code's per-mode model selection](https://kilo.ai/))
  will let users assign different models to different roles —
  Discovery on a reasoning-heavy model, Generator on a fast one,
  Repair on a cheap one — with pillar enforcement that survives any
  provider choice.
- **Pretend to verify what we can't.** The WebGPU compile target
  paints UI on the GPU behind an HTML shell. The conformance kit
  classifies it `NotApplicable` for HTML-scraped semantic parity,
  with the explicit note "requires an out-of-HTML-lens extractor."
  Honest classification beats a green checkmark that means nothing.
- **Make the binary IR human-readable.** It's FlatBuffers. There's
  a JSON canonical form for AI generation, debugging, and version
  control diffing — but the production artifact is binary by design.

## Where we are, where we're going

v1.0.0 shipped with the schema, the validator, the DOM compiler, the
AI bridge, the inspector, six other compile targets, and the
deployment adapters. The current phase (production readiness) added
the agent contract, the conformance kit, hardened security, perf
budgets, and the accessibility deep-dive that turned a pillar into a
proven invariant.

v1.1.0 readiness is what this launch covers. After that: pluggable
providers per role (S92), the typed model recommender (S93), and the
conformance kit's full spec + signed-attestation badge program (the
remainder of S91). The roadmap is in
[`docs/plans/MASTER_PLAN.md`](https://github.com/marc-pelland/voce-ir/blob/main/docs/plans/MASTER_PLAN.md).

If you build with AI today and you've noticed that the assistant is
better at writing code than your codebase is at telling the assistant
the truth — Voce is for you. The agent contract is the codebase.

The code is gone. The experience remains.

---

**Repo:** [github.com/marc-pelland/voce-ir](https://github.com/marc-pelland/voce-ir)
**Site & playground:** [voce-ir.xyz](https://voce-ir.xyz)
**Try it in 60 seconds:** `npm install -g @voce-ir/cli-chat && voce-chat`
