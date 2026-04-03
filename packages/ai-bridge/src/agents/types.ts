/**
 * Typed intermediate objects passed between agents.
 */

/** Output of the Discovery Agent — structured requirements. */
export interface DiscoveryBrief {
  /** What is being built. */
  purpose: string;
  /** Who the target audience is. */
  audience: string;
  /** Key page sections in order. */
  sections: string[];
  /** Call-to-action descriptions. */
  ctas: string[];
  /** Visual tone (e.g., "dark and bold", "clean and minimal"). */
  tone: string;
  /** Technical constraints (e.g., "needs form", "no auth"). */
  constraints: string[];
  /** Readiness score 0-100. Below 70 = need more info. */
  readinessScore: number;
  /** Follow-up questions if readiness is low. */
  followUpQuestions: string[];
}

/** Output of the Design Agent — concrete design decisions. */
export interface DesignSpec {
  /** Page layout structure. */
  layout: {
    type: "landing" | "dashboard" | "article" | "form" | "custom";
    sections: SectionSpec[];
  };
  /** Color palette as RGB values. */
  colors: {
    background: ColorRGB;
    foreground: ColorRGB;
    primary: ColorRGB;
    surface: ColorRGB;
    muted: ColorRGB;
  };
  /** Typography choices. */
  typography: {
    headingSize: number;
    bodySize: number;
    headingWeight: string;
    bodyWeight: string;
  };
  /** Spacing base unit in px. */
  spacingBase: number;
}

export interface SectionSpec {
  name: string;
  type: "hero" | "features" | "testimonials" | "pricing" | "cta" | "form" | "footer" | "header" | "content";
  children: string[];
}

export interface ColorRGB {
  r: number;
  g: number;
  b: number;
}

/** Result of the Repair Agent. */
export interface RepairResult {
  /** The corrected IR JSON. */
  irJson: string;
  /** Whether all errors were fixed. */
  allFixed: boolean;
  /** Remaining errors after repair. */
  remainingErrors: string[];
  /** Number of repair cycles used. */
  cyclesUsed: number;
}

/** Event emitted by the orchestrator for progress tracking. */
export interface PipelineEvent {
  phase: "discovery" | "design" | "generation" | "validation" | "repair" | "complete";
  status: "started" | "completed" | "failed";
  message: string;
  tokensUsed?: number;
}
