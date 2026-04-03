/**
 * Style pack type definitions.
 *
 * A style pack contains design tokens, layout patterns, and
 * example IR files. It provides a distinct visual identity
 * for AI-generated output.
 */

export interface StylePack {
  /** Pack identifier (e.g., "minimal-saas"). */
  id: string;
  /** Display name (e.g., "Minimal SaaS"). */
  name: string;
  /** One-line description. */
  description: string;
  /** Tags for matching (e.g., ["saas", "landing", "clean"]). */
  tags: string[];
  /** Design tokens. */
  tokens: DesignTokens;
  /** Example IR file descriptions for RAG matching. */
  examples: PackExample[];
}

export interface DesignTokens {
  colors: {
    background: RGB;
    foreground: RGB;
    primary: RGB;
    surface: RGB;
    muted: RGB;
    accent?: RGB;
  };
  typography: {
    headingFamily: string;
    bodyFamily: string;
    headingSize: number;
    bodySize: number;
    headingWeight: string;
    lineHeight: number;
  };
  spacing: {
    base: number;
    scale: number[];
  };
  radii: {
    small: number;
    medium: number;
    large: number;
  };
}

export interface RGB {
  r: number;
  g: number;
  b: number;
}

export interface PackExample {
  /** Filename within the pack's examples/ directory. */
  filename: string;
  /** Natural language description for RAG matching. */
  description: string;
  /** Tags for matching. */
  tags: string[];
  /** The IR JSON content. */
  irJson?: string;
}
