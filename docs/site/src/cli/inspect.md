# voce inspect

Display a human-readable summary of a Voce IR file. Shows node counts,
type distribution, state machines, and feature usage without compiling
or validating.

## Usage

```
voce inspect <FILE>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<FILE>` | Path to a `.voce.json` IR file (required) |

## Output

The inspect command prints a structured summary to stdout. There are no
output format options -- the output is always a human-readable table.

### Summary Sections

**Document overview** -- total node count, document-level metadata (title,
locale, auth configuration).

**Node type distribution** -- count of each node type present in the IR
(e.g., Container: 5, TextNode: 12, MediaNode: 3).

**State machines** -- names, state counts, and transition counts for each
StateMachine node.

**Features detected** -- which optional IR features are in use:
animations, forms, navigation/routing, i18n, SEO metadata, theming,
accessibility annotations, data/backend bindings.

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | IR file parsed and summary printed |
| `1` | File could not be read or parsed as valid JSON |

## Examples

Inspect a landing page IR:

```bash
voce inspect examples/landing-page.voce.json
```

Example output:

```
Voce IR Summary
===============

Document: landing-page
Nodes:    37
Types:    11 distinct

Node Distribution:
  Container      8
  Surface        4
  TextNode      12
  MediaNode      3
  FormNode       1
  FormField      4
  StateMachine   1
  SemanticNode   2
  PageMetadata   1
  AnimationTransition 1

State Machines:
  form-states    3 states, 4 transitions

Features:
  [x] Accessibility
  [x] Forms
  [x] SEO metadata
  [x] Animation
  [ ] Navigation
  [ ] Internationalization
  [ ] Theming
  [ ] Data bindings
```

## Use Cases

- **Quick audit** -- understand what an IR file contains before validating
  or compiling it.
- **CI reporting** -- log IR complexity metrics alongside build output.
- **Debugging** -- verify that AI-generated IR includes the expected node
  types and features.
- **Diffing** -- compare inspect output across versions to spot structural
  changes.

## Notes

The inspect command does not validate the IR. A file with structural errors
can still be inspected as long as it is parseable JSON. To check correctness,
use [`voce validate`](./validate.md).
