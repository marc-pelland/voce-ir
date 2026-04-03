# Layout Nodes

Layout nodes define the spatial composition of a Voce IR document. There are five layout node types: ViewRoot (the document root), Container (grouping and layout), Surface (visual rectangles), TextNode (styled text), and MediaNode (images, video, audio).

## ViewRoot

Top-level container for a document or route. One ViewRoot per page. Defines viewport bounds, document language, and holds the flat semantic node list.

| Field             | Type            | Required | Description                                      |
|-------------------|-----------------|----------|--------------------------------------------------|
| node_id           | string          | yes      | Unique identifier                                |
| children          | [ChildNode]     | no       | Child nodes                                      |
| width             | Length          | no       | Viewport width                                   |
| height            | Length          | no       | Viewport height                                  |
| background        | Color           | no       | Document background color                        |
| document_language | string          | no       | BCP 47 language tag (e.g., "en")                 |
| text_direction    | TextDirection   | no       | Ltr (default), Rtl, or Auto                      |
| semantic_nodes    | [SemanticNode]  | no       | Flat list of semantic nodes referenced by visual nodes |
| metadata          | PageMetadata    | no       | Per-page SEO metadata                            |

```json
{
  "node_id": "root",
  "document_language": "en",
  "text_direction": "Ltr",
  "background": { "r": 255, "g": 255, "b": 255, "a": 255 },
  "children": []
}
```

## Container

Groups children with a layout strategy. The primary structural node for composition.

| Field          | Type              | Required | Description                                    |
|----------------|-------------------|----------|------------------------------------------------|
| node_id        | string            | yes      | Unique identifier                              |
| children       | [ChildNode]       | no       | Child nodes                                    |
| layout         | ContainerLayout   | no       | Stack (default), Flex, Grid, or Absolute       |
| direction      | LayoutDirection   | no       | Row, Column (default), RowReverse, ColumnReverse |
| main_align     | Alignment         | no       | Main axis alignment (default Start)            |
| cross_align    | Alignment         | no       | Cross axis alignment (default Start)           |
| gap            | Length            | no       | Gap between children                           |
| padding        | EdgeInsets        | no       | Inner padding                                  |
| wrap           | bool              | no       | Enable flex wrapping (default false)           |
| grid_columns   | [Length]          | no       | Column track sizes for Grid layout             |
| grid_rows      | [Length]          | no       | Row track sizes for Grid layout                |
| width          | Length            | no       | Explicit width                                 |
| height         | Length            | no       | Explicit height                                |
| min_width      | Length            | no       | Minimum width constraint                       |
| max_width      | Length            | no       | Maximum width constraint                       |
| min_height     | Length            | no       | Minimum height constraint                      |
| max_height     | Length            | no       | Maximum height constraint                      |
| overflow_x     | Overflow          | no       | Horizontal overflow (default Visible)          |
| overflow_y     | Overflow          | no       | Vertical overflow (default Visible)            |
| clip           | bool              | no       | Clip overflowing content (default false)       |
| position       | Position          | no       | Relative (default), Absolute, Fixed, Sticky    |
| top            | Length            | no       | Top offset (for positioned elements)           |
| right          | Length            | no       | Right offset                                   |
| bottom         | Length            | no       | Bottom offset                                  |
| left           | Length            | no       | Left offset                                    |
| z_index        | int32             | no       | Stacking order (default 0)                     |
| opacity        | float32           | no       | Opacity 0.0-1.0 (default 1.0)                 |
| background     | Color             | no       | Background color                               |
| border         | BorderSides       | no       | Per-side border configuration                  |
| corner_radius  | CornerRadii       | no       | Per-corner border radius                       |
| shadow         | [Shadow]          | no       | Box shadows                                    |
| semantic_node_id | string          | no       | Reference to a SemanticNode                    |

```json
{
  "node_id": "hero-row",
  "layout": "Flex",
  "direction": "Row",
  "gap": { "value": 16, "unit": "Px" },
  "padding": {
    "top": { "value": 24, "unit": "Px" },
    "right": { "value": 24, "unit": "Px" },
    "bottom": { "value": 24, "unit": "Px" },
    "left": { "value": 24, "unit": "Px" }
  },
  "main_align": "Center",
  "cross_align": "Center",
  "children": []
}
```

## Surface

A visible rectangular region used for cards, backgrounds, dividers, and decorative elements.

| Field            | Type         | Required | Description                                  |
|------------------|--------------|----------|----------------------------------------------|
| node_id          | string       | yes      | Unique identifier                            |
| children         | [ChildNode]  | no       | Child nodes                                  |
| fill             | Color        | no       | Fill color                                   |
| stroke           | Color        | no       | Stroke/border color                          |
| stroke_width     | Length       | no       | Stroke thickness                             |
| corner_radius    | CornerRadii  | no       | Per-corner border radius                     |
| shadow           | [Shadow]     | no       | Box shadows                                  |
| opacity          | float32      | no       | Opacity 0.0-1.0 (default 1.0)               |
| border           | BorderSides  | no       | Per-side border configuration                |
| width            | Length       | no       | Explicit width                               |
| height           | Length       | no       | Explicit height                              |
| min_width        | Length       | no       | Minimum width constraint                     |
| max_width        | Length       | no       | Maximum width constraint                     |
| min_height       | Length       | no       | Minimum height constraint                    |
| max_height       | Length       | no       | Maximum height constraint                    |
| padding          | EdgeInsets   | no       | Inner padding                                |
| decorative       | bool         | no       | If true, no SemanticNode required (default false) |
| semantic_node_id | string       | no       | Reference to a SemanticNode                  |

```json
{
  "node_id": "card",
  "fill": { "r": 248, "g": 248, "b": 248, "a": 255 },
  "corner_radius": {
    "top_left": { "value": 8, "unit": "Px" },
    "top_right": { "value": 8, "unit": "Px" },
    "bottom_right": { "value": 8, "unit": "Px" },
    "bottom_left": { "value": 8, "unit": "Px" }
  },
  "shadow": [{
    "offset_x": { "value": 0, "unit": "Px" },
    "offset_y": { "value": 2, "unit": "Px" },
    "blur": { "value": 8, "unit": "Px" },
    "spread": { "value": 0, "unit": "Px" },
    "color": { "r": 0, "g": 0, "b": 0, "a": 25 }
  }],
  "decorative": false,
  "children": []
}
```

## TextNode

Styled text content. All typography properties are explicit with no cascade.

| Field             | Type            | Required | Description                                    |
|-------------------|-----------------|----------|------------------------------------------------|
| node_id           | string          | yes      | Unique identifier                              |
| content           | string          | no       | Static text content                            |
| content_binding   | DataBinding     | no       | Dynamic content from a DataNode                |
| localized_content | LocalizedString | no       | i18n content (alternative to static content)   |
| font_family       | string          | no       | Font family name                               |
| font_size         | Length          | no       | Font size                                      |
| font_weight       | FontWeight      | no       | 100-900 weight (default Regular/400)           |
| line_height       | float32         | no       | Line height multiplier (default 1.5)           |
| letter_spacing    | Length          | no       | Letter spacing                                 |
| text_align        | TextAlign       | no       | Start (default), Center, End, Justify          |
| text_overflow     | TextOverflow    | no       | Clip (default), Ellipsis, Fade                 |
| text_decoration   | TextDecoration  | no       | None (default), Underline, LineThrough         |
| max_lines         | int32           | no       | Maximum number of visible lines                |
| color             | Color           | no       | Text color                                     |
| opacity           | float32         | no       | Opacity 0.0-1.0 (default 1.0)                 |
| heading_level     | int8            | no       | 0 = not a heading, 1-6 = h1-h6 (default 0)    |
| semantic_node_id  | string          | no       | Reference to a SemanticNode                    |

```json
{
  "node_id": "page-title",
  "content": "Welcome to Voce",
  "heading_level": 1,
  "font_size": { "value": 3, "unit": "Rem" },
  "font_weight": "Bold",
  "color": { "r": 17, "g": 24, "b": 39, "a": 255 }
}
```

## MediaNode

Image, video, audio, or SVG with explicit dimensions, loading strategy, and format negotiation.

| Field            | Type            | Required | Description                                   |
|------------------|-----------------|----------|-----------------------------------------------|
| node_id          | string          | yes      | Unique identifier                             |
| media_type       | MediaType       | no       | Image (default), Video, Audio, Svg            |
| src              | string          | yes      | Source URL                                     |
| alt              | string          | no       | Alt text (required for non-decorative images)  |
| width            | Length          | no       | Explicit width (recommended for CLS prevention)|
| height           | Length          | no       | Explicit height                               |
| aspect_ratio     | float32         | no       | Aspect ratio (width/height)                   |
| object_fit       | ObjectFit       | no       | Cover (default), Contain, Fill, ScaleDown, None|
| loading          | LoadingStrategy | no       | Eager or Lazy (default)                       |
| corner_radius    | CornerRadii     | no       | Per-corner border radius                      |
| opacity          | float32         | no       | Opacity 0.0-1.0 (default 1.0)                |
| srcset_widths    | [int32]         | no       | Widths for responsive srcset generation        |
| sizes            | string          | no       | Sizes attribute for responsive images          |
| decorative       | bool            | no       | If true, no alt text required (default false)  |
| above_fold       | bool            | no       | If true, compiler uses eager loading (default false) |
| semantic_node_id | string          | no       | Reference to a SemanticNode                   |

```json
{
  "node_id": "hero-image",
  "media_type": "Image",
  "src": "/images/hero.webp",
  "alt": "Product dashboard showing analytics overview",
  "width": { "value": 100, "unit": "Percent" },
  "aspect_ratio": 1.778,
  "above_fold": true,
  "loading": "Eager"
}
```
