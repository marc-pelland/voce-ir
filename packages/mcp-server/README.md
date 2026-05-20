# @voce-ir/mcp-server

Voce IR Model Context Protocol server. Exposes 22 tools and 4
resources for working with Voce IR through any MCP client —
Claude Code, Cursor, Continue, your own agent.

```bash
npm install -g @voce-ir/mcp-server
```

```jsonc
// Claude Code: ~/.claude/claude_desktop_config.json
{
  "mcpServers": {
    "voce-ir": { "command": "voce-mcp-server" }
  }
}
```

The binary is published under two names — **`voce-mcp-server`** (preferred,
self-describing) and `voce-mcp` (legacy alias). Both invoke the same
entry point.

## What it exposes

22 tools covering the full pipeline:

- **Pipeline:** `voce_validate`, `voce_compile`, `voce_inspect`, `voce_generate`
- **Agent contract** (S79): `voce_skills`, `voce_graph`, `voce_doctor`
- **Memory** (`.voce/` store): `voce_brief_get/set`, `voce_decisions_log/list`,
  `voce_check_drift`, `voce_session_resume`
- **Generation workflow:** `voce_generate_start/answer/propose/refine/finalize`,
  `voce_generation_readiness`, `voce_feature_completeness`
- **Reference:** `voce_schema`, `voce_examples`

Plus 4 resources: `voce://brief`, `voce://decisions`,
`voce://drift-warnings`, `voce://status`.

## Why this exists

Voce IR is an AI-native UI intermediate representation — the AI emits
typed binary IR, a compiler emits the artifact, and there is no
human-readable code in the pipeline. The MCP server is one of two
ways to drive that pipeline conversationally; its sister
[`@voce-ir/cli-chat`](https://npmjs.com/package/@voce-ir/cli-chat) is
the standalone REPL.

Both share the same `.voce/` memory store and the same agent contract.

## Full project

See [github.com/marc-pelland/voce-ir](https://github.com/marc-pelland/voce-ir)
for the full project — schema, validator, compiler, AI bridge, docs.
Site: [voce-ir.xyz](https://voce-ir.xyz).

## License

Apache-2.0. See [LICENSE](./LICENSE).
