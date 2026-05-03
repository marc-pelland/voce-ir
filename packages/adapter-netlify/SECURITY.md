# Security — adapter-netlify

## Token / secret handling audit (S70 Day 3)

**Netlify auth tokens are NOT touched by this adapter.** Deployment shells
out to the `netlify` CLI via `std::process::Command::new("netlify")`. The
CLI manages its own auth (`NETLIFY_AUTH_TOKEN` env var or
`~/.config/netlify/config.json`). Voce code never reads, writes, caches,
or logs the token.

### What we DO touch

- `bundle.output_dir` — the directory we wrote compiled output to. Public.
- `netlify deploy` stdout — captured and surfaced as
  `DeployResult.message`. Netlify's CLI does not print tokens to stdout
  in current versions.

### What we DON'T touch

- `NETLIFY_AUTH_TOKEN`, `NETLIFY_SITE_ID` — set by the user; inherited by
  the spawned `netlify` process via the OS env.
- `~/.config/netlify/` — managed exclusively by the Netlify CLI.

### Trust boundary

Same as the other adapters: trusts the vendor CLI on `PATH`. Compromise
of that binary is out of scope here; see the threat model's "elevation
of privilege" entry.

### Reportable issues

See the repo root `SECURITY.md` for the disclosure policy.
