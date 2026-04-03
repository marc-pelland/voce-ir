# Sprint 52 — Deployment Adapters (Vercel, Cloudflare, Netlify)

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Implement real deployment adapter crates that produce target-specific output bundles and a `voce deploy` command for interactive deployment. Move from conceptual adapters to working integrations that ship sites.

**Depends on:** Sprint 51 (image pipeline produces real asset files)

---

## Deliverables

- New crate: `packages/adapter-static/` — copies compiled output to `dist/`, generates correct relative paths, produces a ZIP for manual upload
- New crate: `packages/adapter-vercel/` — generates `vercel.json`, converts `ActionNode` handlers to serverless functions in `api/`, outputs `.vercel/output/` directory structure
- New crate: `packages/adapter-cloudflare/` — generates `wrangler.toml`, converts `ActionNode` handlers to Cloudflare Workers scripts, outputs `dist/` with `_worker.js`
- New crate: `packages/adapter-netlify/` — generates `netlify.toml`, converts `ActionNode` handlers to Netlify Functions in `netlify/functions/`, outputs `publish/` directory
- `AdapterTrait` in a shared `adapter-core` module defining: `fn prepare(&self, compiled: &CompiledOutput) -> Result<Bundle>`, `fn deploy(&self, bundle: &Bundle, config: &DeployConfig) -> Result<DeployResult>`
- `voce deploy` subcommand: detects adapter from config or prompts interactively, runs compilation, invokes adapter, calls platform CLI (vercel/wrangler/netlify) if available
- `.voce/config.toml` file format for project-level adapter configuration
- Dry-run mode: `voce deploy --dry-run` that produces the bundle without uploading
- Integration tests using the static adapter (no external service needed)

## Acceptance Criteria

- [ ] `voce deploy --adapter static` produces a valid `dist/` folder with all assets, correct relative paths, and working links
- [ ] `voce deploy --adapter vercel --dry-run` produces `.vercel/output/` matching Vercel Build Output API v3 spec
- [ ] `voce deploy --adapter cloudflare --dry-run` produces valid `wrangler.toml` and `_worker.js` for Pages
- [ ] `voce deploy --adapter netlify --dry-run` produces valid `netlify.toml` and function stubs
- [ ] ActionNode with `method: POST` compiles to a serverless function in Vercel/Cloudflare/Netlify adapters
- [ ] Static adapter output serves correctly from `python3 -m http.server` with no broken links
- [ ] `voce deploy` without `--adapter` flag prompts user to choose interactively
- [ ] `.voce/config.toml` is read for default adapter, custom domain, environment variables
- [ ] All adapter crates compile independently with `cargo build -p adapter-{name}`
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
