# Schema Overview

Voce IR uses [FlatBuffers](https://google.github.io/flatbuffers/) as its binary serialization format. Every UI is represented as a `VoceDocument` containing a tree of typed nodes spanning 11 domains: layout, state, motion, navigation, accessibility, theming, data, forms, SEO, and i18n.

## Binary Format

FlatBuffers provides zero-copy deserialization, schema evolution with forward/backward compatibility, and a compact binary representation. Voce IR files use the `VOCE` file identifier and the `.voce` extension.

The binary format is immutable by design. Runtime mutable state lives in a separate reactive layer managed by the compiler output, not in the buffer itself.

## JSON Canonical Representation

Every Voce IR binary round-trips losslessly to and from JSON. The JSON representation serves as:

- **AI generation target** -- LLMs emit JSON, which is then compiled to binary
- **Debugging format** -- human-inspectable when needed
- **Version control diffing** -- text diffs for review workflows
- **Escape hatch** -- interop with tools that cannot read FlatBuffers

The JSON form is not intended for hand-authoring. It is a machine-readable text serialization of the IR.

## ChildUnion Pattern

FlatBuffers does not support vectors of unions directly in Rust codegen. Voce IR works around this with a wrapper table:

```json
{
  "children": [
    { "value_type": "Container", "value": { "node_id": "c1", "layout": "Flex" } },
    { "value_type": "TextNode", "value": { "node_id": "t1", "content": "Hello" } }
  ]
}
```

The `ChildUnion` is a union of all 27 node types across all domains. Each child entry is wrapped in a `ChildNode` table containing a single `value` field of type `ChildUnion`.

### ChildUnion Members

| Domain       | Node Types                                                         |
|--------------|--------------------------------------------------------------------|
| Layout       | Container, Surface, TextNode, MediaNode                            |
| State        | StateMachine, DataNode, ComputeNode, EffectNode, ContextNode       |
| Motion       | AnimationTransition, Sequence, GestureHandler, ScrollBinding, PhysicsBody |
| Navigation   | RouteMap                                                           |
| Accessibility| SemanticNode, LiveRegion, FocusTrap                                |
| Theming      | ThemeNode, PersonalizationSlot, ResponsiveRule                     |
| Data         | ActionNode, SubscriptionNode, AuthContextNode, ContentSlot, RichTextNode |
| Forms        | FormNode                                                           |

## VoceDocument

The root table of every Voce IR file.

| Field                  | Type             | Required | Description                                      |
|------------------------|------------------|----------|--------------------------------------------------|
| schema_version_major   | int32            | no       | Major schema version (default 0)                 |
| schema_version_minor   | int32            | no       | Minor schema version (default 1)                 |
| root                   | ViewRoot         | yes      | Top-level view root for the document             |
| routes                 | RouteMap         | no       | Application-level route map for multi-route apps |
| theme                  | ThemeNode        | no       | Primary theme                                    |
| alternate_themes       | [ThemeNode]      | no       | Additional themes (dark, high-contrast, etc.)    |
| auth                   | AuthContextNode  | no       | Application-level auth configuration             |
| i18n                   | I18nConfig       | no       | Internationalization configuration               |

### Minimal Example

```json
{
  "schema_version_major": 0,
  "schema_version_minor": 1,
  "root": {
    "node_id": "root",
    "document_language": "en",
    "children": [
      {
        "value_type": "TextNode",
        "value": {
          "node_id": "greeting",
          "content": "Hello, world",
          "heading_level": 1
        }
      }
    ]
  }
}
```

## Foundation Types

These shared types appear throughout the schema.

| Type        | Fields                                  | Description                          |
|-------------|-----------------------------------------|--------------------------------------|
| Color       | r, g, b, a (ubyte)                      | RGBA color value                     |
| Length      | value (float32), unit (LengthUnit)      | Dimensional value with unit          |
| Duration    | ms (float32)                            | Time duration in milliseconds        |
| Easing      | easing_type, control points/spring params | Animation timing function          |
| EdgeInsets  | top, right, bottom, left (Length)       | Four-sided spacing                   |
| CornerRadii | top_left, top_right, bottom_right, bottom_left (Length) | Per-corner radius    |
| Shadow      | offset_x, offset_y, blur, spread, color, inset | Box shadow definition         |
| DataBinding | source_node_id, field_path              | Runtime data reference               |

### LengthUnit Values

`Px`, `Rem`, `Em`, `Percent`, `Vw`, `Vh`, `Dvh`, `Svh`, `Auto`, `FitContent`, `MinContent`, `MaxContent`, `Fr`

### EasingType Values

`Linear`, `CubicBezier`, `Spring`, `Steps`, `CustomLinear`
