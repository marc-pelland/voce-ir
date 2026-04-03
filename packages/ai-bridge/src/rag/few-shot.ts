/**
 * Few-shot prompt builder — injects retrieved examples into the
 * Generator Agent's context for higher quality output.
 */

import type { RetrievalResult } from "./index.js";

/**
 * Build a few-shot context string from retrieved examples.
 * Each example is truncated to fit within token budget.
 */
export function buildFewShotContext(
  results: RetrievalResult[],
  maxTokensPerExample: number = 500
): string {
  if (results.length === 0) return "";

  const parts: string[] = [
    "Here are examples of similar pages for reference:\n",
  ];

  for (const result of results) {
    const desc = result.example.description;
    const packId = result.packId;

    parts.push(`### Example: ${desc} (from ${packId} pack)`);

    if (result.example.irJson) {
      // Truncate IR to fit token budget (~4 chars per token)
      const maxChars = maxTokensPerExample * 4;
      const truncated =
        result.example.irJson.length > maxChars
          ? result.example.irJson.slice(0, maxChars) + "\n... (truncated)"
          : result.example.irJson;
      parts.push("```json\n" + truncated + "\n```\n");
    } else {
      parts.push(`(${desc} — IR file available in the ${packId} style pack)\n`);
    }
  }

  parts.push(
    "Use these examples as reference for structure and style, but adapt to the specific requirements.\n"
  );

  return parts.join("\n");
}

/**
 * Build design token context from the selected style pack.
 */
export function buildTokenContext(tokens: {
  colors: Record<string, { r: number; g: number; b: number }>;
  typography: Record<string, string | number>;
  spacing: { base: number };
  radii: Record<string, number>;
}): string {
  const lines: string[] = ["## Design Tokens (from selected style pack)\n"];

  // Colors
  lines.push("Colors:");
  for (const [name, color] of Object.entries(tokens.colors)) {
    if (color && typeof color === "object" && "r" in color) {
      lines.push(`  ${name}: rgb(${color.r}, ${color.g}, ${color.b})`);
    }
  }

  // Typography
  lines.push("\nTypography:");
  for (const [name, value] of Object.entries(tokens.typography)) {
    lines.push(`  ${name}: ${value}`);
  }

  // Spacing
  lines.push(`\nSpacing base: ${tokens.spacing.base}px`);

  // Radii
  lines.push("\nBorder radii:");
  for (const [name, value] of Object.entries(tokens.radii)) {
    lines.push(`  ${name}: ${value}px`);
  }

  return lines.join("\n");
}
