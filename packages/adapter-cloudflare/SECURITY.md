# Security — adapter-cloudflare

## Token / secret handling audit (S70 Day 3)

**Cloudflare API tokens are NOT touched by this adapter.** Deployment shells
out to `wrangler` via `std::process::Command::new("wrangler")`. Wrangler
manages its own auth (`CLOUDFLARE_API_TOKEN` env var or
`~/.config/.wrangler/config.toml`). Voce code never reads, writes, caches,
or logs the token.

### What we DO touch

- `bundle.output_dir` — the directory we wrote compiled output to. Public.
- `config.settings["project_name"]` — passed via `--project-name` to
  wrangler. Public; not a secret.
- `wrangler pages deploy` stdout — captured and surfaced as
  `DeployResult.message`. Wrangler does not print tokens to stdout in
  current versions.

### What we DON'T touch

- `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID` — set by the user;
  inherited by the spawned `wrangler` process via the OS env.
- `~/.config/.wrangler/` — managed exclusively by Wrangler.

### Trust boundary

Same as adapter-vercel: this adapter trusts `wrangler` on `PATH` to handle
auth correctly. The threat model entry "elevation of privilege" covers
malicious binaries impersonating wrangler on `PATH`.

### Reportable issues

See the repo root `SECURITY.md` for the disclosure policy.
