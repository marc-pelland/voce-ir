# voce deploy

Validate, compile, and prepare a deployment bundle for a target hosting
platform. Wraps the full pipeline (validate, compile) and adds
platform-specific configuration files.

## Usage

```
voce deploy <FILE> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to a `.voce.json` IR file (required) |

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--adapter <ADAPTER>` | `static` | Deployment target: `static`, `vercel`, `cloudflare`, `netlify` |
| `--dry-run` | off | Show what would be generated without writing any files |

## Adapters

Each adapter produces a deployment-ready bundle in `dist/`:

| Adapter | Output |
|---------|--------|
| `static` | Compiled HTML only. Suitable for any static file host (S3, GitHub Pages, rsync). |
| `vercel` | HTML plus `vercel.json` with routing and cache headers. |
| `cloudflare` | HTML plus `_headers` and `_redirects` files for Cloudflare Pages. |
| `netlify` | HTML plus `netlify.toml` with headers and redirect rules. |

## Configuration

The deploy command reads defaults from `.voce/config.toml` if present:

```toml
[deploy]
adapter = "vercel"
output_dir = "dist"
```

Command-line flags override config file values.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Deployment bundle created successfully |
| `1` | Validation failed |
| `2` | Compilation failed |
| `3` | Deployment preparation failed (e.g., missing config, I/O error) |

## Examples

Deploy as static files (default):

```bash
voce deploy app.voce.json
# writes dist/app.html
```

Deploy to Vercel:

```bash
voce deploy app.voce.json --adapter vercel
# writes dist/app.html and dist/vercel.json
```

Preview what a Cloudflare deploy would produce:

```bash
voce deploy app.voce.json --adapter cloudflare --dry-run
```

Deploy with config file defaults:

```bash
# With .voce/config.toml setting adapter = "netlify"
voce deploy app.voce.json
# uses netlify adapter from config
```

## Dry Run

The `--dry-run` flag prints the list of files that would be written and their
approximate sizes, without creating or modifying anything on disk. Use this to
verify adapter output before committing to a deploy.

```
$ voce deploy app.voce.json --adapter vercel --dry-run
[dry-run] dist/app.html (12.4 KB)
[dry-run] dist/vercel.json (0.3 KB)
```

## Pipeline

The deploy command runs the full pipeline internally:

1. `voce validate <FILE>` -- abort on errors
2. `voce compile <FILE>` -- produce HTML
3. Generate adapter-specific files
4. Write all files to the output directory
