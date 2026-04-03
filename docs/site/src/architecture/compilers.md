# Compiler Architecture

Voce IR supports seven compile targets. Each compiler lives in its own Rust
crate under `packages/`, reads validated IR, and emits platform-specific output
with zero runtime dependencies. The compiler selection happens at build time --
the same IR can be compiled to any supported target without modification.

## Compiler Crates

| Crate              | Target    | Output Format                    |
|--------------------|-----------|----------------------------------|
| `compiler-dom`     | DOM       | Single-file HTML + CSS + JS      |
| `compiler-webgpu`  | WebGPU    | WGSL shaders + JS harness        |
| `compiler-wasm`    | WASM      | WAT text format / WASM binary    |
| `compiler-hybrid`  | Hybrid    | Mixed targets per component      |
| `compiler-ios`     | iOS       | SwiftUI view files               |
| `compiler-android` | Android   | Jetpack Compose Kotlin           |
| `compiler-email`   | Email     | Table-based HTML                 |

## DOM Compiler

The DOM compiler (`packages/compiler-dom/`) is the primary compile target and
the most mature. It emits a single self-contained HTML file with inlined CSS
and JavaScript. No framework, no bundler, no CDN dependencies.

Internal pipeline stages:
1. **Lower** -- Transform IR nodes into a compiler-internal representation
   (`compiler_ir.rs`) optimized for code generation
2. **Animation** -- Process motion nodes into CSS keyframes and JS animation
   code
3. **Assets** -- Resolve and inline media references
4. **Emit** -- Generate the final HTML string with embedded styles and scripts

The output follows patterns from SolidJS and Svelte compiled output: surgical
DOM mutations rather than virtual DOM diffing. State changes produce direct
`element.textContent = value` assignments, not tree reconciliation.

## WebGPU Compiler

The WebGPU compiler targets GPU-accelerated rendering using the WebGPU API.
It produces WGSL (WebGPU Shading Language) shader programs alongside a
JavaScript harness that manages the render pipeline.

Key capabilities:
- PBR (Physically Based Rendering) material support
- Scene3D, MeshNode, and ShaderNode compilation
- Particle system emission as compute shaders
- Automatic fallback annotations for non-WebGPU browsers

## WASM Compiler

The WASM compiler translates StateMachine and ComputeNode logic into WebAssembly
Text Format (WAT), which can then be assembled into `.wasm` binaries. This
target is used when state logic needs to run at near-native speed in the browser.

The compiler maps Voce state machines to WASM function tables: each state
becomes a function, transitions become conditional branches, and data bindings
become memory load/store operations.

## Hybrid Compiler

The hybrid compiler performs per-component target analysis. Rather than
compiling the entire document to one target, it examines each subtree and
selects the optimal compiler:

- Static content with no interactivity routes to DOM (minimal output)
- Heavy animation or 3D content routes to WebGPU
- Complex state logic routes to WASM
- The final output stitches the pieces together with a thin coordination layer

This allows a single page to mix GPU-rendered hero sections with lightweight
DOM content sections, optimizing both performance and payload size.

## iOS Compiler

The iOS compiler emits SwiftUI view code. IR layout nodes map to SwiftUI's
`VStack`, `HStack`, `ZStack`, and `LazyVGrid`. Theming tokens become SwiftUI
`Color` and `Font` definitions. Navigation maps to `NavigationStack` and
`NavigationLink`.

Accessibility semantics translate directly -- Voce's `SemanticNode` maps to
SwiftUI's `.accessibilityLabel`, `.accessibilityHint`, and role modifiers.

## Android Compiler

The Android compiler targets Jetpack Compose, emitting Kotlin composable
functions. IR containers become `Column`, `Row`, and `Box` composables.
Theming maps to Material 3 `MaterialTheme` with custom color schemes generated
from the IR's `ThemeNode`.

State machines compile to Compose `State` holders with `LaunchedEffect` for
side effects, matching the IR's reactive model.

## Email Compiler

The email compiler produces HTML that renders correctly across email clients --
a notoriously constrained environment. It uses table-based layouts (not flexbox
or grid), inline styles (not CSS classes), and conservative markup that passes
Litmus and Email on Acid testing.

Key constraints the compiler handles:
- All layout via nested `<table>` elements
- All styles inlined on each element
- No JavaScript (email clients strip it)
- Image references as absolute URLs (no inlining)
- MSO conditional comments for Outlook compatibility

## Shared Architecture

All seven compilers share common patterns:

- **Input:** Validated `VoceIr` (the serde model, not raw FlatBuffers)
- **Output:** A string or file bundle representing the compiled artifact
- **No runtime dependencies:** Every compiler produces self-contained output
- **Snapshot testing:** Compiler output is tested with `insta` snapshots to
  catch regressions
- **Accessibility preservation:** Semantic information from the IR must appear
  in the compiled output -- compilers cannot silently drop it

The compiler selection is exposed through the CLI:

```bash
voce compile input.voce.json --target dom -o output.html
voce compile input.voce.json --target ios -o OutputView.swift
voce compile input.voce.json --target email -o newsletter.html
```
