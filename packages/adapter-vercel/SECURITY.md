# Security — adapter-vercel

## Token / secret handling audit (S70 Day 3)

**Vercel auth tokens are NOT touched by this adapter.** Deployment shells out
to the official `vercel` CLI via `std::process::Command::new("vercel")`, and
the CLI manages its own auth (`~/.vercel/auth.json` or the `VERCEL_TOKEN`
env var that the user sets in their shell). Voce code never reads, writes,
caches, or logs the token.

### What we DO touch

- `bundle.output_dir` — the directory we wrote compiled output to. Public.
- `vercel --prebuilt` stdout — the CLI's output is captured and surfaced as
  `DeployResult.message`. If the Vercel CLI ever printed a token in its
  stdout, that string would land in the message. Vercel's CLI does not
  emit tokens to stdout in current versions; we treat the stdout as
  trusted-but-third-party.

### What we DON'T touch

- `VERCEL_TOKEN`, `VERCEL_ORG_ID`, `VERCEL_PROJECT_ID` — set by the user;
  inherited by the spawned `vercel` process via the OS env. Never read by
  Voce.
- `~/.vercel/` directory — managed exclusively by the Vercel CLI.

### Trust boundary

This adapter trusts the `vercel` CLI on the user's `PATH` to handle auth
correctly. If a malicious binary is named `vercel` and earlier on `PATH`,
that's a separate compromise outside this adapter's scope (covered by the
threat model's "elevation of privilege" entry).

### Verification

```sh
# Confirm Voce never reads VERCEL_TOKEN at compile time.
grep -rE 'VERCEL_TOKEN|vercel.*token' packages/adapter-vercel/src/
# (Expect: no matches.)
```

### Reportable issues

If you find a way for this adapter to leak a token (stdout capture, env
inheritance, Cargo build script, etc.), follow the disclosure policy in
the repo root `SECURITY.md`.
