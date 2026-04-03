# IR Format

Voce IR uses [FlatBuffers](https://flatbuffers.dev/) as its binary wire format.
FlatBuffers provide zero-copy deserialization, schema evolution, and compact
binary encoding -- properties borrowed from GPU shader pipelines (SPIR-V) rather
than traditional web frameworks.

## File Extensions

| Extension     | Format         | Purpose                                    |
|---------------|----------------|--------------------------------------------|
| `.voce`       | Binary         | FlatBuffers binary, used at compile time    |
| `.voce.json`  | JSON           | Canonical text representation for AI, debug |

The two formats are interchangeable. The CLI provides round-trip conversion:

```bash
voce json2bin input.voce.json -o output.voce
voce bin2json input.voce -o output.voce.json
```

Internally, both commands delegate to `flatc` (the FlatBuffers compiler) with
the Voce schema. The JSON form is what AI models emit; the binary form is what
the validator and compilers consume.

## Schema Organization

FlatBuffers schemas live in `packages/schema/schemas/`. Each `.fbs` file covers
one domain:

| File              | Domain                                      |
|-------------------|---------------------------------------------|
| `types.fbs`       | Primitive types (RGB, Edge, Dimension, etc.) |
| `layout.fbs`      | ViewRoot, Container, Surface, TextNode, MediaNode |
| `state.fbs`       | StateMachine, DataNode, ComputeNode, EffectNode, ContextNode |
| `motion.fbs`      | AnimationTransition, Sequence, GestureHandler, ScrollBinding, PhysicsBody |
| `navigation.fbs`  | RouteMap, RouteEntry, RouteTransition, RouteGuard |
| `a11y.fbs`        | SemanticNode, LiveRegion, FocusTrap          |
| `theming.fbs`     | ThemeNode, ColorPalette, TypographyScale, SpacingScale, PersonalizationSlot, ResponsiveRule |
| `data.fbs`        | ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode |
| `forms.fbs`       | FormNode, FormField, ValidationRule, FormSubmission |
| `seo.fbs`         | PageMetadata, OpenGraphData, StructuredData  |
| `i18n.fbs`        | LocalizedString, MessageCatalog, FormatOptions, I18nConfig |
| `voce.fbs`        | Master file -- ChildUnion, ChildNode, VoceDocument |

The master file `voce.fbs` includes all domain files and defines the document
root. This is the single compilation target for `flatc`.

## The ChildUnion Wrapper Pattern

FlatBuffers unions allow heterogeneous node types in a single tree. However,
the Rust codegen does not support vectors of unions directly. Voce solves this
with a wrapper table:

```flatbuffers
union ChildUnion {
  Container,
  Surface,
  TextNode,
  MediaNode,
  StateMachine,
  // ... 27 total node types
  FormNode
}

table ChildNode {
  value: ChildUnion;
}
```

Parent nodes store children as `[ChildNode]` -- a vector of wrapper tables,
each containing one union variant. This pattern adds one level of indirection
but preserves type safety and allows the full 27-type union to appear anywhere
in the tree.

## VoceDocument Root

Every IR file has a `VoceDocument` at its root:

```flatbuffers
table VoceDocument {
  schema_version_major: int32 = 0;
  schema_version_minor: int32 = 1;
  root: ViewRoot (required);
  routes: RouteMap;
  theme: ThemeNode;
  alternate_themes: [ThemeNode];
  auth: AuthContextNode;
  i18n: I18nConfig;
}
```

The `root` field is the visual tree entry point (always a `ViewRoot`).
Top-level configuration -- routing, theming, authentication, and
internationalization -- lives alongside the root rather than nested inside it.

The binary file uses the FlatBuffers file identifier `"VOCE"` (4 bytes at
offset 4), enabling quick format detection without parsing.

## Schema Versioning

The `schema_version_major` and `schema_version_minor` fields follow semver
conventions:

- **Minor bump:** New optional fields, new union members. Old validators and
  compilers can still read the IR (FlatBuffers forward-compatibility).
- **Major bump:** Removed fields, changed semantics, breaking structural
  changes. Requires matching validator/compiler versions.

FlatBuffers' wire format naturally supports forward compatibility -- unknown
fields are silently ignored by older readers. This means minor version bumps
require no coordination between the AI bridge and the compiler.

## JSON Canonical Form

The JSON representation mirrors the FlatBuffers schema exactly. Field names
match the schema, enums use string names, and nested tables become nested
objects. This is not a separate format -- it is the standard FlatBuffers JSON
encoding produced by `flatc --json`.

A minimal valid document in JSON:

```json
{
  "schema_version_major": 0,
  "schema_version_minor": 1,
  "root": {
    "id": "root",
    "children": []
  }
}
```

The JSON form exists for three reasons: AI generation (LLMs produce text, not
binary), debugging (humans can inspect the IR during development), and version
control (text diffs are meaningful, binary diffs are not). It is never the
primary runtime format.
