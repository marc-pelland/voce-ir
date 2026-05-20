# Show HN: Voce IR — AI-Native UI Without Human-Readable Code

*Draft for Hacker News Show HN. Tone: technical, specific, low-hype.
HN's audience hates puff — lead with the architectural bet, not the
adjectives. Edit before posting.*

---

## Title

**Show HN: Voce IR – AI-native UI without human-readable code**

(70 chars; HN soft limit is 80. Avoid "introducing," avoid "the
future of," avoid emoji.)

## Body

Most AI coding tools assume the output is source code a human will
eventually read, edit, and maintain. Voce IR starts from the other
assumption: the AI produces a typed binary IR (FlatBuffers), a
compiler emits the final artifact (DOM/HTML, SwiftUI, Compose, Email,
WebGPU, WASM), and there is no human-readable code in the pipeline.

That single decision lets the project commit to things most
AI-tooling stacks can't:

- "Accessibility is a compile error" is a literal claim, not a
  slogan: missing `SemanticNode` on an interactive node = build
  stops. WCAG 2.2 AA contrast computed at compile time. 391 tests
  enforce the three pillars (stability / experience / accessibility).
- The compiled output has zero runtime dependencies. The supply
  chain attack surface for what ships to users is, literally, zero.
- One IR compiles to seven targets, with a cross-target conformance
  kit (`voce conformance run --target X --level core|standard|full`)
  any third-party compiler can certify against.
- Because there's no source text, the **agent contract is the only
  interface** — and we made it complete. Six versioned, schema-locked
  JSON envelopes (skills, graph, doctor, fix-plan, perf-report,
  conformance). Schemas derived from the live structs via `schemars`,
  drift-gated in CI. A contract-completeness test asserts every
  diagnostic code emitted at runtime is declared in the manifest;
  forget a `CodeMeta` entry, CI fails.

There's a standalone REPL (`voce-chat`, Anthropic SDK tool-use loop,
`~78 ms` cold start) and an MCP server (`@voce-ir/mcp-server`, 22
tools) that plugs into Claude Code / Cursor / any MCP client. Both
share the same store and the same workflow gates — a five-phase
discovery→design→propose→refine→finalize pipeline with a
quality-gate readiness score, append-only decision log, and drift
detection.

v1.0.0+ today: 15 Rust crates, 4 TS packages, 7 compile targets, 52
validation rules (17 auto-fixable via JSON Patch). Prepping v1.1.0.

Roadmap, design docs, and the full agent contract are in the repo.
Honestly curious what the HN crowd thinks of the no-source-text bet —
particularly whether the "agent contract is provably complete" line
holds up under scrutiny.

Repo: https://github.com/marc-pelland/voce-ir
Site: https://voce-ir.xyz
Blog: https://voce-ir.xyz/blog/introducing-voce-ir
Try in 60 seconds: `npm install -g @voce-ir/cli-chat && voce-chat`

## Notes for the launch operator

- **Best post window:** Tuesday–Thursday, 6:30–8:30 AM Pacific.
  HN's algorithm rewards a fast initial vote velocity from a
  pre-warmed audience; weekends are quieter.
- **Don't title-cap. Don't emoji.** "Show HN: voce-ir – ..." is fine;
  "Show HN: 🚀 Introducing Voce IR!" gets flagged.
- **Be present in the comments for the first 90 minutes.** HN
  expects the author to answer. Plan responses for the predictable
  pushback:
  - *"This is just a DSL with extra steps."* — fair framing question;
    the answer is the agent-contract completeness guarantee that a
    text DSL with a fallback-to-source escape hatch structurally
    can't match. Cite the B4 contract-completeness test concretely.
  - *"How is this different from v0?"* — v0 generates React source
    you maintain; Voce generates IR that compiles to seven targets
    with no source for anyone to maintain. Different layer.
  - *"How is this different from vercel-labs/zero?"* — Zero is a
    general-purpose programming language for agents (still text
    source); Voce is a UI IR (no source). Different problem, both
    valid, see the blog post's comparison.
  - *"Vendor lock-in to Anthropic?"* — provider-agnostic by design;
    S92 (in the roadmap) lets users assign any provider per role.
    Today's default uses Claude because the discovery agent
    benefits from strong reasoning; everything else is pluggable.
- **Don't link the X thread from the HN post.** Cross-promotion
  reads as marketing. Let the threads live separately.
