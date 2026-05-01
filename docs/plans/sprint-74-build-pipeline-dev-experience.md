# Sprint 74 — Build Pipeline + Dev Experience

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Make local Voce development a first-class experience. Today the workflow is: edit `.voce.json` → run `voce compile` → manually open the HTML → repeat. This sprint adds `voce dev` (file-watch + live-reload preview), structured error overlays in the preview, an editor command palette via the MCP server, and proper integration with the most common IDEs. Friction goes from "scripted" to "hot."

**Depends on:** S65 (MCP polish — for IDE integration), S67 (validator diagnostics — for error overlays). Independent of S68–S72.

---

## Motivation

The CLI subcommands today (`validate`, `compile`, `inspect`, `preview`, `deploy`) are correct but disconnected. There's no single "I'm working on this IR, watch it for me" command. Every change is a manual recompile-and-refresh. Compare to `vite dev` or `next dev` which auto-rebuild and inject HMR. The bar isn't to clone Vite — it's to make the inner-loop comfortable enough that a developer would happily build a real site with this.

---

## Deliverables

### 1. `voce dev` command

```
voce dev [<path>] [--port 5173] [--open] [--target dom|email]
```

Behavior:
- Watches the IR file (default: `./index.voce.json`) and any referenced asset paths
- On change: re-validates, re-compiles, pushes the new HTML over a server-sent events (SSE) channel
- The browser uses a tiny inline reload script (~200 bytes) injected only in dev mode
- Validation errors don't crash; they render as a structured overlay on top of the last good output
- Compile errors render as a prominent red banner + the validation diagnostics
- Preserves browser scroll position across reloads

Implementation: small `tokio::watch` loop with `notify` crate for filesystem events. SSE over a hyper or axum server (workspace already pulls warp-style deps via adapter-cloudflare; reuse). The reload script subscribes to `/__voce_dev_events` and triggers `location.reload()` on ping.

### 2. Validation error overlay

When validation fails, the previewed page gets a non-blocking overlay:

```
╔═════════════════════════════════════════════════════════╗
║ 2 errors · 1 warning                       [retry] [×] ║
╠═════════════════════════════════════════════════════════╣
║ ✗ STR002 at /root/children/2                            ║
║   NONE node is missing a node_id                        ║
║   → Add "node_id": "<unique-name>" to the node          ║
║   [apply auto-fix]                                       ║
║                                                          ║
║ ✗ A11Y001 at /root/children/3/value                     ║
║   Interactive Container has no SemanticNode              ║
║   → Add a SemanticNode with role="button" and reference ║
║     it via semantic_node_id                              ║
║                                                          ║
║ ⚠ SEO007 at /root/metadata                              ║
║   OpenGraph data is present but missing og:image         ║
╚═════════════════════════════════════════════════════════╝
```

- "apply auto-fix" calls back to `voce dev`, which uses S67's `voce fix --apply` on the offending range
- Pulls hint and patch info from S67's enhanced diagnostic output
- Overlay dismissible; hidden by default if no diagnostics

### 3. Compile-error overlay

When compilation itself fails (validator green, compiler exception), the overlay is full-screen with:
- Stack trace (compiler's, not browser's)
- The IR snippet around the offending node
- A "report bug" link with prefilled GitHub issue body

### 4. VS Code extension

A new `tools/vscode-voce/` directory containing:
- Syntax highlighting for `.voce.json` (using JSON schema with custom annotations)
- Validation on save (calls the MCP server in the workspace's `.vscode/voce-mcp.config`)
- Hover tooltips for node types (pulls from the schema)
- Code lens above each node showing its compiled size estimate
- Quick-fix actions (CodeLens links to S67's auto-fixes)
- "Run voce dev" command palette entry
- Marketplace publish workflow

Initial scope: schema-driven validation + run-dev command. Defer the full set of features.

### 5. JetBrains plugin (stub only)

A `tools/jetbrains-voce/` skeleton with `plugin.xml`, language definition, and stub validation. Not a full implementation — the marker for community contribution. Documented as "wanted: maintainer."

### 6. CLI ergonomic improvements

- Colored output (already partial via `colored` crate); audit for consistency
- `voce` with no args prints a friendly help screen, not the clap error
- `voce <typo>` suggests the closest valid subcommand
- Shell completions for bash/zsh/fish via `clap_complete` — published as a new CLI subcommand: `voce completions zsh > ~/.zfunc/_voce`
- `voce --version` prints version + git commit + build date (when available)

### 7. Project init: `voce init`

```
voce init [<dir>]
```

Creates a starter project structure:
- `index.voce.json` — minimal valid IR (a hello-world page)
- `.voce/brief.md` — empty brief template
- `.voce/decisions.jsonl` — empty
- `.gitignore` — ignores `dist/`, `.voce/session.jsonl`
- `voce.toml` — project config (default adapter, schema version, etc.)
- A small README with "next: `voce dev`"

Mirrors `npm init` / `cargo init` ergonomics.

### 8. Documentation

- `docs/dev-experience.md` — getting started workflow with `voce init` → `voce dev` → `voce build` → `voce deploy`
- VS Code extension README + screenshots
- A short screencast linked from `voce-ir.xyz` (out of scope to record; placeholder for now)

---

## Acceptance Criteria

- [ ] `voce dev` command runs, watches files, hot-reloads on save
- [ ] Validation errors render as an overlay; auto-fix button works
- [ ] Compile errors render as a full-screen overlay with stack + IR snippet
- [ ] VS Code extension installs, syntax-highlights `.voce.json`, validates on save
- [ ] `voce` with no args shows friendly help, not an error
- [ ] Typo'd subcommand suggests the closest match
- [ ] `voce completions <shell>` works for bash/zsh/fish
- [ ] `voce init` produces a working starter project
- [ ] `docs/dev-experience.md` exists with full workflow
- [ ] No regression in existing CLI commands

---

## Risks

1. **File-watch reliability across platforms.** `notify` is generally good but has rough edges on Windows and over network mounts. Document known limitations.
2. **SSE through corporate proxies can be flaky.** Provide a `--no-sse` fallback that uses polling.
3. **VS Code extension publishing is a process.** Timeline-wise, treat the extension as ship-when-ready, not gating the rest of the sprint.
4. **The dev server is a new long-running process.** Tests for it need to be integration-shape; don't accidentally leak test servers.

---

## Out of Scope

- Preview-server hosted in the browser (no Node, no Rust process needed) — defer
- Full IDE integration for IntelliJ/JetBrains — stub only
- Git-aware "what changed since last commit" diff view in the overlay
- Live collaboration / multiplayer editing
- Server-side rendering for IRs that include DataNode fetches
