/**
 * Generator Agent — emits complete Voce IR JSON from a DiscoveryBrief + DesignSpec.
 * Uses the full schema context and design decisions as input.
 */

import { ClaudeClient } from "../api/claude-client.js";
import { Agent } from "./base-agent.js";
import { buildSchemaContext } from "../context/schema-context.js";
import type { DesignSpec, DiscoveryBrief } from "./types.js";

interface GeneratorInput {
  brief: DiscoveryBrief;
  design: DesignSpec;
}

export class GeneratorAgent extends Agent<GeneratorInput, string> {
  name = "generator";
  systemPrompt: string;

  constructor(client: ClaudeClient) {
    super(client, { temperature: 0.3, maxTokens: 8192 });
    this.systemPrompt = buildSchemaContext();
  }

  buildUserPrompt(input: GeneratorInput): string {
    const { brief, design } = input;

    return `Generate a complete Voce IR JSON document based on these specifications:

## Requirements
Purpose: ${brief.purpose}
Audience: ${brief.audience}
Tone: ${brief.tone}
Sections: ${brief.sections.join(", ")}
CTAs: ${brief.ctas.join(", ")}

## Design Decisions
Layout type: ${design.layout.type}
Sections in order:
${design.layout.sections.map((s) => `  - ${s.name} (${s.type}): ${s.children.join(", ")}`).join("\n")}

Colors (RGB):
  background: rgb(${design.colors.background.r},${design.colors.background.g},${design.colors.background.b})
  foreground: rgb(${design.colors.foreground.r},${design.colors.foreground.g},${design.colors.foreground.b})
  primary: rgb(${design.colors.primary.r},${design.colors.primary.g},${design.colors.primary.b})
  surface: rgb(${design.colors.surface.r},${design.colors.surface.g},${design.colors.surface.b})
  muted: rgb(${design.colors.muted.r},${design.colors.muted.g},${design.colors.muted.b})

Typography: heading ${design.typography.headingSize}px ${design.typography.headingWeight}, body ${design.typography.bodySize}px
Spacing base: ${design.spacingBase}px

Output ONLY the complete Voce IR JSON. Include PageMetadata, SemanticNodes for all interactive elements, and a ThemeNode with the colors above.`;
  }

  parseResponse(response: string): string {
    // Extract JSON from response
    try {
      JSON.parse(response);
      return response;
    } catch {
      // ignored
    }

    const match = response.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
    if (match) {
      try {
        JSON.parse(match[1]);
        return match[1];
      } catch {
        // ignored
      }
    }

    const first = response.indexOf("{");
    const last = response.lastIndexOf("}");
    if (first !== -1 && last > first) {
      const candidate = response.slice(first, last + 1);
      try {
        JSON.parse(candidate);
        return candidate;
      } catch {
        // ignored
      }
    }

    return response;
  }
}
