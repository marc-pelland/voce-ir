# Sprint 60 — Community Launch & v1.1.0

**Phase:** 7 — Production Readiness
**Status:** Planned
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
