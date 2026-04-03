/**
 * Discovery Agent — quality gate that extracts structured requirements
 * from vague prompts. Blocks generation until readiness score >= 70.
 */

import { ClaudeClient } from "../api/claude-client.js";
import { Agent } from "./base-agent.js";
import type { DiscoveryBrief } from "./types.js";

export class DiscoveryAgent extends Agent<string, DiscoveryBrief> {
  name = "discovery";

  systemPrompt = `You are a requirements analyst for a UI generation system. Your job is to extract structured requirements from a user's description of what they want to build.

Output a JSON object with these fields:
- purpose: string — what is being built (1-2 sentences)
- audience: string — who will use it
- sections: string[] — key page sections in visual order
- ctas: string[] — call-to-action descriptions
- tone: string — visual/aesthetic tone description
- constraints: string[] — technical or business constraints
- readinessScore: number (0-100) — how well-specified the request is
- followUpQuestions: string[] — questions to ask if readinessScore < 70

Scoring guide:
- 90-100: Specific sections, audience, tone, and constraints described
- 70-89: Clear purpose and some sections, but missing details
- 50-69: Vague purpose, no specific sections (ask questions)
- 0-49: Extremely vague ("make me a website") — need many questions

Output ONLY valid JSON. No other text.`;

  constructor(client: ClaudeClient) {
    super(client, { temperature: 0.2, maxTokens: 2048 });
  }

  buildUserPrompt(input: string): string {
    return `Analyze this request and extract structured requirements:\n\n"${input}"`;
  }

  parseResponse(response: string): DiscoveryBrief {
    try {
      // Extract JSON from response
      const jsonStr = extractJson(response);
      const parsed = JSON.parse(jsonStr);
      return {
        purpose: parsed.purpose || "",
        audience: parsed.audience || "general",
        sections: parsed.sections || [],
        ctas: parsed.ctas || [],
        tone: parsed.tone || "clean and professional",
        constraints: parsed.constraints || [],
        readinessScore: parsed.readinessScore ?? 50,
        followUpQuestions: parsed.followUpQuestions || [],
      };
    } catch {
      return {
        purpose: "",
        audience: "",
        sections: [],
        ctas: [],
        tone: "",
        constraints: [],
        readinessScore: 0,
        followUpQuestions: [
          "Could you describe what you want to build in more detail?",
        ],
      };
    }
  }
}

function extractJson(text: string): string {
  // Try direct parse
  try {
    JSON.parse(text);
    return text;
  } catch {
    // ignored
  }

  // Try code fence
  const match = text.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
  if (match) return match[1];

  // Try braces
  const first = text.indexOf("{");
  const last = text.lastIndexOf("}");
  if (first !== -1 && last > first) {
    return text.slice(first, last + 1);
  }

  return text;
}
