/**
 * Style pack loader — loads built-in packs and provides selection.
 */

import type { StylePack, DesignTokens, RGB } from "./types.js";

/** All built-in style packs. */
const PACKS: StylePack[] = [
  {
    id: "minimal-saas",
    name: "Minimal SaaS",
    description: "Clean, modern SaaS aesthetic — generous whitespace, blue accents, system fonts",
    tags: ["saas", "landing", "clean", "minimal", "startup", "tech", "professional"],
    tokens: {
      colors: {
        background: { r: 255, g: 255, b: 255 },
        foreground: { r: 17, g: 24, b: 39 },
        primary: { r: 37, g: 99, b: 235 },
        surface: { r: 249, g: 250, b: 251 },
        muted: { r: 107, g: 114, b: 128 },
      },
      typography: {
        headingFamily: "Inter, system-ui, sans-serif",
        bodyFamily: "Inter, system-ui, sans-serif",
        headingSize: 48,
        bodySize: 16,
        headingWeight: "Bold",
        lineHeight: 1.6,
      },
      spacing: { base: 4, scale: [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64] },
      radii: { small: 4, medium: 8, large: 16 },
    },
    examples: [
      { filename: "landing.json", description: "SaaS landing page with hero, features grid, pricing, and CTA", tags: ["landing", "hero", "features", "pricing"] },
      { filename: "pricing.json", description: "Pricing page with three-tier comparison table and FAQ", tags: ["pricing", "comparison", "faq"] },
    ],
  },
  {
    id: "editorial",
    name: "Editorial",
    description: "Content-focused — serif headings, warm tones, optimized reading experience",
    tags: ["editorial", "blog", "content", "magazine", "article", "reading", "warm"],
    tokens: {
      colors: {
        background: { r: 252, g: 249, b: 244 },
        foreground: { r: 41, g: 37, b: 36 },
        primary: { r: 180, g: 83, b: 9 },
        surface: { r: 255, g: 255, b: 255 },
        muted: { r: 120, g: 113, b: 108 },
      },
      typography: {
        headingFamily: "Playfair Display, Georgia, serif",
        bodyFamily: "Source Serif Pro, Georgia, serif",
        headingSize: 42,
        bodySize: 18,
        headingWeight: "Bold",
        lineHeight: 1.75,
      },
      spacing: { base: 8, scale: [0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32] },
      radii: { small: 0, medium: 2, large: 4 },
    },
    examples: [
      { filename: "blog-post.json", description: "Long-form blog post with hero image, article body, author bio, and related posts", tags: ["blog", "article", "long-form"] },
      { filename: "magazine.json", description: "Magazine homepage with featured article, article grid, and newsletter signup", tags: ["magazine", "homepage", "grid"] },
    ],
  },
  {
    id: "ecommerce",
    name: "Ecommerce",
    description: "Conversion-optimized — bold CTAs, product-focused, trust signals",
    tags: ["ecommerce", "shop", "product", "store", "conversion", "commerce", "sell"],
    tokens: {
      colors: {
        background: { r: 255, g: 255, b: 255 },
        foreground: { r: 15, g: 23, b: 42 },
        primary: { r: 22, g: 163, b: 74 },
        surface: { r: 248, g: 250, b: 252 },
        muted: { r: 100, g: 116, b: 139 },
        accent: { r: 234, g: 88, b: 12 },
      },
      typography: {
        headingFamily: "DM Sans, system-ui, sans-serif",
        bodyFamily: "DM Sans, system-ui, sans-serif",
        headingSize: 40,
        bodySize: 15,
        headingWeight: "Bold",
        lineHeight: 1.5,
      },
      spacing: { base: 4, scale: [0, 1, 2, 3, 4, 6, 8, 12, 16, 20, 24, 32, 48] },
      radii: { small: 6, medium: 12, large: 20 },
    },
    examples: [
      { filename: "product.json", description: "Product detail page with image gallery, add-to-cart, reviews, and related products", tags: ["product", "detail", "cart", "reviews"] },
      { filename: "category.json", description: "Category listing with filters, product grid, and pagination", tags: ["category", "listing", "grid", "filter"] },
    ],
  },
];

/** Get all available style packs. */
export function getAllPacks(): StylePack[] {
  return PACKS;
}

/** Get a pack by ID. */
export function getPack(id: string): StylePack | undefined {
  return PACKS.find((p) => p.id === id);
}

/** Find the best matching pack for a set of tags/keywords. */
export function matchPack(keywords: string[]): StylePack {
  const lower = keywords.map((k) => k.toLowerCase());
  let bestPack = PACKS[0];
  let bestScore = 0;

  for (const pack of PACKS) {
    let score = 0;
    for (const keyword of lower) {
      for (const tag of pack.tags) {
        if (tag.includes(keyword) || keyword.includes(tag)) {
          score++;
        }
      }
    }
    if (score > bestScore) {
      bestScore = score;
      bestPack = pack;
    }
  }

  return bestPack;
}

/** Format pack list for display in conversation. */
export function formatPackList(): string {
  return PACKS.map(
    (p) => `  ${p.id}: ${p.description}`
  ).join("\n");
}
