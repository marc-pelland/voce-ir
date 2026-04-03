# Sprint 30 — Public Launch

**Status:** Planned
**Goal:** Prepare and execute the public launch of Voce IR v0.3.0: write the launch blog post, build a browser-based playground, polish all documentation, tag the release, and publish to npm/crates.io. After this sprint, Phase 3 is complete and Voce IR is publicly available for AI-first builders.
**Depends on:** Sprint 29 (demo projects complete, full pipeline battle-tested)

---

## Deliverables

1. Launch blog post: narrative + technical deep-dive + demo GIFs
2. Web playground: browser-based "try Voce IR" experience (no install required)
3. Documentation site: getting started, CLI reference, architecture guide, API docs
4. npm packages published: `@voce-ir/sdk`, `@voce-ir/ai-bridge`, `@voce-ir/mcp-server`
5. crates.io crates published: `voce-schema`, `voce-validator`, `voce-compiler-dom`
6. v0.3.0 git tag and GitHub release with changelog
7. Public GitHub repository with README, LICENSE, CONTRIBUTING

---

## Tasks

### 1. Launch Blog Post (`docs/blog/launch.md`)

Write the narrative: what Voce IR is, why AI-native IR matters, the anti-vibe-coding philosophy, what "sotto voce" means for the project. Include: architecture diagram, demo GIF (from S29), benchmark comparison table, code-free workflow walkthrough. Target: 1500-2000 words. Tone: technical but accessible, opinionated but not preachy.

### 2. Web Playground (`packages/playground/`)

Browser-based single-page app. Left panel: text input for a prompt. Right panel: live preview of compiled HTML. Flow: user types prompt, hits Generate, sees loading states per agent, then rendered output. Built with vanilla HTML/CSS/JS (dogfooding: could be built with Voce IR itself). Calls a hosted API endpoint or runs client-side via WASM validator. Deploy to Cloudflare Pages.

### 3. Documentation Polish

Review and update all docs: `docs/PRD.md`, `docs/ROADMAP.md`, `docs/PROJECT_PLAN.md`. Write new: Getting Started guide (install, first generation, first edit), CLI Reference (all commands with examples), Architecture Guide (system diagram, data flow, agent roles), Style Pack Guide (using and creating packs). Ensure all code examples work.

### 4. Package Publishing

Prepare npm packages: set versions to 0.3.0, write package READMEs, configure `publishConfig`, ensure `types` field points to declarations. Prepare crates: update `Cargo.toml` versions, write crate-level doc comments, ensure `cargo publish --dry-run` succeeds. Publish in order: schema -> validator -> compiler -> sdk -> ai-bridge -> mcp-server.

### 5. GitHub Repository Prep

Write root `README.md`: elevator pitch, feature list, quick start, architecture diagram, demo GIF, links to docs. Add `LICENSE` (MIT). Add `CONTRIBUTING.md`: development setup, PR process, code style, testing expectations. Configure: issue templates, PR template, branch protection on main, CI via GitHub Actions (test + lint + format check).

### 6. Release Tag & Changelog

Write `CHANGELOG.md` covering all 3 phases: v0.1.0 (schema + validator), v0.2.0 (DOM compiler), v0.3.0 (AI bridge + conversational design + MCP). Tag `v0.3.0` on main. Create GitHub Release with changelog body, link to blog post and playground. Attach pre-built binaries for macOS/Linux/Windows.

### 7. Launch Checklist

Final checks before going public: all tests pass (`cargo test --workspace && npm test`), all demos reproduce cleanly, playground works end-to-end, docs have no broken links, README renders correctly on GitHub, npm packages install cleanly in a fresh project, MCP server registers correctly in Claude Code. Smoke test the full flow on a clean machine.

---

## Files to Create

- `docs/blog/launch.md`
- `packages/playground/index.html`
- `packages/playground/style.css`
- `packages/playground/app.js`
- `docs/guides/getting-started.md`
- `docs/guides/cli-reference.md`
- `docs/guides/architecture.md`
- `docs/guides/style-packs.md`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `LICENSE`
- `.github/workflows/ci.yml`
- `.github/ISSUE_TEMPLATE/bug_report.md`
- `.github/ISSUE_TEMPLATE/feature_request.md`
- `.github/pull_request_template.md`

---

## Acceptance Criteria

- [ ] Blog post is written, reviewed, and ready to publish (1500+ words)
- [ ] Web playground loads in browser, accepts a prompt, and shows compiled output
- [ ] Playground works without any local installation
- [ ] Getting Started guide takes a new user from zero to first generated page in under 5 minutes
- [ ] CLI Reference documents all commands with working examples
- [ ] All npm packages publish successfully with correct versions and type declarations
- [ ] All crates publish successfully to crates.io
- [ ] `v0.3.0` tag exists on main with GitHub Release and changelog
- [ ] README renders correctly on GitHub with architecture diagram and demo GIF
- [ ] CI pipeline passes: test, lint, format check on every PR
- [ ] Clean machine smoke test: install from npm/crates.io, run full flow, get HTML output
- [ ] CONTRIBUTING.md covers setup, testing, and PR process
