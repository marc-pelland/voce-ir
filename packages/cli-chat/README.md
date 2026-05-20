# @voce-ir/cli-chat

`voce-chat` — a standalone conversational terminal for Voce IR.
Sister to running `claude` or `gemini`, dedicated to authoring,
validating, and compiling Voce IR documents.

```bash
npm install -g @voce-ir/cli-chat
export ANTHROPIC_API_KEY=…
voce-chat
```

## What it does

- **Tool-use loop** driven by the Anthropic SDK, with the same 22
  tools exposed by [`@voce-ir/mcp-server`](https://npmjs.com/package/@voce-ir/mcp-server)
- **Persistent `.voce/` memory** — project brief, append-only
  decision log, drift detection, per-session JSONL transcripts
- **18 slash commands** for inspection, branching, history,
  context management
- **Multi-line input** with explicit submit semantics
- **Readiness + drift UX** — the assistant can't propose IR until
  the 5-phase discovery → design → propose → refine → finalize
  workflow's readiness score is ≥ 70
- **Prompt caching** for cheap re-runs; **~78 ms cold start**

## Why this exists

Two surfaces for the same agent contract: this one runs standalone;
the MCP server plugs into an existing client. Both share the same
`.voce/` store, the same quality gates, the same six contract
envelopes (skills, graph, doctor, fix-plan, perf-report,
conformance).

## Full project

See [github.com/marc-pelland/voce-ir](https://github.com/marc-pelland/voce-ir).
Site: [voce-ir.xyz](https://voce-ir.xyz).

## License

Apache-2.0. See [LICENSE](./LICENSE).
