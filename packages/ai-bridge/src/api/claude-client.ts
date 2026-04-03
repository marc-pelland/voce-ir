/**
 * Claude API client wrapper with Voce IR defaults.
 *
 * Handles: model selection, structured JSON output, retries,
 * API key resolution.
 */

import Anthropic from "@anthropic-ai/sdk";

export interface GenerateOptions {
  model?: string;
  maxTokens?: number;
  temperature?: number;
}

const DEFAULT_MODEL = "claude-sonnet-4-20250514";
const DEFAULT_MAX_TOKENS = 8192;
const DEFAULT_TEMPERATURE = 0.3;
const MAX_RETRIES = 2;

export class ClaudeClient {
  private client: Anthropic;

  constructor(apiKey?: string) {
    const key = apiKey || process.env.ANTHROPIC_API_KEY;
    if (!key) {
      throw new Error(
        "ANTHROPIC_API_KEY is required. Set it as an environment variable or pass it to ClaudeClient."
      );
    }
    this.client = new Anthropic({ apiKey: key });
  }

  /**
   * Generate structured JSON from a system + user prompt.
   */
  async generate(
    systemPrompt: string,
    userPrompt: string,
    options: GenerateOptions = {}
  ): Promise<string> {
    const model = options.model || DEFAULT_MODEL;
    const maxTokens = options.maxTokens || DEFAULT_MAX_TOKENS;
    const temperature = options.temperature || DEFAULT_TEMPERATURE;

    let lastError: Error | null = null;

    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        const response = await this.client.messages.create({
          model,
          max_tokens: maxTokens,
          temperature,
          system: systemPrompt,
          messages: [{ role: "user", content: userPrompt }],
        });

        // Extract text content
        const textBlock = response.content.find(
          (block) => block.type === "text"
        );
        if (!textBlock || textBlock.type !== "text") {
          throw new Error("No text content in Claude response");
        }

        return textBlock.text;
      } catch (error) {
        lastError = error as Error;
        if (attempt < MAX_RETRIES) {
          // Exponential backoff
          await new Promise((r) =>
            setTimeout(r, Math.pow(2, attempt) * 1000)
          );
        }
      }
    }

    throw new Error(`Claude API failed after ${MAX_RETRIES + 1} attempts: ${lastError?.message}`);
  }
}
