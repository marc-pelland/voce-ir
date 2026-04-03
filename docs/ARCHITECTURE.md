# Voce IR — Architecture

**Version:** 1.0.0
**Last updated:** 2026-04-02

---

## Overview

Voce IR is an AI-native intermediate representation for user interfaces. The architecture follows the **SPIR-V model**: binary IR + formal schema + validator + multi-target compiler. AI generates typed IR from natural language, the validator enforces quality rules, and the compiler emits optimized output.

```
Natural Language → [AI Bridge] → JSON IR → [Validator] → [Compiler] → Output
                                    ↑                         ↓
                              FlatBuffers Schema         DOM / WebGPU / WASM / Native
```

All 6 phases are complete (v1.0.0). The full pipeline is production-ready across 7 compile targets.

---

## Crate Structure

```
voce-ir/
├── packages/
│   ├── schema/              voce-schema — FlatBuffers schema + generated Rust bindings
│   ├── validator/           voce-validator — Validation engine + voce CLI binary
│   ├── compiler-dom/        voce-compiler-dom — DOM compile target (HTML/CSS/JS)
│   ├── compiler-webgpu/     voce-compiler-webgpu — WebGPU compile target (Scene3D, shaders, particles)
│   ├── compiler-wasm/       voce-compiler-wasm — WASM compile target (state machines, compute)
│   ├── compiler-hybrid/     voce-compiler-hybrid — Hybrid compiler (DOM+WebGPU+WASM, device-aware)
│   ├── compiler-ios/        voce-compiler-ios — iOS SwiftUI compile target
│   ├── compiler-android/    voce-compiler-android — Android Jetpack Compose compile target
│   ├── compiler-email/      voce-compiler-email — Email HTML compile target (table layouts, inline CSS)
│   ├── ai-bridge/           TypeScript — Multi-agent AI generation layer
│   ├── mcp-server/          TypeScript — MCP server (6 tools)
│   ├── sdk/                 TypeScript — Programmatic SDK
│   └── inspector/           TypeScript — Visual inspector & debugging tools
├── tests/
│   ├── schema/valid/        Valid IR test fixtures
│   └── schema/invalid/      Invalid IR test fixtures (one per error code)
├── examples/
│   ├── landing-page/        Reference landing page IR (37 nodes, 11 types)
│   ├── product-viewer/      3D product viewer demo (WebGPU)
│   └── intents/             Natural language → IR training pairs
└── scripts/
    └── regenerate-schema.sh FlatBuffers codegen script
```

**Rust crate dependency graph (9 crates):**
```
voce-schema ← voce-validator ← voce-compiler-dom
                    ↑      ├── voce-compiler-webgpu
              voce CLI     ├── voce-compiler-wasm
                binary     ├── voce-compiler-hybrid (depends on dom + webgpu + wasm)
                           ├── voce-compiler-ios
                           ├── voce-compiler-android
                           └── voce-compiler-email
```

**TypeScript packages:**
```
ai-bridge ── mcp-server
    ↑            ↑
    sdk      inspector
```

---

## IR Format

### FlatBuffers Schema

The IR is defined as FlatBuffers tables across 12 `.fbs` files in `packages/schema/schemas/`:

| File | Domain | Key Types |
|------|--------|-----------|
| `types.fbs` | Foundation | Color, Length, Easing (with Spring), Vec2/3/4, EdgeInsets, Shadow |
| `layout.fbs` | Scene & Layout | ViewRoot, Container, Surface, TextNode, MediaNode |
| `state.fbs` | State & Logic | StateMachine, DataNode, ComputeNode, EffectNode, ContextNode |
| `motion.fbs` | Animation | AnimationTransition, Sequence, GestureHandler, ScrollBinding, PhysicsBody, ReducedMotion |
| `navigation.fbs` | Routing | RouteMap, RouteEntry, RouteGuard, RouteTransitionConfig |
| `a11y.fbs` | Accessibility | SemanticNode, LiveRegion, FocusTrap |
| `theming.fbs` | Theming | ThemeNode, ColorPalette, TypographyScale, PersonalizationSlot, ResponsiveRule |
| `data.fbs` | Backend | ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode |
| `forms.fbs` | Forms | FormNode, FormField, ValidationRule, FormSubmission |
| `seo.fbs` | SEO | PageMetadata, OpenGraphData, StructuredData |
| `i18n.fbs` | i18n | LocalizedString, MessageCatalog, I18nConfig |
| `voce.fbs` | Master | ChildUnion (27 types), ChildNode, VoceDocument |

### Binary Format

- **Serialization:** FlatBuffers zero-copy binary with `VOCE` file identifier
- **Extension:** `.voce` (binary), `.voce.json` (JSON canonical format)
- **Schema versioning:** `schema_version_major` / `schema_version_minor` on VoceDocument

### JSON Canonical Format

The primary development format. FlatBuffers JSON with union representation:
```json
{
  "value_type": "TextNode",
  "value": { "node_id": "heading", "content": "Hello" }
}
```

The validator operates on JSON. Conversion to/from binary uses `flatc`.

### Heterogeneous Children

FlatBuffers Rust codegen doesn't support `[union]` directly. The workaround:

```flatbuffers
union ChildUnion { Container, TextNode, ... }  // 27 types
table ChildNode { value: ChildUnion; }          // wrapper
// Containers use: children: [ChildNode];
```

---

## Validation Pipeline

### Architecture

```
JSON string
    → serde_json::from_str() → VoceIr (serde model)
    → NodeIndex::build() → HashMap<node_id, NodeEntry>
    → Pass 1: structural → diagnostics
    → Pass 2: references → diagnostics
    → Pass 3: state_machine → diagnostics
    → Pass 4: accessibility → diagnostics
    → Pass 5: security → diagnostics
    → Pass 6: seo → diagnostics
    → Pass 7: forms → diagnostics
    → Pass 8: i18n → diagnostics
    → Pass 9: motion → diagnostics
    → ValidationResult { diagnostics: Vec<Diagnostic> }
```

### Why a Serde Model (Not FlatBuffers Direct)

FlatBuffers zero-copy access uses lifetime-bound references and verbose accessor chains. Multi-pass validation needs to: build indexes, walk the tree multiple times, and collect cross-references. A serde-deserializable model (`ir.rs`) provides ergonomic Rust structs for this. Fields the validator doesn't inspect use `serde_json::Value` as a passthrough.

### Pass Trait

```rust
pub trait ValidationPass {
    fn name(&self) -> &'static str;
    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult);
}
```

### Error Code Taxonomy

| Prefix | Domain | Count | Severity |
|--------|--------|-------|----------|
| STR | Structural completeness | 5 | Error |
| REF | Reference resolution | 9 | Error |
| STA | State machine validity | 4 | Error/Warning |
| A11Y | Accessibility | 5 | Error |
| SEC | Security | 4 | Error/Warning |
| SEO | Search engine optimization | 7 | Error/Warning |
| FRM | Form validation | 4 | Error/Warning |
| I18N | Internationalization | 3 | Error/Warning |
| MOT | Motion safety | 5 | Error/Warning |

---

## CLI

The `voce` binary provides 4 subcommands:

| Command | Purpose | Exit Code |
|---------|---------|-----------|
| `voce validate <file>` | Run all 9 passes, report diagnostics | 0=valid, 1=errors |
| `voce inspect <file>` | Human-readable IR summary | 0 |
| `voce json2bin <file>` | JSON → FlatBuffer binary | 0=ok, 2=error |
| `voce bin2json <file>` | FlatBuffer binary → JSON | 0=ok, 2=error |

Output modes: `--format terminal` (colored, default) or `--format json` (machine-readable).

---

## Key Decisions

1. **Rust edition 2024** for validator/compiler. **Edition 2021** for schema crate (FlatBuffers codegen compatibility).

2. **Combined FlatBuffers compilation.** The `regenerate-schema.sh` script concatenates all `.fbs` files into a single compilation unit to avoid cross-module codegen issues. Individual `.fbs` files remain the source of truth for editing.

3. **`ChildNode` wrapper table** for heterogeneous children. FlatBuffers Rust doesn't support `[union]` directly.

4. **Serde IR model** for validation, separate from FlatBuffers generated code. The generated code is the wire format; the serde model is the validation format.

5. **`node_id: string`** on every node for cross-referencing, delta updates, and accessibility tree mapping.

6. **ReducedMotion required** on all animation types. Missing ReducedMotion is a compile error (Severity::Error), not a warning.

7. **Security as part of Stability.** CSRF required on mutations, HTTPS encouraged, auth routes need redirects. These are validator rules, not optional configuration.

8. **`flatc` subprocess** for json2bin/bin2json conversion. A programmatic Rust serialization bridge is planned for Phase 2.

---

## Testing Strategy

- **Schema tests** (`packages/schema/src/lib.rs`): FlatBufferBuilder round-trips for each node type domain. 12 tests.
- **Validator integration tests** (`packages/validator/tests/validation_tests.rs`): Load fixtures, run validation, assert error codes. 24 tests.
- **Invalid IR fixtures** (`tests/schema/invalid/`): One fixture per error code category. 12 files.
- **Valid IR fixtures** (`tests/schema/valid/`): Minimal page and landing page. 1 file.
- **Example validation**: Landing page and intent-IR pairs validate cleanly.

Total: 37 tests across the workspace.

---

## Compile Targets

### DOM (compiler-dom)
IR → single-file HTML with inline CSS, surgical JS, ARIA attributes, CSP headers. 6.6KB output for reference landing page. Zero framework runtime.

### WebGPU (compiler-webgpu)
Scene3D, MeshNode, ShaderNode (WGSL transpilation), ParticleSystem (GPU compute shaders). PBR material system, camera controls, lighting pipeline.

### WASM (compiler-wasm)
StateMachine → WASM functions, ComputeNode → WASM pure functions. JS interop bridge via wasm-bindgen. Used for compute-heavy operations.

### Hybrid (compiler-hybrid)
Per-component target analysis: DOM for layout/text, WebGPU for 3D/particles, WASM for compute. Device profile matching selects targets automatically. Unified output bundling. Graceful degradation: WebGPU → Canvas 2D → static image fallback chain.

### iOS (compiler-ios)
IR → SwiftUI view hierarchy. SemanticNode → VoiceOver accessibility modifiers. GestureHandler → SwiftUI gesture recognizers. Responsive layout via GeometryReader.

### Android (compiler-android)
IR → Jetpack Compose Composable functions. SemanticNode → Compose semantics (TalkBack). ThemeNode → Material Design tokens. GestureHandler → Compose pointer input.

### Email (compiler-email)
IR → email-safe HTML: table-based layouts, inline CSS, Outlook conditional comments, Gmail CSS resets. Cross-client preview rendering. Responsive via media query fallbacks.

---

## Visual Inspector

The inspector (packages/inspector) is a TypeScript runtime debugging tool that injects into compiled Voce output:

- **Scene graph overlay:** click-to-inspect element selection, IR node property panel
- **State machine visualizer:** live current state, transition history, guard evaluation
- **Animation timeline:** pause, scrub, step frame-by-frame
- **Accessibility auditor:** live a11y tree view, focus order visualization, tab-through simulator
- **Performance profiler:** frame timing, GPU utilization, per-node render cost
- **CMS visual editing:** content click-to-edit, CMS bridge protocol, preview/publish
- **Conversational debugging:** describe a bug in natural language, AI traces and proposes a patch
- **Extension API:** community plugins for custom inspector panels

---

See `docs/plans/MASTER_PLAN.md` for the full sprint breakdown (all 50 sprints complete).
