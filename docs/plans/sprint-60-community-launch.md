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
- ⏳ **Slice 2 — Package publishing:** crates.io (voce-schema /
  voce-validator / voce-compiler-dom) + npm (`@voce-ir/sdk`,
  `@voce-ir/mcp-server`, `@voce-ir/cli-chat`) + pre-built binaries
  (cargo-dist or release workflow) + Homebrew tap.
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
