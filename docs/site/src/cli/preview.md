# voce preview

Compile a Voce IR file and open the result in the default web browser.
A convenience command that combines `voce compile` with a platform-aware
browser launch.

## Usage

```
voce preview <FILE>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to a `.voce.json` IR file (required) |

## How It Works

1. **Validate** -- runs the full 9-pass validation suite.
2. **Compile** -- produces a self-contained HTML file in a temporary
   location (or `dist/` if it exists).
3. **Open** -- launches the compiled HTML in the default browser using
   the platform-appropriate command.

### Platform Commands

| Platform | Command used |
|----------|-------------|
| macOS    | `open <file>` |
| Linux    | `xdg-open <file>` |
| Windows  | `start <file>` |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | File compiled and browser launch initiated |
| `1` | Validation failed |
| `2` | Compilation failed |
| `3` | Browser could not be opened |

## Examples

Preview a landing page:

```bash
voce preview examples/landing-page.voce.json
```

Preview after making changes to the IR:

```bash
# Edit the IR (or regenerate via AI), then preview
voce preview app.voce.json
```

Chain with validation for verbose feedback:

```bash
voce validate app.voce.json && voce preview app.voce.json
```

## Comparison with compile

| | `voce compile` | `voce preview` |
|---|----------------|----------------|
| Validates | Yes | Yes |
| Writes HTML | To `-o` path or `dist/` | To temporary or `dist/` |
| Opens browser | No | Yes |
| Debug attributes | `--debug` flag | Not available |
| Custom output path | `-o` flag | Not available |

For production builds, use `voce compile`. For quick iteration during
development, use `voce preview`.

## Notes

- The preview command always compiles a fresh copy. It does not cache
  previous builds.
- Debug attributes (`data-voce-id`) are not included in preview builds.
  Use `voce compile --debug` if you need them, then open the output
  file manually.
- If no default browser is configured on Linux, `xdg-open` may fail
  silently. Set the `BROWSER` environment variable as a fallback.
- The compiled HTML is fully self-contained with no external dependencies,
  so it renders correctly when opened as a local file.
