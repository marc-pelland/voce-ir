# Theming & Personalization Nodes

Design tokens, multi-theme support, responsive breakpoints, and personalization slots. Theme switching is modeled as a state machine transition (e.g., light to dark).

## ThemeNode

A named set of design tokens. Multiple themes can coexist (light, dark, high-contrast). Referenced from `VoceDocument.theme` and `VoceDocument.alternate_themes`.

| Field               | Type           | Required | Description                                |
|---------------------|----------------|----------|--------------------------------------------|
| node_id             | string         | yes      | Unique identifier                          |
| name                | string         | yes      | Theme name (e.g., "light", "dark")         |
| colors              | ColorPalette   | no       | Color token definitions                    |
| typography          | TypographyScale| no       | Typography token definitions               |
| spacing             | SpacingScale   | no       | Spacing token definitions                  |
| shadows             | ShadowScale    | no       | Shadow scale definitions                   |
| radii               | RadiusScale    | no       | Border radius scale definitions            |
| transition_duration | Duration       | no       | Animation duration when switching to theme |
| transition_easing   | Easing         | no       | Easing function for theme transition       |

### ColorPalette

Semantic color tokens. Each token is a `Color` struct (r, g, b, a).

| Field                | Type  | Description                  |
|----------------------|-------|------------------------------|
| primary              | Color | Primary brand color          |
| primary_foreground   | Color | Text on primary              |
| secondary            | Color | Secondary brand color        |
| secondary_foreground | Color | Text on secondary            |
| accent               | Color | Accent color                 |
| accent_foreground    | Color | Text on accent               |
| background           | Color | Page background              |
| foreground           | Color | Default text color           |
| surface              | Color | Card/surface background      |
| surface_foreground   | Color | Text on surface              |
| muted                | Color | Muted/subtle background      |
| muted_foreground     | Color | Text on muted                |
| border_color         | Color | Default border color         |
| error                | Color | Error state color            |
| error_foreground     | Color | Text on error                |
| success              | Color | Success state color          |
| success_foreground   | Color | Text on success              |
| warning              | Color | Warning state color          |
| warning_foreground   | Color | Text on warning              |
| info                 | Color | Info state color             |
| info_foreground      | Color | Text on info                 |

### TypographyScale

| Field             | Type            | Description                                     |
|-------------------|-----------------|-------------------------------------------------|
| font_body         | FontDefinition  | Body text font                                  |
| font_display      | FontDefinition  | Display/heading font                            |
| font_mono         | FontDefinition  | Monospace font                                  |
| size_scale        | [Length]        | Size steps (xs through 4xl)                     |
| line_height_scale | [float32]       | Line heights matching size_scale indexes        |
| letter_spacing    | Length          | Default letter spacing                          |

### SpacingScale

| Field       | Type      | Description                                          |
|-------------|-----------|------------------------------------------------------|
| base        | Length    | Base spacing unit (e.g., 4px)                        |
| multipliers | [float32] | Multiplier scale (actual spacing = base * multiplier)|

```json
{
  "node_id": "theme-light",
  "name": "light",
  "colors": {
    "primary": { "r": 59, "g": 130, "b": 246, "a": 255 },
    "primary_foreground": { "r": 255, "g": 255, "b": 255, "a": 255 },
    "background": { "r": 255, "g": 255, "b": 255, "a": 255 },
    "foreground": { "r": 17, "g": 24, "b": 39, "a": 255 }
  },
  "spacing": {
    "base": { "value": 4, "unit": "Px" },
    "multipliers": [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32]
  }
}
```

## PersonalizationSlot

A point in the IR that adapts based on user context: locale, device type, color scheme preference, A/B test cohort, or custom conditions.

| Field                 | Type                        | Required | Description                              |
|-----------------------|-----------------------------|----------|------------------------------------------|
| node_id               | string                      | yes      | Unique identifier                        |
| name                  | string                      | no       | Human-readable name                      |
| variants              | [PersonalizationVariant]    | yes      | List of conditional variants             |
| default_variant_index | int32                       | no       | Fallback variant index (default 0)       |

### PersonalizationVariant

| Field      | Type                          | Required | Description                              |
|------------|-------------------------------|----------|------------------------------------------|
| conditions | [PersonalizationCondition]    | yes      | All conditions must be true to activate  |
| show_nodes | [string]                      | no       | Node IDs to show when active             |
| hide_nodes | [string]                      | no       | Node IDs to hide when active             |
| overrides  | [PropertyOverride]            | no       | Property overrides to apply              |

### PersonalizationCondition

| Field          | Type                            | Description                              |
|----------------|---------------------------------|------------------------------------------|
| condition_type | PersonalizationConditionType    | Locale, DeviceType, ColorScheme, ReducedMotion, HighContrast, Viewport, Custom |
| operator       | string                          | Comparison: "eq", "neq", "gt", "lt", "gte", "lte", "contains" |
| value          | string                          | Value to compare against                 |

```json
{
  "node_id": "mobile-variant",
  "name": "mobile-layout",
  "variants": [
    {
      "conditions": [
        { "condition_type": "DeviceType", "operator": "eq", "value": "mobile" }
      ],
      "hide_nodes": ["desktop-sidebar"],
      "show_nodes": ["mobile-nav"]
    }
  ],
  "default_variant_index": 0
}
```

## ResponsiveRule

Adapts layout based on viewport dimensions using explicit breakpoints with property overrides. Unlike CSS media queries, there is no cascading.

| Field                | Type                  | Required | Description                              |
|----------------------|-----------------------|----------|------------------------------------------|
| node_id              | string                | yes      | Unique identifier                        |
| breakpoints          | [Breakpoint]          | yes      | Breakpoint definitions                   |
| responsive_overrides | [ResponsiveOverride]  | yes      | Per-breakpoint property overrides        |

### Breakpoint

| Field     | Type   | Required | Description                                   |
|-----------|--------|----------|-----------------------------------------------|
| name      | string | yes      | Breakpoint name (e.g., "sm", "md", "lg")      |
| min_width | Length | yes      | Minimum viewport width for this breakpoint     |

### ResponsiveOverride

| Field           | Type               | Required | Description                              |
|-----------------|--------------------|----------|------------------------------------------|
| breakpoint_name | string             | yes      | Which breakpoint this applies at         |
| overrides       | [PropertyOverride] | yes      | Property overrides at this breakpoint    |

### PropertyOverride

| Field          | Type   | Required | Description                    |
|----------------|--------|----------|--------------------------------|
| target_node_id | string | yes      | Node to override               |
| property       | string | yes      | Property name to override      |
| value          | string | yes      | New value at this breakpoint   |

```json
{
  "node_id": "responsive-grid",
  "breakpoints": [
    { "name": "sm", "min_width": { "value": 640, "unit": "Px" } },
    { "name": "lg", "min_width": { "value": 1024, "unit": "Px" } }
  ],
  "responsive_overrides": [
    {
      "breakpoint_name": "sm",
      "overrides": [
        { "target_node_id": "content-grid", "property": "grid_columns", "value": "1" }
      ]
    },
    {
      "breakpoint_name": "lg",
      "overrides": [
        { "target_node_id": "content-grid", "property": "grid_columns", "value": "3" }
      ]
    }
  ]
}
```
