# Sprint 60 — Community Launch & v1.1.0

**Phase:** 7 — Production Readiness
**Status:** In progress (Slice 1: README discoverability shipped)

## Implementation Status

- ✅ **Slice 1 — README discoverability:** the technical moat was
  meaningfully ahead of the front door. Surfaced everything shipped
  since the last README pass: a new **Conversational Interfaces**
  section documenting `voce-chat` (S66) and `@voce-ir/mcp-server`
  (S65 + S79; 22 tools) — both previously invisible to anyone landing
  on the repo. A new **Agent Contract (6 envelopes)** Features
  subsection with the schema-locked envelope table, and the new CLI
  surfaces (`voce skills` / `graph` / `doctor` / `conformance run` /
  `fix --until-clean --plan`) added to the CLI Reference. New
  Documentation links to `docs/schema/contract/v1/`,
  `docs/compatibility-matrix.md`, and `docs/accessibility/OVERVIEW.md`
  (all three verified present). Reconciled stale numbers: "46 rules"
  → 52, "12 fix codes" → 17, "172 tests" → 391 (321 Rust + 70
  vitest). The S54 historical roadmap entry left as-is (accurate
  point-in-time marker).
- ✅ **Slice 2 — Package publishing prep (in-repo bits):** every
  in-repo piece of the publish path now exists; only credentials +
  GitHub Environment setup + the actual tag-push remain operator
  work (documented in `docs/launch/release-runbook.md`).
  - **Cargo:** workspace metadata audit clean — `voce-schema`,
    `voce-validator`, `voce-compiler-dom` all carry name / version
    (workspace-inherited) / license (Apache-2.0) / repository /
    rust-version / description / keywords / categories / readme.
    Cargo publish-readiness verified.
  - **npm:** the three target packages (`@voce-ir/sdk`,
    `@voce-ir/mcp-server`, `@voce-ir/cli-chat`) gained license /
    repository (with `directory`) / homepage / bugs / keywords /
    author / `files` allowlist / `prepublishOnly` build hook.
    Each now ships its own `LICENSE` + a focused per-package
    `README.md` (proper npm landing pages, not the kitchen-sink
    workspace README).
  - **Real publish-blockers found and fixed:** (1) `cli-chat`
    depended on `"@voce-ir/mcp-server": "file:../mcp-server"`,
    invalid for a published package — converted to `^0.4.0`;
    (2) `@voce-ir/mcp-server` exposed only the `voce-mcp` binary
    but launch docs / Claude Code config snippets reference
    `voce-mcp-server` — added `voce-mcp-server` as a bin alias
    (non-breaking, both names work). (3) Added a root
    `package.json` with `workspaces: ["packages/*"]` so fresh
    clones resolve `@voce-ir/*` locally via symlinks rather than
    failing against the (unpublished) registry.
  - **CI release workflow** extended in `.github/workflows/ci.yml`:
    new `publish-crates` job (publishes schema → compiler-dom →
    validator in dependency order with index-refresh sleeps, gated
    on `refs/tags/v*` + a `release-crates-io` GitHub Environment for
    manual approval); new `publish-npm` job (mcp-server → cli-chat
    → sdk, gated on `release-npm` Environment); Windows x86_64
    added to the binary release matrix (now 5 targets: linux x86_64
    / linux aarch64 / macos x86_64 / macos aarch64 / **windows
    x86_64**).
  - **Launch-blocker finding surfaced loudly:** the workspace
    `LICENSE` is the Apache-2.0 *header stub*, not the full
    ~200-line license text. Three options presented in the runbook
    (replace with full Apache 2.0; switch to MIT; keep as-is and
    accept the compliance gap). Not auto-fixed — legal-posture call.
  - **`docs/launch/release-runbook.md`:** the operator checklist —
    required secrets (`CARGO_REGISTRY_TOKEN`, `NPM_TOKEN`),
    Environment protection rules, tag-and-push flow, rollback paths,
    dry-run commands, post-publish smoke. Homebrew tap explicitly
    deferred — not blocking.
- ✅ **Slice 3 — Repo polish (in-repo bits):** CHANGELOG regenerated
  for ~14 commits of work since 2026-05-01 — S65 / S66 / S69 parts
  3–4 / S70 / S71 / S72 part 2 / S82 closeout / S68 slices 1–2 +
  D5+D6 / S79 (entire sprint) / S91 slice 1 / S60 slice 1 — plus
  pointers to the newly-scoped S92 / S93 / S79b. LICENSE / CoC /
  CONTRIBUTING / SECURITY audit: all present at root, no action
  needed. PR template + 2 issue templates (bug, feature) already
  exist under `.github/`; added a 3rd template `new_compile_target.md`
  per the S60 deliverables list, structured around the canonical
  `ConformanceClass` taxonomy so proposals self-classify against
  the contract from the start. **Branch protection** is GitHub-UI
  config (not in-repo) — flagged for the launch operator.
- ⏳ **Slice 4 — Community infra:** GitHub Discussions, Discord
  server, response-SLA scaffolding.
- ✅ **Slice 5 — Launch content (drafts):** `docs/launch/` set
  shipped — blog post (~1900 words, the canonical case), Show HN
  copy (≤500 words, low-hype HN-appropriate framing), 10-post X
  thread (each ≤280 chars, with media positions marked), 60–75 sec
  demo-video shot list (7 scenes, concrete typing + browser cuts +
  what NOT to film), plus a `docs/launch/README.md` index with
  launch sequencing + voice notes + operator handoff. All
  artifacts are first drafts intended for heavy editing before
  publishing — the value is having something concrete to react to.
  Heavy operator work (actually publishing, recording video,
  posting to platforms) is the launch-day work, not draftable
  here.
- ⏳ **Slice 6 — v1.1.0 tag + GitHub Release.**

---
**Goal:** Public launch of Voce IR as an open-source project. Publish to npm and crates.io, set up community infrastructure, write launch content, and gather the first 10 external users. Tag v1.1.0.

**Depends on:** Sprint 55 (docs site live), Sprint 58 (production demo proves viability), Sprint 59 (performance meets targets)

---

## Deliverables

- **Package publishing:**
  - Publish `voce-schema`, `voce-validator`, `voce-compiler-dom` to crates.io with complete metadata (description, license, repository, keywords, categories)
  - Publish `@voce-ir/sdk`, `@voce-ir/mcp-server` to npm with correct peer dependencies
  - Pre-built binaries for macOS (arm64, x86_64), Linux (x86_64, aarch64), Windows (x86_64) via `cargo-dist` or GitHub Releases
  - `brew install voce-ir` formula submitted to homebrew-core (or tap)
- **Repository polish:**
  - LICENSE file (MIT)
  - CONTRIBUTING.md with development setup, code conventions, PR process
  - CHANGELOG.md generated from commit history (Sprints 1-60)
  - Issue templates: bug report, feature request, new compiler target proposal
  - PR template with checklist (tests, clippy, fmt, docs)
  - GitHub Actions CI: test, clippy, fmt, build binaries, publish on tag
  - Branch protection on `main`: require CI pass, require review
- **Community infrastructure:**
  - GitHub Discussions enabled (categories: Q&A, Show & Tell, Ideas, General)
  - Discord server with channels: #general, #help, #showcase, #development, #announcements
  - Code of Conduct (Contributor Covenant v2.1)
- **Launch content:**
  - Blog post: "Introducing Voce IR: AI-Native UI Without Human-Readable Code" (~2000 words, published on voce-ir.xyz/blog)
  - ProductHunt launch page with demo video (60-second screencast: prompt to deployed site)
  - Hacker News Show HN post
  - Twitter/X thread (10 posts) walking through the architecture
- **v1.1.0 release:**
  - Git tag `v1.1.0` on `main`
  - GitHub Release with changelog, binary assets, and migration notes from v1.0.0
  - npm packages tagged `latest`
  - crates.io versions published
- **Feedback loop:** GitHub issue labeled `first-user-feedback` for tracking external reports

## Acceptance Criteria

- [ ] `cargo install voce-ir` installs a working CLI from crates.io
- [ ] `npm install @voce-ir/sdk` installs and `import { VoceClient } from '@voce-ir/sdk'` works
- [ ] Pre-built binary for macOS arm64 runs without Rust toolchain installed
- [ ] GitHub Actions CI passes on `main` (test + clippy + fmt + build)
- [ ] CONTRIBUTING.md, LICENSE, and CODE_OF_CONDUCT.md are present and complete
- [ ] GitHub Discussions has at least the 4 required categories configured
- [ ] Discord server is created with invite link in README
- [ ] Blog post is published and accessible at voce-ir.xyz/blog
- [ ] ProductHunt page is live with demo video
- [ ] Show HN post is submitted
- [ ] Git tag `v1.1.0` exists with corresponding GitHub Release
- [ ] At least 10 external GitHub stars or npm downloads within 7 days of launch (tracked, not guaranteed)
- [ ] First external issue or discussion post received and responded to within 24 hours
