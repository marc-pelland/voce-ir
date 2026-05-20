# Release Runbook — v1.1.0

The operator-facing checklist for the launch. The release machinery
is in `.github/workflows/ci.yml` (`publish-crates`, `publish-npm`,
`release`, `publish-release` jobs, all gated on `refs/tags/v*`).
Everything below is what you do **outside** the repo to make a
tagged release actually publish.

---

## ⚠ Pre-flight finding — LICENSE compliance

**The workspace `LICENSE` file is the Apache-2.0 *header stub*, not
the full ~200-line Apache 2.0 license text.** `Cargo.toml` correctly
declares `license = "Apache-2.0"` (which Cargo treats as an SPDX
identifier and crates.io accepts), but distributing under Apache 2.0
also requires shipping the full license text alongside the code.
Three options:

1. **Replace `LICENSE` with the full Apache 2.0 text** (recommended;
   ~200 lines, canonical text at
   <https://www.apache.org/licenses/LICENSE-2.0.txt>). Per-package
   `LICENSE` files (already created under `packages/sdk`,
   `packages/mcp-server`, `packages/cli-chat`) currently mirror the
   workspace stub and need the same fix.
2. **Switch to MIT.** Update `Cargo.toml`'s `workspace.package.license`
   to `"MIT"` and replace `LICENSE` with the MIT text. Smaller, simpler,
   no requirement to ship a 200-line file.
3. **Keep as-is.** Technically out of Apache-2.0 compliance; not a
   crates.io / npm blocker but a real legal posture decision.

Pick before tagging v1.1.0. The choice doesn't change the workflow
machinery — only the file content.

---

## Required GitHub repository secrets

Set under **Settings → Secrets and variables → Actions**:

| Secret | What it is | Used by |
| --- | --- | --- |
| `CARGO_REGISTRY_TOKEN` | crates.io API token (`cargo login`, scope: publish-new + publish-update) | `publish-crates` job |
| `NPM_TOKEN` | npm.com automation token (scope: publish for `@voce-ir`) | `publish-npm` job |

**`GITHUB_TOKEN`** is provided automatically by GitHub Actions; no
setup needed for the GitHub Release creation step.

## Required GitHub environment protection rules

Both publish jobs reference protected GitHub **Environments** so a
publish requires manual approval after a tag is pushed — this is the
safety net that prevents a fat-fingered tag from accidentally
publishing.

Create these under **Settings → Environments**:

| Environment | Suggested protection |
| --- | --- |
| `release-crates-io` | Required reviewers: yourself. Wait timer: 5 min (gives you time to cancel if you tagged wrong). |
| `release-npm` | Same. |

Without these environments, the jobs will run unconditionally on tag
push — which works but removes the manual-approval safety net.
**Recommended:** add the environments before the first real publish.

## Dev-workflow note — npm workspaces

The repo root now has a `package.json` declaring `workspaces:
["packages/*"]`. This is what makes `cli-chat`'s
`"@voce-ir/mcp-server": "^0.4.0"` dependency resolve to the
**local workspace package** during dev (via a `node_modules/@voce-ir/mcp-server -> ../../packages/mcp-server`
symlink) — not the published-but-stale npm version, and not the
broken `file:../mcp-server` pre-launch syntax. The published
semver range is the only thing on the registry; the workspace
shortcut is local-only.

Fresh-clone workflow:

```bash
git clone https://github.com/marc-pelland/voce-ir
cd voce-ir
npm install          # resolves all workspaces; symlinks @voce-ir/* locally
npm test --workspace=@voce-ir/mcp-server
npm test --workspace=@voce-ir/cli-chat
```

## Required external accounts (one-time setup)

| Service | Action |
| --- | --- |
| **crates.io** | Log in via GitHub, generate an API token with `publish-new` + `publish-update` scopes. Reserve the three crate names if not already: `voce-schema`, `voce-validator`, `voce-compiler-dom`. |
| **npm.com** | Create org (or user) account. Create the `@voce-ir` scope (free tier supports public-scoped packages). Generate an automation-type token. |
| **Homebrew tap (optional)** | Out of scope for the GitHub Actions workflow — Homebrew formulae are typically a separate `homebrew-voce` repo. Defer to a follow-up if not blocking. |

## Tag → publish flow

Once the secrets + environments are in place:

```bash
# 1. Make sure HEAD is what you want to ship
git status
cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings
(cd packages/mcp-server && npm test)

# 2. Bump the version in Cargo.toml (workspace.package.version)
#    and in each package.json that's publishing. The workspace
#    inherits to all 15 Rust crates; the 3 npm packages need
#    individual edits.
#    Voce convention: tag and version always match.

# 3. Commit the version bump
git commit -am "release: v1.1.0"

# 4. Tag and push
git tag v1.1.0
git push origin main --follow-tags
```

CI now fires:

- `check` (lint + test + build) — must pass before anything publishes
- `coverage`, `supply-chain`, `cross-target-parity`, `mcp-server`,
  `cli-chat`, `frontend-verify`, `wasm-size`, `lighthouse` — all
  must pass too (they're not in `needs:` for the publish jobs today,
  so a `check` pass is sufficient — consider adding the gates if you
  want stricter publish guarantees)
- `release` (4 binary builds: linux x86_64/aarch64, macos
  x86_64/aarch64, **windows x86_64**) — produces tarballs
- `sbom` — CycloneDX SBOMs
- `publish-crates` — **waits for your approval** in the
  `release-crates-io` environment; on approval, publishes
  `voce-schema` → `voce-compiler-dom` → `voce-validator` in
  dependency order
- `publish-npm` — **waits for your approval** in the `release-npm`
  environment; on approval, publishes `@voce-ir/mcp-server` →
  `@voce-ir/cli-chat` → `@voce-ir/sdk`
- `publish-release` — creates the GitHub Release with binaries +
  SBOM bundle + auto-generated notes

You can approve `publish-crates` and `publish-npm` independently — if
one is in a bad state, the other still ships.

## Rollback if something goes wrong

**Rust (crates.io):** crates can't be deleted, only **yanked**
(`cargo yank --vers <ver> <crate>`). Yanking prevents new resolves
but doesn't break existing lockfiles. Bump to a patch version and
re-publish if the fix is small.

**npm:** packages can be unpublished within 72 hours of publish
(`npm unpublish @voce-ir/<pkg>@<version>`); after that, only deprecate
(`npm deprecate @voce-ir/<pkg>@<version> "use 1.1.1+"`). Same
patch-bump-and-republish flow for fixes.

**GitHub Release:** delete the release in the UI (the tag stays;
re-running the workflow re-creates the release).

**The tag itself:** `git push --delete origin v1.1.0` + delete
local tag. The crates.io / npm publishes survive — handle those
separately per above.

## Dry-run before the real thing

Test the publish path without actually publishing:

```bash
# Cargo: build the tarballs that would be published
cargo publish --dry-run -p voce-schema
cargo publish --dry-run -p voce-compiler-dom
cargo publish --dry-run -p voce-validator

# npm: see what would be in the tarball (--dry-run uploads nothing,
# just prints the file list and metadata)
cd packages/mcp-server && npm publish --dry-run --access public
cd ../cli-chat       && npm publish --dry-run --access public
cd ../sdk            && npm publish --dry-run --access public
```

The cargo dry-run also catches metadata errors (missing description,
unrecognized SPDX, etc.) before they hit the registry. Useful before
the first ever publish.

## Post-publish smoke

```bash
# Rust
cargo install voce-validator --version 1.1.0
voce skills
voce conformance run --target dom --corpus tests/fixtures

# npm (one-by-one — global installs land in different bin dirs)
npm install -g @voce-ir/mcp-server@1.1.0
voce-mcp-server --version  # confirms the new alias works
npm install -g @voce-ir/cli-chat@1.1.0
voce-chat --version
```

If any of these fail, see "Rollback" above — don't try to patch
in-place.
