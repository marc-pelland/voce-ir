# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in Voce IR, please report it responsibly.

**Do not open a public GitHub issue for security vulnerabilities.**

Instead, please email **marc@voce-ir.xyz** with:

- A description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

You should receive a response within 48 hours. I'll work with you to understand the issue and coordinate a fix before any public disclosure.

## Scope

Security issues in the following areas are in scope:

- **Compiled output**: XSS, injection, or other vulnerabilities in HTML/JS/CSS emitted by the compilers
- **Validator bypass**: IR that passes validation but produces insecure output
- **CLI**: Command injection or path traversal in the `voce` CLI
- **WASM playground**: Sandbox escapes or data exfiltration

## Security by Design

Voce IR treats security as a compile-time concern:

- CSP headers are emitted automatically in all DOM output
- CSRF tokens are required on all mutation actions (enforced by validator SEC002)
- HTTPS is encouraged for all external URLs (SEC003)
- Auth routes require redirect configuration (SEC001)
- Compiled output has zero runtime dependencies, eliminating supply chain risk
