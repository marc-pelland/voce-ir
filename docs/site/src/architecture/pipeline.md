# Pipeline Overview

Voce IR follows a SPIR-V-inspired pipeline: a binary intermediate representation
flows through validation and compilation stages before reaching the end user.
No human-readable source code exists in the pipeline. The AI generates IR
directly, the validator enforces correctness, and the compiler emits optimized
output for each target platform.

## Pipeline Stages

```
Natural Language
       |
       v
 +-----------+
 | AI Bridge |   LLM generates JSON IR from conversation
 +-----------+
       |
       v
 +-----------+
 | JSON IR   |   Machine-readable text (.voce.json)
 +-----------+
       |
       v
 +-----------+
 | Validator |   9 passes, 46 rules — errors block compilation
 +-----------+
       |
       v
 +-----------+
 | Compiler  |   7 targets — DOM, WebGPU, WASM, Hybrid, iOS, Android, Email
 +-----------+
       |
       v
 +-----------+
 | Deployer  |   4 adapters — Static, Cloudflare, Netlify, Vercel
 +-----------+
       |
       v
   End User
```

## Stage 1: AI Bridge

The AI bridge (`packages/ai-bridge/`) is a TypeScript layer that sits between
the LLM and the rest of the pipeline. It manages the conversation, applies
style packs, and produces valid JSON IR. The bridge uses structured output
to ensure the LLM emits well-formed IR conforming to the FlatBuffers schema.

Key responsibilities:
- Conversation management (anti-vibe-coding: the AI asks questions, pushes back)
- Style pack selection and token injection
- Schema-aware JSON generation
- Intent-IR pair matching via RAG

## Stage 2: JSON IR

The JSON representation is the canonical text form of the binary IR. It is not
source code -- it is a machine-readable serialization used for AI generation,
debugging, and version control diffing. Files use the `.voce.json` extension.
The `voce json2bin` command converts JSON to the binary FlatBuffers format
(`.voce`), and `voce bin2json` reverses the process.

## Stage 3: Validator

The validator (`packages/validator/`) runs 9 ordered passes over the IR,
checking 46 rules across structural integrity, reference resolution, state
machines, accessibility, security, SEO, forms, internationalization, and
motion safety. Validation errors block compilation entirely -- there is no
"build with warnings" mode for critical rules.

Passes execute in dependency order:
1. Structural (STR) -- document shape, required fields, node nesting
2. References (REF) -- all ID references resolve to existing nodes
3. State Machine (STA) -- valid transitions, initial states, no orphans
4. Accessibility (A11Y) -- keyboard equivalents, heading hierarchy, alt text
5. Security (SEC) -- CSRF on mutations, auth redirects, HTTPS enforcement
6. SEO -- title, description, h1 count, Open Graph completeness
7. Forms (FRM) -- field labels, unique names, validation rules
8. Internationalization (I18N) -- localized key presence, default values
9. Motion (MOT) -- ReducedMotion required, physics constraints, duration limits

## Stage 4: Compiler

Seven compile targets live in separate crates under `packages/`. Each compiler
reads validated IR and emits platform-specific output with zero runtime
dependencies:

| Target   | Crate              | Output                        |
|----------|--------------------|-------------------------------|
| DOM      | `compiler-dom`     | Single-file HTML              |
| WebGPU   | `compiler-webgpu`  | WGSL shaders + JS harness     |
| WASM     | `compiler-wasm`    | WAT/WASM modules              |
| Hybrid   | `compiler-hybrid`  | Per-component target analysis  |
| iOS      | `compiler-ios`     | SwiftUI views                 |
| Android  | `compiler-android` | Jetpack Compose functions      |
| Email    | `compiler-email`   | Table-based HTML              |

## Stage 5: Deploy

Four deploy adapters handle the last mile, packaging compiler output for
specific hosting environments:

| Adapter    | Crate                | Description                          |
|------------|----------------------|--------------------------------------|
| Static     | `adapter-static`     | Plain files, any static host         |
| Cloudflare | `adapter-cloudflare` | Cloudflare Workers / Pages           |
| Netlify    | `adapter-netlify`    | Netlify Functions + deploy config    |
| Vercel     | `adapter-vercel`     | Vercel serverless + edge config      |

## Design Principles

- **No human-readable code in the pipeline.** The IR is the source of truth,
  not a stepping stone to hand-editable files.
- **Accessibility is a compile error.** Missing semantic information blocks
  the build, not just produces a warning.
- **Zero runtime dependencies.** Compiled output has no npm packages, no CDN
  links, no framework bundles. This eliminates the supply chain attack surface.
- **Binary IR is not human-readable by design.** JSON exists for AI generation
  and debugging, not for human authorship.
