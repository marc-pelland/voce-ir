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

## Step 5: Deploy voce-ir.xyz (Production Site)

### 5a: Landing page

```bash
# Compile the production IR
voce compile examples/production/landing.voce.json \
  -o examples/production/dist/index.html \
  --skip-fonts --minify

# Deploy to Cloudflare Pages
cd examples/production/dist
npx wrangler pages deploy . --project-name voce-ir
```

Or set up continuous deployment:
```bash
# In Cloudflare dashboard:
# Pages > Create project > Connect to Git
# Repository: marcpelland/voce-ir
# Build command: cargo run -p voce-validator -- compile examples/production/landing.voce.json -o dist/index.html --skip-fonts --minify
# Build output directory: dist
```

### 5b: Documentation site

```bash
# Build docs
cd docs/site
mdbook build

# Deploy to Cloudflare Pages (as a separate project or subdirectory)
npx wrangler pages deploy book --project-name voce-ir-docs
```

**DNS setup for docs.voce-ir.xyz:**
1. Cloudflare dashboard > DNS > Add CNAME record:
   - Name: `docs`
   - Target: `voce-ir-docs.pages.dev`

### 5c: Playground

```bash
# Build WASM
cd packages/playground-wasm
PATH="$HOME/.cargo/bin:$PATH" wasm-pack build --target web --release

# Copy WASM to playground
cp -r pkg/* ../playground/wasm/

# Build playground
cd ../playground
npm install
npm run build

# Deploy
npx wrangler pages deploy dist --project-name voce-ir-playground
```

**DNS setup for playground.voce-ir.xyz:**
- CNAME: `playground` -> `voce-ir-playground.pages.dev`

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
- [ ] docs.voce-ir.xyz serves the mdBook site
- [ ] playground.voce-ir.xyz loads and compiles IR in-browser
- [ ] GitHub Discussions are enabled
- [ ] First external issue gets a response within 24 hours
- [ ] Analytics set up on voce-ir.xyz (Cloudflare Web Analytics — privacy-friendly)

---

## Domain DNS Summary (Cloudflare)

| Record | Type | Name | Target |
|--------|------|------|--------|
| Landing page | CNAME | `@` | `voce-ir.pages.dev` |
| Documentation | CNAME | `docs` | `voce-ir-docs.pages.dev` |
| Playground | CNAME | `playground` | `voce-ir-playground.pages.dev` |

---

## Publish Order Summary

```
1. git push (GitHub)
2. cargo publish (crates.io) — in dependency order
3. npm publish (npm)
4. git tag v1.1.0 && git push --tags (triggers binary builds)
5. wrangler pages deploy (Cloudflare — landing, docs, playground)
6. Homebrew tap (optional)
7. Launch content (HN, ProductHunt, Twitter)
```
