# Voce IR

**AI-native intermediate representation for user interfaces.**

*The code is gone. The experience remains.*

Voce IR is a binary IR format, like SPIR-V for graphics but for UI. AI generates typed IR from natural language, a validator enforces quality rules, and a compiler emits optimized output across 7 targets:

| Target | Output |
|--------|--------|
| **DOM** | Single-file HTML with inline CSS, zero-dependency JS, ARIA attributes |
| **WebGPU** | WGSL shaders, PBR materials, particle systems |
| **WASM** | State machines and compute as WebAssembly functions |
| **Hybrid** | Per-component target analysis (DOM + WebGPU + WASM) |
| **iOS** | SwiftUI views with VoiceOver accessibility |
| **Android** | Jetpack Compose with Material Design and TalkBack |
| **Email** | Table-based HTML with inline CSS for cross-client support |

## Pipeline

```
Natural Language
    → [AI Bridge] → JSON IR
    → [Validator] → 9 quality passes (46 rules)
    → [Compiler] → Optimized output
    → [Deploy]   → Vercel / Cloudflare / Netlify / Static
```

## Quick Start

```bash
# Install
cargo install voce-validator

# Validate an IR file
voce validate my-page.voce.json

# Compile to HTML
voce compile my-page.voce.json

# Deploy
voce deploy my-page.voce.json --adapter static
```

## Three Pillars

Every design decision in Voce IR is anchored to three non-negotiable pillars:

1. **Stability** — Security is a compile error, not a configuration option. CSRF required on mutations, HTTPS enforced, auth routes need redirects.

2. **Experience** — Zero-runtime output. No framework JS shipped. Spring physics solved at compile time. Every byte serves the user.

3. **Accessibility** — Missing `SemanticNode` is a validation error, not a warning. Keyboard equivalents required on every gesture. Heading hierarchy enforced.

## Learn More

- [Getting Started](./getting-started/installation.md) — install and compile your first IR
- [Schema Reference](./schema/overview.md) — every node type documented
- [Architecture](./architecture/pipeline.md) — how the pipeline works
- [Contributing](./guides/contributing.md) — help build Voce IR
