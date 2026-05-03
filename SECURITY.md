# Security Policy

## Reporting a vulnerability

If you discover a security vulnerability in Voce IR, please report it
privately. **Do not open a public GitHub issue** for exploitable
findings — coordinate via email so a fix can land before the issue is
widely known.

Email: **marc@voce-ir.xyz**

Please include:

- A description of the vulnerability
- Reproduction steps (a minimal IR file or a curl + payload is ideal)
- Potential impact (what can an attacker do, against whom)
- Suggested fix, if you have one

### Response SLA

| Stage | Target |
|---|---|
| Acknowledgement of receipt | within **48 hours** |
| Initial assessment + severity rating | within **7 days** |
| Coordinated fix or mitigation plan | within **30 days** for high/critical, **90 days** for low/medium |
| Public disclosure | after a fix has shipped, with credit to the reporter unless they request anonymity |

If a vulnerability is already public (e.g. a known RustSec advisory in
a transitive dep), the SLA tightens — fix is expected within 7 days
regardless of severity, since the advisory itself is the public
disclosure.

### Encryption

For sensitive reports, you can encrypt to my public PGP key (fingerprint
will be added once the project moves to a multi-maintainer setup; for
now, plain email to a personal address is the channel).

## Scope

### In scope

The following count as security bugs in this project:

- **Compiled output (`compiler-dom`, `compiler-email`, etc.)** — XSS,
  injection, CSP bypass, JSON-LD breakout, dangerous-URL execution, or
  other vulnerabilities in the HTML/JS/CSS the compiler emits.
- **Validator bypass** — IR that passes validation but produces insecure
  output. This includes finding cases where a SEC* rule is supposed to
  catch something but doesn't.
- **CLI binaries (`voce`, `voce-mcp`, `voce-chat`)** — command injection,
  path traversal, secret leakage, or any way to make the CLI take an
  action on behalf of the user that they didn't authorize.
- **`.voce/` storage** — append-only invariant violations, atomic-write
  races, schema-validation bypass that allows tampered entries to be
  accepted as valid.
- **MCP server** — tool-use exploits, transport-level confusion, or any
  way for an MCP client to escalate beyond the documented tool surface.
- **WASM playground** — sandbox escapes, data exfiltration, or any way
  for a malicious IR to break out of the browser sandbox.
- **Adapter trust boundaries** — see each adapter's `SECURITY.md`. Token
  leakage via stdout / env inheritance is in scope.

### Out of scope

The following are **not** considered security bugs (see
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md) for the
full reasoning):

- **`PATH` poisoning** — a malicious `voce` / `vercel` / `wrangler` /
  `netlify` binary on the user's `PATH`. The user owns their `PATH`.
- **File-system permissions** — anyone with write access to your project
  directory can write to `.voce/`. That's by design.
- **Encryption at rest** — `.voce/sessions/` is gitignored but lives in
  plaintext on disk. Use FileVault / dm-crypt / equivalent.
- **Side-channel attacks** — timing, cache, power. Voce doesn't process
  secrets in a way these would matter.
- **Vendor CLI behavior** — if `vercel`/`wrangler`/`netlify` mishandle a
  token, that's a vendor bug, not ours.
- **Adversarial fine-tunes of the model** — the system prompt asks the
  model to refuse injected instructions; if the model is adversarial
  itself, the validator's SEC005-SEC009 rules backstop the IR boundary
  but we can't defend against a fully-compromised model.

## Threat model

The full STRIDE walkthrough lives at
[`docs/security/THREAT_MODEL.md`](docs/security/THREAT_MODEL.md). It
documents what Voce defends, what gets delegated to the user's
environment, and the explicit "out of scope" list.

## Security by design

Voce treats security as a compile-time + validate-time concern, not a
runtime one:

### Validator security rules

| Code | Rule | Severity |
|---|---|---|
| SEC001 | Protected route needs `redirect_on_fail` | Error |
| SEC002 | Mutating ActionNode must have `csrf_protected: true` | Error |
| SEC003 | Resource URL uses `http://` (use HTTPS) | Warning |
| SEC004 | Password field must declare `autocomplete` | Warning |
| SEC005 | ActionNode endpoint must be relative or HTTPS | Error |
| SEC006 | Dangerous URL scheme (`javascript:`, `vbscript:`, `data:`) in href/src | Error |
| SEC007 | External HTTP image — pin to trusted CDN or self-host | Warning |
| SEC008 | Link `target` value is not a recognized HTML keyword | Warning |
| SEC009 | `</script` substring in `properties_json` (JSON-LD breakout) | Error |

Each rule has a hint explaining the fix; many ship a JSON Patch fix
proposal callable via `voce fix`.

### Compiled output

- **Hardened CSP** — per-script SHA-256 hashes (no `'unsafe-inline'` for
  scripts), `frame-ancestors 'none'`, `base-uri 'self'`, `form-action
  'self'`. Per-IR override via `PageMetadata.content_security_policy`.
- **Zero runtime dependencies** — no framework JS shipped, eliminating
  supply-chain attack surface in the compiled output.
- **`X-Frame-Options: DENY`**, `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: strict-origin-when-cross-origin` — emitted on every
  page.

### Conversational tools

- **Prompt-injection defense** — every user message in `voce-chat` and
  `voce-mcp` is wrapped in `<user_input>` tags; the system prompt tells
  the model to treat content inside those tags as data, not instructions.
- **Session ledger is append-only** — `decisions.jsonl`,
  `drift-warnings.jsonl`, `sessions/<id>.jsonl` cannot be rewritten by
  any code path. Schema validation surfaces tampered lines without
  silently dropping them.

### Dependency hygiene

- **`cargo audit`** runs in CI on every push to main. Fails on any
  unpatched RustSec advisory.
- **`cargo deny`** runs in CI: advisories, licenses (Apache-2.0/MIT/BSD
  family allowlist), bans (`openssl` preemptively banned in favor of
  rustls), sources (only `crates.io-index` allowed).
- **SBOM** (CycloneDX format) generated per release as a downloadable
  artifact.

## Hall of fame

Reporters who helped harden Voce IR — credited at their request. *(None
yet — be the first.)*

## Supported versions

Currently the `main` branch is the supported version. Once the project
tags a v1.0 release, the most recent two minor versions will be
supported with security fixes.
