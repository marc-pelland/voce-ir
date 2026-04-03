/**
 * IR Generator — orchestrates prompt construction, Claude API call,
 * JSON extraction, and validation via voce CLI.
 */

import { execSync } from "node:child_process";
import { writeFileSync, mkdirSync, existsSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { ClaudeClient, type GenerateOptions } from "../api/claude-client.js";
import { buildSchemaContext } from "../context/schema-context.js";
import { buildUserPrompt } from "../context/base-prompt.js";

export interface GenerationResult {
  success: boolean;
  irJson: string | null;
  htmlPath: string | null;
  htmlSize: number | null;
  validationErrors: string[];
  rawResponse: string;
}

export class IrGenerator {
  private client: ClaudeClient;
  private schemaContext: string;

  constructor(apiKey?: string) {
    this.client = new ClaudeClient(apiKey);
    this.schemaContext = buildSchemaContext();
  }

  /**
   * Generate IR from a natural language prompt.
   * Validates and optionally compiles the result.
   */
  async generate(
    intent: string,
    options: GenerateOptions & { compile?: boolean } = {}
  ): Promise<GenerationResult> {
    const userPrompt = buildUserPrompt(intent);

    // Call Claude
    const rawResponse = await this.client.generate(
      this.schemaContext,
      userPrompt,
      options
    );

    // Extract JSON from response (Claude might wrap in code fences)
    const irJson = extractJson(rawResponse);
    if (!irJson) {
      return {
        success: false,
        irJson: null,
        htmlPath: null,
        htmlSize: null,
        validationErrors: ["Failed to extract valid JSON from Claude response"],
        rawResponse,
      };
    }

    // Write to temp file
    const outputDir = join(tmpdir(), "voce-generate");
    if (!existsSync(outputDir)) {
      mkdirSync(outputDir, { recursive: true });
    }
    const irPath = join(outputDir, "generated.voce.json");
    writeFileSync(irPath, irJson);

    // Validate
    const validationErrors: string[] = [];
    try {
      execSync(`voce validate --format json "${irPath}"`, {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
    } catch (error: unknown) {
      // Parse validation errors from JSON output
      try {
        const err = error as { stdout?: string };
        const result = JSON.parse(err.stdout || "{}");
        if (result.diagnostics) {
          for (const d of result.diagnostics) {
            if (d.severity === "error") {
              validationErrors.push(`${d.code}: ${d.message} (at ${d.path})`);
            }
          }
        }
      } catch {
        validationErrors.push("Validation failed (could not parse output)");
      }
    }

    // Compile if requested and valid
    let htmlPath: string | null = null;
    let htmlSize: number | null = null;

    if (options.compile !== false && validationErrors.length === 0) {
      try {
        const compiledPath = join(outputDir, "index.html");
        const output = execSync(
          `voce compile "${irPath}" -o "${compiledPath}"`,
          { encoding: "utf-8", stdio: ["pipe", "pipe", "pipe"] }
        );
        htmlPath = compiledPath;

        // Extract size from compile output
        const sizeMatch = output.match(/\((\d+) bytes\)/);
        if (sizeMatch) {
          htmlSize = parseInt(sizeMatch[1], 10);
        }
      } catch {
        // Compilation failed but that's ok — IR was valid
      }
    }

    return {
      success: validationErrors.length === 0,
      irJson,
      htmlPath,
      htmlSize,
      validationErrors,
      rawResponse,
    };
  }
}

/**
 * Extract JSON from a Claude response that might contain code fences
 * or other wrapping text.
 */
function extractJson(text: string): string | null {
  // Try direct parse first
  try {
    JSON.parse(text);
    return text;
  } catch {
    // Not raw JSON
  }

  // Try extracting from code fences
  const fenceMatch = text.match(/```(?:json)?\s*\n([\s\S]*?)\n```/);
  if (fenceMatch) {
    try {
      JSON.parse(fenceMatch[1]);
      return fenceMatch[1];
    } catch {
      // Not valid JSON in fence
    }
  }

  // Try finding the first { ... } block
  const firstBrace = text.indexOf("{");
  const lastBrace = text.lastIndexOf("}");
  if (firstBrace !== -1 && lastBrace > firstBrace) {
    const candidate = text.slice(firstBrace, lastBrace + 1);
    try {
      JSON.parse(candidate);
      return candidate;
    } catch {
      // Not valid JSON
    }
  }

  return null;
}
