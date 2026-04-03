# Sprint 02 — Core Schema: Types & Layout

**Status:** Ready for review
**Goal:** Define the foundational FlatBuffers schema files (`types.fbs`, `layout.fbs`), generate Rust bindings, verify JSON round-trip, and write first test fixtures. After this sprint, you can create a ViewRoot with Containers, Surfaces, TextNodes, and MediaNodes — the minimum for a static page.
**Depends on:** Sprint 01 (workspace, flatc)

---

## Deliverables

1. `packages/schema/schemas/types.fbs` — primitive types, composite types, constraint types
2. `packages/schema/schemas/layout.fbs` — ViewRoot, Container, Surface, TextNode, MediaNode
3. Generated Rust bindings in `packages/schema/src/generated/`
4. JSON canonical format: a hand-written JSON IR that parses to FlatBuffer and back
5. Unit tests for schema round-trip
6. First valid + invalid test fixtures in `tests/schema/`

---

## Tasks

### 1. Design `types.fbs` — The Type System

This is the foundation everything else builds on. Every other schema file imports from types.

**Key design decisions to make during implementation:**

- **Node identity:** Every node gets a `node_id: string` field. This is used for cross-referencing (Ref), delta updates, a11y tree mapping, and the `.voce/` memory system.
- **Union types for heterogeneous children:** Containers hold children of different types. FlatBuffers unions are the mechanism. Define a `Node` union covering all node types.
- **Enums vs strings:** Use enums for constrained values (layout direction, alignment). Use strings only for free-text (content, labels).

**`types.fbs` should define:**

```
namespace voce;

// --- Primitive scalar types ---
// FlatBuffers has built-in: bool, int8-64, uint8-64, float32/64, string
// We define semantic wrappers:

struct Color { r: uint8; g: uint8; b: uint8; a: uint8; }
struct Vec2 { x: float32; y: float32; }
struct Vec3 { x: float32; y: float32; z: float32; }
struct Vec4 { x: float32; y: float32; z: float32; w: float32; }

enum LengthUnit : byte { Px, Rem, Percent, Vw, Vh, Auto, FitContent, MinContent, MaxContent }
table Length { value: float32; unit: LengthUnit; }

enum DurationUnit : byte { Ms, S }
table Duration { value: float32; unit: DurationUnit; }

table Angle { degrees: float32; }

// --- Easing ---
enum EasingType : byte { Linear, CubicBezier, Spring, Steps, CustomLinear }
table Easing {
    easing_type: EasingType;
    // CubicBezier: x1, y1, x2, y2
    x1: float32; y1: float32; x2: float32; y2: float32;
    // Spring: stiffness, damping, mass
    stiffness: float32; damping: float32; mass: float32;
    // Steps: count, position
    steps: int32;
    // CustomLinear: pre-computed points for linear() CSS
    points: [float32];
}

// --- Composite types ---
enum Alignment : byte { Start, Center, End, Stretch, SpaceBetween, SpaceAround, SpaceEvenly }
enum LayoutDirection : byte { Row, Column, RowReverse, ColumnReverse }
enum TextDirection : byte { Ltr, Rtl, Auto }
enum Overflow : byte { Visible, Hidden, Scroll, Auto }
enum Position : byte { Relative, Absolute, Fixed, Sticky }

table EdgeInsets { top: Length; right: Length; bottom: Length; left: Length; }
table CornerRadii { top_left: Length; top_right: Length; bottom_right: Length; bottom_left: Length; }

table Shadow {
    offset_x: Length;
    offset_y: Length;
    blur: Length;
    spread: Length;
    color: Color;
    inset: bool;
}

// --- Cross-reference ---
// Ref is just a string matching a node_id.
// The validator resolves these.

// --- Data binding ---
table DataBinding {
    source_node_id: string;     // which DataNode/ComputeNode/ContextNode
    field_path: string;         // dot-separated path into the data
}
```

**Important FlatBuffers constraints to keep in mind:**
- Tables are the evolution-friendly type (use for everything that might grow)
- Structs are fixed-size (use for small, stable types like Color, Vec2)
- Unions require a type enum + table pair
- Vectors `[Type]` are used for ordered collections
- Default values reduce serialized size (only non-default fields are stored)

### 2. Design `layout.fbs` — Scene & Layout Nodes

```
include "types.fbs";

namespace voce.layout;

// --- The Node Union ---
// This will grow as we add schema files. For Sprint 02, it only has layout nodes.
// In later sprints, we'll need a master union in voce.fbs.

table ViewRoot {
    node_id: string (required);
    children: [voce.layout.LayoutNode];
    width: voce.Length;
    height: voce.Length;
    background: voce.Color;
    document_language: string;          // BCP 47 (e.g., "en-US")
    text_direction: voce.TextDirection;
    // PageMetadata, ThemeNode, RouteMap added in later sprints
}

union LayoutNode {
    Container,
    Surface,
    TextNode,
    MediaNode,
}

// --- Container ---
enum ContainerLayout : byte { Stack, Flex, Grid, Absolute }

table Container {
    node_id: string (required);
    children: [LayoutNode];
    layout: ContainerLayout = Stack;
    direction: voce.LayoutDirection = Column;
    main_align: voce.Alignment = Start;
    cross_align: voce.Alignment = Start;
    gap: voce.Length;
    padding: voce.EdgeInsets;
    wrap: bool = false;
    
    // Grid-specific
    grid_columns: [voce.Length];        // column track sizes
    grid_rows: [voce.Length];           // row track sizes
    
    // Sizing
    width: voce.Length;
    height: voce.Length;
    min_width: voce.Length;
    max_width: voce.Length;
    min_height: voce.Length;
    max_height: voce.Length;
    
    // Visual
    overflow: voce.Overflow = Visible;
    clip: bool = false;
    opacity: float32 = 1.0;
    
    // Position (for Absolute layout children)
    position: voce.Position = Relative;
    top: voce.Length;
    right: voce.Length;
    bottom: voce.Length;
    left: voce.Length;
    z_index: int32 = 0;
}

// --- Surface ---
table Surface {
    node_id: string (required);
    children: [LayoutNode];
    
    // Visual
    fill: voce.Color;
    stroke: voce.Color;
    stroke_width: voce.Length;
    corner_radius: voce.CornerRadii;
    shadow: [voce.Shadow];
    opacity: float32 = 1.0;
    
    // Sizing (same as Container)
    width: voce.Length;
    height: voce.Length;
    padding: voce.EdgeInsets;
    
    // Flags
    decorative: bool = false;           // if true, no SemanticNode required
}

// --- TextNode ---
enum FontWeight : int16 { 
    Thin = 100, ExtraLight = 200, Light = 300, Regular = 400,
    Medium = 500, SemiBold = 600, Bold = 700, ExtraBold = 800, Black = 900 
}
enum TextAlign : byte { Start, Center, End, Justify }
enum TextOverflow : byte { Clip, Ellipsis, Fade }

table TextNode {
    node_id: string (required);
    
    // Content — either static string or localized
    content: string;                    // static text content
    // LocalizedString reference added in Sprint 05 (i18n)
    // DataBinding for dynamic content added in Sprint 04 (data)
    
    // Typography
    font_family: string;
    font_size: voce.Length;
    font_weight: FontWeight = Regular;
    line_height: float32;               // multiplier (e.g., 1.5)
    letter_spacing: voce.Length;
    text_align: TextAlign = Start;
    text_overflow: TextOverflow = Clip;
    max_lines: int32;                   // 0 = unlimited
    
    // Visual
    color: voce.Color;
    opacity: float32 = 1.0;
    
    // Semantic heading level (0 = not a heading, 1-6 = h1-h6)
    heading_level: int8 = 0;
}

// --- MediaNode ---
enum MediaType : byte { Image, Video, Audio, Svg }
enum LoadingStrategy : byte { Eager, Lazy }
enum ObjectFit : byte { Cover, Contain, Fill, None, ScaleDown }

table MediaNode {
    node_id: string (required);
    
    media_type: MediaType = Image;
    src: string (required);             // URL or local path
    alt: string;                        // required by validator unless decorative
    
    // Dimensions (required for CLS prevention)
    width: voce.Length;
    height: voce.Length;
    aspect_ratio: float32;              // width/height (fallback if dimensions not set)
    
    object_fit: ObjectFit = Cover;
    loading: LoadingStrategy = Lazy;
    
    // Image optimization hints
    srcset_widths: [int32];             // responsive widths for compiler to generate
    sizes: string;                      // CSS sizes attribute
    
    // Flags
    decorative: bool = false;           // if true, alt="" is valid
    above_fold: bool = false;           // compiler hint: preload, eager load
}

// --- Root table for file format ---
table VoceDocument {
    schema_version_major: int32 = 0;
    schema_version_minor: int32 = 1;
    root: ViewRoot (required);
}

root_type VoceDocument;
```

### 3. Generate Rust Bindings

```bash
# From project root
flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/types.fbs
flatc --rust -o packages/schema/src/generated/ packages/schema/schemas/layout.fbs

# Verify generated files exist
ls packages/schema/src/generated/
```

Update `packages/schema/src/lib.rs` to export the generated modules.

**Note:** FlatBuffers Rust codegen produces `#[allow(unused)]` and other attributes. The generated code should NOT be linted — add it to clippy allow list or use `#[allow(clippy::all)]` in the wrapper module.

### 4. JSON Canonical Format Test

Create a hand-written JSON file that represents a minimal valid IR (ViewRoot with one Container and one TextNode). Verify it can be:

1. Parsed as JSON → in-memory structure
2. Encoded to FlatBuffer binary
3. Decoded from binary back to in-memory
4. Serialized back to JSON
5. JSON output matches input (modulo formatting)

**`tests/schema/valid/minimal-page.json`:**
```json
{
  "schema_version_major": 0,
  "schema_version_minor": 1,
  "root": {
    "node_id": "root",
    "document_language": "en",
    "text_direction": "Ltr",
    "children": [
      {
        "type": "Container",
        "value": {
          "node_id": "main",
          "layout": "Stack",
          "direction": "Column",
          "children": [
            {
              "type": "TextNode",
              "value": {
                "node_id": "heading",
                "content": "Hello, Voce",
                "font_size": { "value": 48.0, "unit": "Px" },
                "font_weight": 700,
                "heading_level": 1,
                "color": { "r": 232, "g": 230, "b": 225, "a": 255 }
              }
            }
          ]
        }
      }
    ]
  }
}
```

### 5. Write Unit Tests

**`packages/schema/src/lib.rs` tests:**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_schema_compiles() {
        // If this file compiles, the generated bindings work
        // More detailed tests after bindings exist
    }
}
```

**`tests/schema/round_trip.rs`:**
- Load `minimal-page.json`
- Parse to FlatBuffer
- Decode from FlatBuffer
- Verify all fields match

### 6. Invalid Test Fixtures

**`tests/schema/invalid/missing-node-id.json`** — Container without node_id
**`tests/schema/invalid/missing-root.json`** — VoceDocument without root
**`tests/schema/invalid/empty-children.json`** — ViewRoot with no children (valid structurally, but the validator will flag it later)

---

## Acceptance Criteria

- [ ] `flatc` compiles both `types.fbs` and `layout.fbs` without errors
- [ ] Generated Rust bindings compile (`cargo build -p voce-schema`)
- [ ] `minimal-page.json` round-trips through FlatBuffers binary and back
- [ ] `cargo test --workspace` passes with round-trip test
- [ ] All node types have `node_id: string (required)` field
- [ ] `LayoutNode` union contains all 4 layout types
- [ ] `VoceDocument` is the root type with schema version fields
- [ ] `cargo clippy --workspace -- -D warnings` passes

---

## Design Decisions to Record

After completing this sprint, record in `.voce/decisions/`:

- **D-S02-001:** FlatBuffers schema namespace structure (`voce`, `voce.layout`, etc.)
- **D-S02-002:** Union-based heterogeneous children (vs vector-of-tables with type field)
- **D-S02-003:** `node_id` as `string` (vs `uint32` or `uint64`) — flexibility vs size tradeoff
- **D-S02-004:** Length as table (value + unit) vs separate fields — extensibility decision
- **D-S02-005:** JSON canonical format structure (how unions are represented in JSON)

---

## Open Questions (Resolve During Sprint)

1. **FlatBuffers union representation in JSON** — How does `flatc` serialize unions to JSON? Does it use `{ "type": "TypeName", "value": {...} }` or a different convention? This determines the canonical JSON format.

2. **Namespace strategy** — One flat namespace (`voce`) or domain-specific (`voce.layout`, `voce.state`, etc.)? FlatBuffers namespaces affect Rust module generation. Test both and pick based on ergonomics.

3. **Should `children` be on every node?** — Currently Container and Surface have children. TextNode and MediaNode are leaf nodes. Is this the right split, or should all nodes accept children for flexibility?

4. **Default values** — FlatBuffers omits default-valued fields from the binary. Set good defaults to minimize IR size (e.g., `opacity: 1.0`, `layout: Stack`, `loading: Lazy`).

---

## Notes

- FlatBuffers Rust codegen is somewhat opinionated about naming and module structure. You may need wrapper types for ergonomic Rust API.
- The `LayoutNode` union will be replaced by a broader `Node` union in `voce.fbs` (Sprint 05) once all schema files exist.
- Don't worry about making the schema perfect — it will evolve. The additive-only policy means we can always add fields. We can't remove them, so be conservative about what's required.
