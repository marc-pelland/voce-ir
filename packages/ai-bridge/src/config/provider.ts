/**
 * Provider configuration — loads settings from voce.config.toml.
 *
 * Supports: API key resolution (from env vars), model selection,
 * default style pack, and generation options.
 */

import { existsSync, readFileSync } from "node:fs";

export interface VoceConfig {
  ai: {
    provider: string;
    model: string;
    apiKeyEnv: string;
    temperature: number;
    maxTokens: number;
  };
  defaults: {
    stylePack: string | null;
    target: string;
  };
}

const DEFAULT_CONFIG: VoceConfig = {
  ai: {
    provider: "claude",
    model: "claude-sonnet-4-20250514",
    apiKeyEnv: "ANTHROPIC_API_KEY",
    temperature: 0.3,
    maxTokens: 8192,
  },
  defaults: {
    stylePack: null,
    target: "dom",
  },
};

/**
 * Load configuration from voce.config.toml in the project root.
 * Falls back to defaults if not found.
 */
export function loadConfig(root: string = "."): VoceConfig {
  const configPath = `${root}/voce.config.toml`;

  if (!existsSync(configPath)) {
    return { ...DEFAULT_CONFIG };
  }

  try {
    const content = readFileSync(configPath, "utf-8");
    return parseToml(content);
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Get the API key from the configured environment variable.
 */
export function resolveApiKey(config: VoceConfig): string | undefined {
  return process.env[config.ai.apiKeyEnv];
}

/**
 * Generate a default voce.config.toml template.
 */
export function generateConfigTemplate(): string {
  return `# Voce IR Configuration
# See docs for full options: https://voce-ir.xyz/docs/config

[ai]
provider = "claude"
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
temperature = 0.3
max_tokens = 8192

[defaults]
style_pack = ""  # "minimal-saas", "editorial", or "ecommerce"
target = "dom"

[voice]
enabled = false
stt_provider = "whisper"
tts_provider = "none"
`;
}

/** Simple TOML parser for our known config format. */
function parseToml(content: string): VoceConfig {
  const config = { ...DEFAULT_CONFIG };
  const lines = content.split("\n");
  let section = "";

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith("#") || trimmed === "") continue;

    const sectionMatch = trimmed.match(/^\[(\w+(?:\.\w+)*)\]$/);
    if (sectionMatch) {
      section = sectionMatch[1];
      continue;
    }

    const kvMatch = trimmed.match(/^(\w+)\s*=\s*"?([^"]*)"?$/);
    if (!kvMatch) continue;

    const [, key, value] = kvMatch;

    if (section === "ai") {
      switch (key) {
        case "provider": config.ai.provider = value; break;
        case "model": config.ai.model = value; break;
        case "api_key_env": config.ai.apiKeyEnv = value; break;
        case "temperature": config.ai.temperature = parseFloat(value); break;
        case "max_tokens": config.ai.maxTokens = parseInt(value, 10); break;
      }
    } else if (section === "defaults") {
      switch (key) {
        case "style_pack": config.defaults.stylePack = value || null; break;
        case "target": config.defaults.target = value; break;
      }
    }
  }

  return config;
}
