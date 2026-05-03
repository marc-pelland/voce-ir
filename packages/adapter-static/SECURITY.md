# Security — adapter-static

## Token / secret handling audit (S70 Day 3)

**This adapter has no auth surface.** It writes compiled HTML and assets to
a local directory; deploying to a static host (S3, GCS, GitHub Pages,
Cloudflare R2, etc.) is the user's responsibility, with whatever tooling
they prefer.

### What we DO touch

- `bundle.output_dir` — the directory we write to. Public.
- File system writes via `bundle.write_to_disk()`. Standard process
  permissions apply.

### What we DON'T touch

- Any cloud credentials. Period. There is no env var this adapter reads,
  no token cache it consults, no third-party service it talks to.

### Trust boundary

The user picks the host and the upload tool. This adapter terminates at
the local file system. If the user's chosen host requires auth, that's
the host's problem to solve.

### Reportable issues

See the repo root `SECURITY.md` for the disclosure policy.
