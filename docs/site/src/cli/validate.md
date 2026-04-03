# voce validate

Run the full Voce IR validation suite against a `.voce.json` file. The validator
executes 9 independent passes covering structural integrity, accessibility,
security, SEO, and more. Any failing pass causes a non-zero exit.

## Usage

```
voce validate <FILE> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to a `.voce.json` IR file (required) |

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--format <FORMAT>` | `terminal` | Output format: `terminal` (colored, human-readable) or `json` (machine-readable) |
| `--warn-as-error` | off | Treat warnings as errors, causing a non-zero exit code |

## Validation Passes

The validator runs the following passes in order:

| Pass | Code Prefix | What it checks |
|------|-------------|----------------|
| Structural | STR001--STR005 | Required fields, node completeness, document structure |
| References | REF001--REF009 | All node refs resolve, no dangling IDs, no cycles |
| State Machine | STA001--STA004 | Valid states, transitions, initial state exists |
| Accessibility | A11Y001--A11Y005 | Keyboard equivalents, heading hierarchy, alt text, form labels |
| Security | SEC001--SEC004 | CSRF on mutations, auth redirects, HTTPS enforcement, password autocomplete |
| SEO | SEO001--SEO007 | Title present, description length, single h1, OpenGraph completeness |
| Forms | FRM001--FRM009 | Fields required, labels present, unique names, email pattern validation |
| Internationalization | I18N001--I18N003 | Localized key non-empty, default value present, consistency across locales |
| Motion | MOT001--MOT005 | ReducedMotion fallback required, damping > 0, duration warnings |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All passes succeeded (no errors, no warnings or `--warn-as-error` not set) |
| `1` | One or more validation errors (or warnings when `--warn-as-error` is set) |

## Examples

Validate a file with colored terminal output:

```bash
voce validate examples/landing-page.voce.json
```

Validate and get JSON output (useful for CI or piping to `jq`):

```bash
voce validate examples/landing-page.voce.json --format json
```

Fail the build on any warning:

```bash
voce validate my-app.voce.json --warn-as-error
```

Combine with other tools:

```bash
# Validate, then compile only if valid
voce validate app.voce.json && voce compile app.voce.json
```

## JSON Output Schema

When `--format json` is used, the output is a JSON object:

```json
{
  "valid": false,
  "errors": [
    { "pass": "A11Y", "code": "A11Y003", "message": "Image node missing alt text", "node_id": "img-hero" }
  ],
  "warnings": [
    { "pass": "MOT", "code": "MOT005", "message": "Animation duration exceeds 5s", "node_id": "fade-in" }
  ]
}
```
