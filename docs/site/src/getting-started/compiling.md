# Compiling to HTML

Once you have a valid `.voce.json` file, the Voce compiler transforms it into production-ready output. This guide covers the DOM compile target, which produces a single-file HTML page.

## Validate first

Always validate before compiling. The compiler assumes valid input:

```bash
voce validate hello.voce.json
```

```
hello.voce.json: VALID (2 nodes, 0 warnings)
```

## Compile

Run the compile command:

```bash
voce compile hello.voce.json
```

By default this writes output to the `dist/` directory:

```
Compiling hello.voce.json → dist/hello.html
  2 nodes compiled
  Output: dist/hello.html (4.2 KB)
```

## What the compiler produces

The output is a **single self-contained HTML file**. Open `dist/hello.html` in any browser and you will see your rendered page.

Key properties of the compiled output:

- **Zero runtime dependencies.** No JavaScript frameworks, no CSS libraries, no CDN links. The HTML file contains everything it needs inline.
- **Semantic markup.** A `TextNode` with `heading_level: 1` becomes an `<h1>`. Containers become appropriate structural elements. Accessibility attributes are wired automatically from `SemanticNode` references.
- **Surgical DOM updates.** When the IR includes state machines, the compiler emits minimal JavaScript that performs targeted DOM mutations -- similar to what Svelte or SolidJS produce, but generated from binary IR rather than a component DSL.
- **No supply chain risk.** Because there are zero external dependencies in the output, there is no attack surface from third-party packages.

## Output options

### Custom output directory

```bash
voce compile hello.voce.json --output ./build
```

### JSON output mode

For debugging, you can emit a JSON representation of the compile plan:

```bash
voce compile hello.voce.json --format json
```

## Preview in the browser

The `preview` command compiles and immediately opens the result in your default browser:

```bash
voce preview hello.voce.json
```

This is the fastest way to iterate. It compiles to a temporary directory and launches the browser in one step.

## Compile targets

The DOM/HTML target is the default. Voce supports multiple compile targets:

| Target | Flag | Output |
|--------|------|--------|
| DOM (HTML) | `--target dom` | Single-file `.html` |
| WebGPU | `--target webgpu` | HTML + WebGPU rendering |
| WASM | `--target wasm` | WebAssembly module |
| Hybrid | `--target hybrid` | DOM for content, WebGPU for effects |
| iOS (SwiftUI) | `--target ios` | Swift source files |
| Android (Compose) | `--target android` | Kotlin source files |
| Email | `--target email` | Email-safe HTML |

For example, to compile for email:

```bash
voce compile hello.voce.json --target email
```

## Inspecting the IR

Before compiling, you can get a quick summary of what is in your IR file:

```bash
voce inspect hello.voce.json
```

```
Document: Hello World
Schema:   v1.0
Nodes:    2 (1 TextNode, 1 ViewRoot)
Language: en
Warnings: 0
```

## A complete workflow

Putting it all together, a typical workflow looks like this:

```bash
# 1. Create or generate the IR file
#    (by hand, via AI, or from an existing tool)

# 2. Validate
voce validate my-page.voce.json

# 3. Compile
voce compile my-page.voce.json

# 4. Preview
voce preview my-page.voce.json

# 5. Deploy the output
#    dist/my-page.html is a static file -- serve it from anywhere
```

## Next steps

Writing IR by hand works for learning, but Voce is designed for AI authorship. Continue to [AI Generation](./ai-generation.md) to learn how to generate IR from natural language.
