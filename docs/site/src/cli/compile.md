# voce compile

Compile a validated Voce IR file into a single-file HTML document. The compiler
runs the full validation suite first -- if validation fails, compilation is
aborted and errors are printed.

## Usage

```
voce compile <FILE> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to a `.voce.json` IR file (required) |

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-o, --output <PATH>` | `dist/<stem>.html` | Output file path. If omitted, derives from the input filename. |
| `--debug` | off | Add `data-voce-id` attributes to every emitted DOM element, mapping each back to its IR node ID. |

## How It Works

1. **Validate** -- all 9 validation passes run. Any error aborts compilation.
2. **Resolve layout** -- Taffy computes flexbox/grid geometry at compile time.
3. **Emit HTML** -- a single self-contained `.html` file is written. All styles
   are inlined. There are zero runtime dependencies.

The compiled output follows the SolidJS/Svelte pattern of surgical DOM
construction -- no virtual DOM, no framework runtime.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Compilation succeeded, output file written |
| `1` | Validation failed (errors printed to stderr) |
| `2` | Compilation error (e.g., unsupported node type, I/O failure) |

## Examples

Compile with default output path:

```bash
voce compile examples/landing-page.voce.json
# writes dist/landing-page.html
```

Compile to a specific output file:

```bash
voce compile app.voce.json -o build/index.html
```

Compile with debug attributes for development:

```bash
voce compile app.voce.json --debug
# Each element gets: <div data-voce-id="container-main">...</div>
```

Validate and compile in sequence:

```bash
voce validate app.voce.json --format json && voce compile app.voce.json
```

## Debug Mode

When `--debug` is passed, every emitted HTML element receives a
`data-voce-id` attribute containing the IR node ID it was generated from.
This is useful for:

- Tracing compiled output back to the source IR
- Browser DevTools inspection
- Integration with `voce inspect` for cross-referencing

Do not ship debug builds to production -- the extra attributes increase file
size and expose internal structure.

## Output Format

The compiled HTML file is fully self-contained:

- Inline `<style>` block with all computed styles
- Inline `<script>` block for state machines and event handlers (if present)
- No external dependencies, CDN links, or network requests
- Valid HTML5 with lang attribute and semantic structure
