# Validation Passes

The Voce validator runs 9 ordered passes over the IR, enforcing 46 rules that
span structural correctness, accessibility, security, and more. Validation
errors block compilation -- there is no way to skip or suppress critical
failures. This is by design: accessibility and security are compile errors,
not warnings.

## The ValidationPass Trait

Every pass implements the `ValidationPass` trait defined in
`packages/validator/src/passes/mod.rs`:

```rust
pub trait ValidationPass {
    fn name(&self) -> &'static str;
    fn run(&self, ir: &VoceIr, index: &NodeIndex, result: &mut ValidationResult);
}
```

The `VoceIr` is a serde-based IR model, separate from the FlatBuffers generated
types. This decoupling is intentional -- the validator works with JSON-
deserialized data, not raw FlatBuffers buffers. The `NodeIndex` provides
pre-built lookup tables (ID-to-node maps, parent chains) so passes can resolve
references without redundant traversals.

Passes execute in dependency order. Structural checks run first because later
passes assume the document shape is valid. Reference resolution runs second
because domain passes may follow reference chains.

## Error Code Taxonomy

Each rule has a unique error code with a prefix identifying its pass:

### STR -- Structural (5 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| STR001 | Document must have a root ViewRoot                    |
| STR002 | All nodes must have a non-empty `id` field            |
| STR003 | Node IDs must be unique within the document           |
| STR004 | Children must be valid for their parent node type     |
| STR005 | Required fields must be present (per schema)          |

### REF -- References (9 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| REF001 | `target_id` references must resolve to existing nodes |
| REF002 | State machine `initial_state` must name a valid state |
| REF003 | Transition targets must name valid states              |
| REF004 | Animation `target_id` must resolve                    |
| REF005 | Route guard `redirect` must name a valid route        |
| REF006 | Context `provider_id` must resolve                    |
| REF007 | Subscription `source_id` must resolve                 |
| REF008 | Form field `form_id` must resolve to a FormNode       |
| REF009 | Scroll binding `source_id` must resolve               |

### STA -- State Machine (4 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| STA001 | State machine must have at least one state            |
| STA002 | Initial state must exist in the state list             |
| STA003 | All transition targets must be reachable states       |
| STA004 | No orphan states (every state reachable from initial) |

### A11Y -- Accessibility (5 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| A11Y001| Interactive elements must have keyboard equivalents   |
| A11Y002| Heading levels must not skip (h1 -> h3 without h2)   |
| A11Y003| Images must have alt text (or `decorative: true`)     |
| A11Y004| Form fields must have associated labels               |
| A11Y005| Focus traps must have an escape mechanism             |

### SEC -- Security (4 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| SEC001 | Mutation actions must include CSRF protection         |
| SEC002 | Auth-guarded routes must specify a redirect           |
| SEC003 | External URLs must use HTTPS                          |
| SEC004 | Password fields must have `autocomplete` attribute    |

### SEO (7 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| SEO001 | Page must have a `<title>` (via PageMetadata)         |
| SEO002 | Title length must be 10-60 characters                 |
| SEO003 | Meta description must be present                      |
| SEO004 | Description length must be 50-160 characters          |
| SEO005 | Exactly one h1 heading per page                       |
| SEO006 | Open Graph data must include title, description, image|
| SEO007 | Structured data must have `@type` field               |

### FRM -- Forms (4 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| FRM001 | Form must have at least one field                     |
| FRM002 | Every field must have a label                         |
| FRM003 | Field names must be unique within their form          |
| FRM004 | Email fields must have email validation               |

### I18N -- Internationalization (3 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| I18N001| Localized string keys must be non-empty               |
| I18N002| Every localized string must have a default value      |
| I18N003| All locales must have consistent key sets             |

### MOT -- Motion (5 rules)

| Code   | Rule                                                  |
|--------|-------------------------------------------------------|
| MOT001 | Animations must specify a `ReducedMotion` alternative |
| MOT002 | Physics bodies must have damping > 0                  |
| MOT003 | Animation duration should not exceed 10 seconds       |
| MOT004 | Sequences must have at least one step                 |
| MOT005 | Gesture handlers must specify a recognized gesture    |

## Output Formats

The validator reports diagnostics in two formats, controlled by CLI flags:

- **Colored terminal output** (default) -- human-readable with error codes,
  node paths, and descriptions
- **JSON output** (`--format json`) -- machine-readable array of diagnostics
  for integration with CI pipelines and editor tooling

```bash
voce validate my-page.voce.json              # colored terminal output
voce validate my-page.voce.json --format json # JSON diagnostics
```

## Serde IR Model

The validator does not read FlatBuffers binary directly. Instead, it
deserializes JSON into a parallel serde-based IR model defined in
`packages/validator/src/ir.rs`. This model mirrors the FlatBuffers schema
but uses standard Rust types (`String`, `Vec`, `Option`) rather than
FlatBuffers accessors. The separation keeps validation logic clean and
testable without requiring binary serialization in test fixtures.
