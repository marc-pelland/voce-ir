# Voce IR — Deployment & Publishing Guide

Step-by-step guide for publishing all components of Voce IR as open source.

---

## Prerequisites

Before starting, you need accounts and tokens for:

- [ ] **GitHub** — repo at `github.com/marcpelland/voce-ir`
- [ ] **crates.io** — API token (`cargo login`)
- [ ] **npm** — auth token (`npm login`, scope `@voce-ir`)
- [ ] **Cloudflare** — account for Pages deployment (voce-ir.xyz)
- [ ] **Domain** — voce-ir.xyz DNS configured

---

## Step 1: Initialize the GitHub Repository

```bash
# Initialize git if not already done
cd /path/to/voce-ir
git init
git add -A
git commit -m "Initial commit: Voce IR v1.1.0

15 Rust crates, 4 TypeScript packages, 7 compile targets, 172 tests.
Full pipeline: conversation → IR → validate → compile → deploy."

# Create the repo on GitHub (using gh CLI)
gh repo create marcpelland/voce-ir --public --source=. --push

# Or manually:
git remote add origin git@github.com:marcpelland/voce-ir.git
git branch -M main
git push -u origin main
```

### Configure GitHub repo settings

1. **Settings > General:**
   - Description: "AI-native intermediate representation for user interfaces"
   - Website: `https://voce-ir.xyz`
   - Topics: `ai`, `ui`, `compiler`, `intermediate-representation`, `rust`, `webassembly`, `accessibility`

2. **Settings > Branches > Branch protection for `main`:**
   - Require status checks (CI must pass)
   - Require pull request reviews (optional for solo, recommended later)

3. **Settings > Discussions:**
   - Enable GitHub Discussions
   - Create categories: Q&A, Show & Tell, Ideas, General

---

## Step 2: Publish Rust Crates to crates.io

Crates must be published in dependency order. Each `cargo publish` uploads to crates.io permanently.

```bash
# Login to crates.io
cargo login

# Step 2a: Publish schema (no dependencies on other voce crates)
cargo publish -p voce-schema

# Step 2b: Publish compiler crates (depend on schema)
cargo publish -p voce-compiler-dom
cargo publish -p voce-compiler-webgpu
cargo publish -p voce-compiler-wasm
cargo publish -p voce-compiler-email
cargo publish -p voce-compiler-ios
cargo publish -p voce-compiler-android

# Step 2c: Publish hybrid compiler (depends on dom + webgpu + wasm)
cargo publish -p voce-compiler-hybrid

# Step 2d: Publish adapter crates
cargo publish -p voce-adapter-core
cargo publish -p voce-adapter-static
cargo publish -p voce-adapter-vercel
cargo publish -p voce-adapter-cloudflare
cargo publish -p voce-adapter-netlify

# Step 2e: Publish validator/CLI (depends on everything)
cargo publish -p voce-validator

# Step 2f: Publish playground WASM (optional — not typically installed from crates.io)
# cargo publish -p voce-playground-wasm
```

**Important:** crates.io requires all path dependencies to be published first. If you get "dependency not found" errors, wait 1-2 minutes between publishes for the index to update.

**Before publishing:** Make sure all `Cargo.toml` path dependencies have a version fallback. For crates.io, path deps need to be replaced with version deps. You may need to temporarily edit them:

```toml
# Local development (current):
voce-schema = { path = "../schema" }

# For crates.io publishing, add version:
voce-schema = { path = "../schema", version = "0.1.0" }
```

### Verify installation

```bash
# From a clean environment:
cargo install voce-validator
voce --version
voce validate --help
```

---

## Step 3: Publish npm Packages

```bash
# Login to npm
npm login --scope=@voce-ir

# Publish SDK
cd packages/sdk
npm publish --access public

# Publish MCP server
cd ../mcp-server
npm publish --access public
```

### Verify

```bash
npm install @voce-ir/sdk
node -e "const { VoceClient } = require('@voce-ir/sdk'); console.log('OK')"
```

---

## Step 4: Create GitHub Release with Binaries

The CI workflow automatically builds binaries when you push a tag:

```bash
# Bump version in Cargo.toml
# (update workspace.package.version to "1.1.0")

# Commit and tag
git add -A
git commit -m "chore: bump version to v1.1.0"
git tag v1.1.0
git push origin main --tags
```

This triggers the `release` job in CI which:
1. Builds binaries for linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64
2. Creates a GitHub Release with the binaries attached
3. Auto-generates release notes from commits

### Manual release (if CI isn't set up yet)

```bash
# Build locally
cargo build --release -p voce-validator

# Create release via gh CLI
gh release create v1.1.0 \
  --title "Voce IR v1.1.0" \
  --notes "First public release. See CHANGELOG.md for details." \
  target/release/voce
```

---

## Step 5: Deploy voce-ir.xyz (GitHub Pages)

Everything deploys as a single site via GitHub Pages. The workflow at
`.github/workflows/pages.yml` builds three things into one `_site/` directory:

```
voce-ir.xyz/              -> Landing page (compiled from production IR)
voce-ir.xyz/docs/         -> mdBook documentation (30 pages)
voce-ir.xyz/playground/   -> Browser-based IR playground (WASM-powered)
```

### 5a: Enable GitHub Pages

1. Go to your repo: **Settings > Pages**
2. Under "Build and deployment", set **Source** to **GitHub Actions**
3. That's it — the workflow handles the rest

### 5b: Set up custom domain

1. In **Settings > Pages > Custom domain**, enter `voce-ir.xyz`
2. GitHub will show you DNS records to add. At your domain registrar, add:
   - **A records** (for apex domain `voce-ir.xyz`):
     ```
     185.199.108.153
     185.199.109.153
     185.199.110.153
     185.199.111.153
     ```
   - **CNAME record** (for `www`):
     ```
     www -> marcpelland.github.io
     ```
3. Check "Enforce HTTPS" once DNS propagates (may take a few minutes)
4. Add a `CNAME` file to the site root so GitHub remembers the domain:

```bash
echo "voce-ir.xyz" > _site/CNAME
```

(This is already handled in the workflow.)

### 5c: Trigger a deploy

The workflow runs automatically on every push to `main`. You can also trigger
it manually from **Actions > Deploy to GitHub Pages > Run workflow**.

### 5d: Verify

After the workflow completes (2-3 minutes):
- `https://voce-ir.xyz` — landing page
- `https://voce-ir.xyz/docs` — documentation
- `https://voce-ir.xyz/playground` — playground

### How it works

The `pages.yml` workflow does:
1. Compiles `examples/production/landing.voce.json` to `_site/index.html` using the Voce compiler
2. Builds the mdBook docs into `_site/docs/`
3. Builds the WASM playground into `_site/playground/`
4. Uploads `_site/` as a GitHub Pages artifact
5. Deploys to GitHub's CDN

Everything rebuilds from source on every push — no manual deploys needed

---

## Step 6: Homebrew Tap (Optional)

Create a separate repo for the tap:

```bash
# Create repo: github.com/marcpelland/homebrew-voce-ir
gh repo create marcpelland/homebrew-voce-ir --public
```

Create the formula file:

```ruby
# Formula/voce-ir.rb
class VoceIr < Formula
  desc "AI-native intermediate representation for user interfaces"
  homepage "https://voce-ir.xyz"
  url "https://github.com/marcpelland/voce-ir/archive/refs/tags/v1.1.0.tar.gz"
  # sha256 "..." # Fill in after release
  license "Apache-2.0"

  depends_on "rust" => :build

  def install
    system "cargo", "install", "--root", prefix, "--path", "packages/validator"
  end

  test do
    system "#{bin}/voce", "--version"
  end
end
```

Users install with:
```bash
brew tap marcpelland/voce-ir
brew install voce-ir
```

---

## Step 7: Community Setup

### Discord

1. Create server at discord.com
2. Create channels: `#general`, `#help`, `#showcase`, `#development`, `#announcements`
3. Add invite link to README.md

### Launch content

1. **Blog post** — Already at `docs/blog/launch.md`. Deploy to voce-ir.xyz/blog
2. **Hacker News** — Submit as "Show HN: Voce IR – AI-native UI IR that compiles to 7 targets (Rust)" with link to voce-ir.xyz
3. **ProductHunt** — Create page, record 60-second demo screencast showing prompt -> compile -> deploy
4. **Twitter/X** — Thread walking through: problem, thesis, architecture, demo, link

---

## Step 8: Post-Launch Checklist

- [ ] `cargo install voce-validator` works from crates.io
- [ ] `npm install @voce-ir/sdk` works from npm
- [ ] voce-ir.xyz is live and loads in < 1s
- [ ] voce-ir.xyz/docs serves the mdBook site
- [ ] voce-ir.xyz/playground loads and compiles IR in-browser
- [ ] GitHub Discussions are enabled
- [ ] First external issue gets a response within 24 hours

---

## Domain DNS Summary (GitHub Pages)

| Record | Type | Name | Value |
|--------|------|------|-------|
| Apex domain | A | `@` | `185.199.108.153` (+ .109, .110, .111) |
| www redirect | CNAME | `www` | `marcpelland.github.io` |

All paths (landing, docs, playground) are served from one GitHub Pages deployment.

---

## Publish Order Summary

```
1. git push (GitHub)
2. cargo publish (crates.io) — in dependency order
3. npm publish (npm)
4. git tag v1.1.0 && git push --tags (triggers binary builds)
5. Enable GitHub Pages (Settings > Pages > Source: GitHub Actions)
6. Set custom domain voce-ir.xyz (Settings > Pages > Custom domain)
7. Homebrew tap (optional)
7. Launch content (HN, ProductHunt, Twitter)
```
