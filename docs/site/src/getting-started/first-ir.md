# Your First IR File

Voce IR documents are JSON files with a `.voce.json` extension. In this guide you will create a minimal "Hello World" IR file by hand and validate it.

## The minimal document

Create a file called `hello.voce.json` with the following content:

```json
{
  "schema_version_major": 1,
  "schema_version_minor": 0,
  "root": {
    "node_id": "root",
    "viewport_width": { "value": 1024, "unit": "Px" },
    "children": [
      {
        "value_type": "TextNode",
        "value": {
          "node_id": "greeting",
          "content": "Hello, world!",
          "heading_level": 1,
          "font_size": { "value": 36, "unit": "Px" },
          "font_weight": "Bold",
          "color": { "r": 0, "g": 0, "b": 0, "a": 255 }
        }
      }
    ],
    "metadata": {
      "title": "Hello World",
      "description": "A minimal Voce IR document."
    }
  },
  "metadata": {
    "title": "Hello World",
    "description": "A minimal Voce IR document.",
    "language": "en"
  }
}
```

## Structure breakdown

### Top-level fields

| Field | Purpose |
|-------|---------|
| `schema_version_major` | Major version of the Voce IR schema. Breaking changes increment this. |
| `schema_version_minor` | Minor version. Additive changes increment this. |
| `root` | The ViewRoot node -- every document has exactly one. |
| `metadata` | Document-level metadata (title, description, language). |

### The ViewRoot (`root`)

The root node represents the top-level viewport:

- **`node_id`** -- A unique string identifier. The root is conventionally called `"root"`.
- **`viewport_width`** -- The design viewport width. Uses a value/unit pair (e.g., `1024` pixels).
- **`children`** -- An array of child nodes. Each child is a tagged union with `value_type` and `value`.
- **`metadata`** -- Page metadata used for SEO and document identification.

### Child nodes (the tagged union)

Every child in the `children` array has two fields:

- **`value_type`** -- The node kind. Common types include `TextNode`, `Container`, `Surface`, `MediaNode`, `FormNode`, and many others.
- **`value`** -- The node's data, whose shape depends on the `value_type`.

### The TextNode

The simplest visible node. In our example:

- **`node_id`** -- Unique identifier for this node (`"greeting"`).
- **`content`** -- The text string to display.
- **`heading_level`** -- Semantic heading level (1-6). Setting this makes the compiler emit an `<h1>`-`<h6>` tag. Omit it for body text.
- **`font_size`** -- Size with unit. Supports `Px`, `Rem`, `Em`, `Vw`, `Vh`, `Percent`.
- **`font_weight`** -- One of `Thin`, `Light`, `Regular`, `Medium`, `SemiBold`, `Bold`, `ExtraBold`, `Black`.
- **`color`** -- RGBA color with values 0-255.

## Validate your file

Run the validator to confirm your IR is structurally correct:

```bash
voce validate hello.voce.json
```

If everything is valid, you will see:

```
hello.voce.json: VALID (1 node, 0 warnings)
```

If there are problems -- say you forgot the `node_id` -- the validator reports the exact issue:

```
hello.voce.json: INVALID
  [STR001] root.children[0]: TextNode missing required field "node_id"
```

## Adding more nodes

Here is the same document with a subtitle added below the heading:

```json
{
  "schema_version_major": 1,
  "schema_version_minor": 0,
  "root": {
    "node_id": "root",
    "viewport_width": { "value": 1024, "unit": "Px" },
    "children": [
      {
        "value_type": "TextNode",
        "value": {
          "node_id": "heading",
          "content": "Hello, world!",
          "heading_level": 1,
          "font_size": { "value": 36, "unit": "Px" },
          "font_weight": "Bold",
          "color": { "r": 0, "g": 0, "b": 0, "a": 255 }
        }
      },
      {
        "value_type": "TextNode",
        "value": {
          "node_id": "subtitle",
          "content": "This page was built with Voce IR.",
          "font_size": { "value": 18, "unit": "Px" },
          "color": { "r": 100, "g": 100, "b": 100, "a": 255 }
        }
      }
    ],
    "metadata": {
      "title": "Hello World",
      "description": "A minimal Voce IR document."
    }
  },
  "metadata": {
    "title": "Hello World",
    "description": "A minimal Voce IR document.",
    "language": "en"
  }
}
```

Note that the subtitle omits `heading_level` -- it will render as a paragraph, not a heading.

## Next steps

Your IR file is ready. Continue to [Compiling to HTML](./compiling.md) to turn it into a working web page.
