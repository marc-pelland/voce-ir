/**
 * Base agent class — typed input/output with system prompt and Claude API call.
 */

import { ClaudeClient, type GenerateOptions } from "../api/claude-client.js";

export abstract class Agent<TInput, TOutput> {
  protected client: ClaudeClient;
  protected options: GenerateOptions;

  /** Human-readable name for logging. */
  abstract name: string;

  /** System prompt for this agent's role. */
  abstract systemPrompt: string;

  constructor(client: ClaudeClient, options: GenerateOptions = {}) {
    this.client = client;
    this.options = options;
  }

  /** Build the user prompt from typed input. */
  abstract buildUserPrompt(input: TInput): string;

  /** Parse the Claude response into typed output. */
  abstract parseResponse(response: string): TOutput;

  /** Execute the agent: build prompt → call Claude → parse response. */
  async execute(input: TInput): Promise<TOutput> {
    const userPrompt = this.buildUserPrompt(input);
    const response = await this.client.generate(
      this.systemPrompt,
      userPrompt,
      this.options
    );
    return this.parseResponse(response);
  }
}
