/**
 * Design Agent — translates a DiscoveryBrief into concrete design decisions.
 * Applies UX best practices, selects layout patterns, color strategy, typography.
 */

import { ClaudeClient } from "../api/claude-client.js";
import { Agent } from "./base-agent.js";
import type { DesignSpec, DiscoveryBrief } from "./types.js";

export class DesignAgent extends Agent<DiscoveryBrief, DesignSpec> {
  name = "design";

  systemPrompt = `You are a UI design system architect. Given structured requirements, you produce concrete design decisions.

Output a JSON object with:
- layout: { type: "landing"|"dashboard"|"article"|"form"|"custom", sections: [{ name, type, children }] }
  - section types: "hero", "features", "testimonials", "pricing", "cta", "form", "footer", "header", "content"
  - children: array of element descriptions (e.g., "h1 headline", "subtitle text", "CTA button")
- colors: { background, foreground, primary, surface, muted } — each { r, g, b } (0-255)
- typography: { headingSize: px number, bodySize: px number, headingWeight: "Bold"|"SemiBold", bodyWeight: "Regular" }
- spacingBase: number (base spacing unit in px, typically 4 or 8)

Design heuristics:
- Landing pages: hero → features → social proof → CTA → footer
- Dark themes: background <30 lightness, text >200 lightness, accent 180-255 range
- Light themes: background >240, text <50, accent saturated
- Typography: heading 36-64px, body 14-18px, line-height 1.5
- Spacing: base 8px, scale by 2/3/4/6/8/12/16
- Always include header and footer sections
- Prefer 3-column grids for features, single column for hero

Output ONLY valid JSON.`;

  constructor(client: ClaudeClient) {
    super(client, { temperature: 0.4, maxTokens: 2048 });
  }

  buildUserPrompt(input: DiscoveryBrief): string {
    return `Design a UI for this project:

Purpose: ${input.purpose}
Audience: ${input.audience}
Sections needed: ${input.sections.join(", ") || "not specified"}
CTAs: ${input.ctas.join(", ") || "not specified"}
Tone: ${input.tone}
Constraints: ${input.constraints.join(", ") || "none"}`;
  }

  parseResponse(response: string): DesignSpec {
    try {
      const jsonStr = extractJson(response);
      const parsed = JSON.parse(jsonStr);
      return {
        layout: parsed.layout || {
          type: "landing",
          sections: [
            { name: "header", type: "header", children: ["logo", "nav links"] },
            { name: "hero", type: "hero", children: ["h1 headline", "subtitle", "CTA button"] },
            { name: "footer", type: "footer", children: ["copyright"] },
          ],
        },
        colors: parsed.colors || {
          background: { r: 12, g: 12, b: 14 },
          foreground: { r: 232, g: 230, b: 225 },
          primary: { r: 232, g: 89, b: 60 },
          surface: { r: 20, g: 20, b: 23 },
          muted: { r: 155, g: 154, b: 148 },
        },
        typography: parsed.typography || {
          headingSize: 48,
          bodySize: 16,
          headingWeight: "Bold",
          bodyWeight: "Regular",
        },
        spacingBase: parsed.spacingBase || 8,
      };
    } catch {
      // Return sensible defaults
      return {
        layout: {
          type: "landing",
          sections: [
            { name: "header", type: "header", children: ["logo"] },
            { name: "hero", type: "hero", children: ["headline", "subtitle", "CTA"] },
            { name: "footer", type: "footer", children: ["copyright"] },
          ],
        },
        colors: {
          background: { r: 12, g: 12, b: 14 },
          foreground: { r: 232, g: 230, b: 225 },
          primary: { r: 232, g: 89, b: 60 },
          surface: { r: 20, g: 20, b: 23 },
          muted: { r: 155, g: 154, b: 148 },
        },
        typography: { headingSize: 48, bodySize: 16, headingWeight: "Bold", bodyWeight: "Regular" },
        spacingBase: 8,
      };
    }
  }
}

function extractJson(text: string): string {
  try {
    JSON.parse(text);
    return text;
  } catch {
    // ignored
  }
  const match = text.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
  if (match) return match[1];
  const first = text.indexOf("{");
  const last = text.lastIndexOf("}");
  if (first !== -1 && last > first) return text.slice(first, last + 1);
  return text;
}
