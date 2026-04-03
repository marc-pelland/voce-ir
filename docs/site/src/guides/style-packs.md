# Style Packs

Style packs are design presets for AI-generated output -- think of them as
LoRA adapters for UI. Instead of describing colors, fonts, and spacing from
scratch in every conversation, you select a style pack and the AI bridge
applies its design tokens to the generated IR.

## What a Style Pack Contains

Each pack defines a complete visual identity:

- **Color palette** -- background, foreground, primary, surface, muted, and
  optional accent colors (as RGB values)
- **Typography** -- heading and body font families, sizes, weights, and line
  height
- **Spacing** -- base unit and a scale array for consistent rhythm
- **Border radii** -- small, medium, and large values for component rounding
- **Component patterns** -- example IR files showing how the pack's tokens
  apply to common UI patterns (hero sections, pricing cards, forms)

The type definitions live in `packages/ai-bridge/src/packs/types.ts`:

```typescript
export interface StylePack {
  id: string;
  name: string;
  description: string;
  tags: string[];
  tokens: DesignTokens;
  examples: PackExample[];
}
```

## Built-in Packs

Voce ships with three built-in style packs:

### minimal-saas

A clean, utilitarian design for SaaS dashboards and landing pages. High
contrast, generous whitespace, and a neutral palette with a single accent
color. Typography uses a system font stack for fast loading.

Tags: `saas`, `landing`, `clean`, `dashboard`

### editorial

A content-first design for blogs, documentation, and long-form reading.
Serif headings, generous line height, narrow content column, and muted
colors that keep attention on the text.

Tags: `blog`, `editorial`, `content`, `documentation`

### ecommerce

A conversion-oriented design for product pages and storefronts. Bold
primary colors, tight spacing for product grids, prominent call-to-action
buttons, and image-heavy layouts.

Tags: `ecommerce`, `product`, `store`, `conversion`

## How the AI Bridge Uses Packs

When a user starts a conversation, the AI bridge can select a style pack
based on the user's description (or the user can request one explicitly).
The bridge then:

1. Loads the pack's design tokens
2. Injects token values into the IR's `ThemeNode` during generation
3. Uses the pack's example IR files for RAG (retrieval-augmented generation)
   matching -- if the user asks for a "pricing section," the bridge retrieves
   the pack's pricing example as context for the LLM

The pack's `tags` field enables automatic matching. When a user says "build me
a SaaS landing page," the bridge scores packs by tag overlap and selects the
best fit.

## Pack Examples and RAG

Each pack includes `PackExample` entries:

```typescript
export interface PackExample {
  filename: string;
  description: string;
  tags: string[];
  irJson?: string;
}
```

The `description` and `tags` fields are indexed for similarity search. When
the AI bridge receives a user intent, it retrieves the most relevant examples
from the active pack and includes them as few-shot context for the LLM. This
grounds generation in concrete, validated IR rather than relying solely on
schema knowledge.

## Creating a Custom Pack

To add a new style pack:

1. Create a new directory under `packages/ai-bridge/src/packs/` with your
   pack ID as the name
2. Define a `StylePack` object with your design tokens
3. Add example IR files that demonstrate your pack's visual language
4. Register the pack in the loader (`packages/ai-bridge/src/packs/loader.ts`)

The pack's tokens must use RGB values (0-255 per channel). The spacing scale
is an array of multipliers applied to the base unit -- for example, a base of
`8` with scale `[0.5, 1, 2, 3, 4, 6, 8]` produces `4, 8, 16, 24, 32, 48, 64`
pixel values.

## Packs vs. Themes

Style packs and IR themes (`ThemeNode`) serve different roles:

- **Style packs** are an AI bridge concept. They guide generation by providing
  design tokens and examples. They exist at authoring time.
- **Themes** are an IR concept. They are embedded in the generated document
  and survive compilation. They exist at runtime.

The AI bridge translates pack tokens into theme nodes during generation. A
single pack can produce multiple theme variants (light/dark) stored as
`alternate_themes` in the `VoceDocument`.
