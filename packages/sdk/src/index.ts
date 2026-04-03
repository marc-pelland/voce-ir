/**
 * Voce IR SDK — programmatic access to IR generation, validation, and compilation.
 *
 * @example
 * ```ts
 * import { VoceClient } from "@voce-ir/sdk";
 *
 * const voce = new VoceClient();
 *
 * const ir = await voce.generate("a landing page with hero and features");
 * const validation = await voce.validate(ir);
 * const html = await voce.compile(ir);
 * ```
 */

import { execSync } from "node:child_process";
import { writeFileSync, readFileSync, mkdirSync, existsSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

export interface ValidationResult {
  valid: boolean;
  errors: number;
  warnings: number;
  diagnostics: Array<{
    severity: string;
    code: string;
    message: string;
    path: string;
  }>;
}

export interface CompileResult {
  html: string;
  sizeBytes: number;
}

export class VoceClient {
  private tmpDir: string;

  constructor() {
    this.tmpDir = join(tmpdir(), "voce-sdk");
    if (!existsSync(this.tmpDir)) {
      mkdirSync(this.tmpDir, { recursive: true });
    }
  }

  /** Validate IR JSON against all quality rules. */
  async validate(irJson: string): Promise<ValidationResult> {
    const tmpFile = this.writeTmp("validate", irJson);
    try {
      execSync(`voce validate --format json "${tmpFile}"`, {
        encoding: "utf-8",
        stdio: ["pipe", "pipe", "pipe"],
      });
      return { valid: true, errors: 0, warnings: 0, diagnostics: [] };
    } catch (error: unknown) {
      const err = error as { stdout?: string };
      try {
        const result = JSON.parse(err.stdout || "{}");
        return {
          valid: result.valid ?? false,
          errors: result.errors ?? 0,
          warnings: result.warnings ?? 0,
          diagnostics: result.diagnostics ?? [],
        };
      } catch {
        return { valid: false, errors: 1, warnings: 0, diagnostics: [] };
      }
    }
  }

  /** Compile IR JSON to HTML. */
  async compile(irJson: string): Promise<CompileResult> {
    const tmpFile = this.writeTmp("compile", irJson);
    const outFile = join(this.tmpDir, "output.html");

    execSync(`voce compile "${tmpFile}" -o "${outFile}"`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });

    const html = readFileSync(outFile, "utf-8");
    return { html, sizeBytes: html.length };
  }

  /** Generate IR from a natural language prompt. Requires ANTHROPIC_API_KEY. */
  async generate(prompt: string): Promise<string> {
    const output = execSync(
      `voce generate "${prompt.replace(/"/g, '\\"')}"`,
      { encoding: "utf-8", timeout: 60000, stdio: ["pipe", "pipe", "pipe"] }
    );
    return output;
  }

  /** Get a structured summary of an IR document. */
  async inspect(irJson: string): Promise<string> {
    const tmpFile = this.writeTmp("inspect", irJson);
    return execSync(`voce inspect "${tmpFile}"`, {
      encoding: "utf-8",
      stdio: ["pipe", "pipe", "pipe"],
    });
  }

  /** Get the Voce IR schema documentation. */
  getSchema(): string {
    // Use the schema context from the AI bridge
    try {
      const { buildSchemaContext } = require("@voce-ir/ai-bridge");
      return buildSchemaContext();
    } catch {
      return "Schema context not available. Install @voce-ir/ai-bridge.";
    }
  }

  private writeTmp(prefix: string, content: string): string {
    const path = join(this.tmpDir, `${prefix}-${Date.now()}.voce.json`);
    writeFileSync(path, content);
    return path;
  }
}
