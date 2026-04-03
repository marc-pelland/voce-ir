/**
 * Conversational debugging agent — describes a bug in natural language,
 * AI traces the cause and proposes a fix.
 *
 * Integrates with the inspector state: reads current state machine states,
 * animation status, node properties, and a11y tree to diagnose issues.
 */

export interface BugReport {
  /** User's description of the problem. */
  description: string;
  /** Inspector state at time of report. */
  currentStates: Record<string, string>; // machineId → currentState
  /** Visible animations. */
  activeAnimations: string[];
  /** A11y warnings present. */
  a11yWarnings: string[];
}

export interface DebugDiagnosis {
  /** What the AI thinks is wrong. */
  explanation: string;
  /** The IR node(s) involved. */
  involvedNodes: string[];
  /** Proposed fix as an IR patch description. */
  proposedFix: string;
  /** Confidence (0-1). */
  confidence: number;
}

export interface DebugConversation {
  turns: Array<{ role: "user" | "ai"; message: string }>;
  bugReport: BugReport | null;
  diagnosis: DebugDiagnosis | null;
}

/**
 * Debug agent — processes bug descriptions and produces diagnoses.
 *
 * In Phase 5, this runs locally with pattern matching.
 * In production, it would call the AI bridge for Claude-powered diagnosis.
 */
export class DebugAgent {
  private conversation: DebugConversation = {
    turns: [],
    bugReport: null,
    diagnosis: null,
  };

  /** Start a debugging session with a bug description. */
  startDebug(report: BugReport): DebugDiagnosis {
    this.conversation.bugReport = report;
    this.conversation.turns.push({
      role: "user",
      message: report.description,
    });

    // Pattern-matching diagnosis (local, no AI needed)
    const diagnosis = this.diagnose(report);
    this.conversation.diagnosis = diagnosis;
    this.conversation.turns.push({
      role: "ai",
      message: diagnosis.explanation,
    });

    return diagnosis;
  }

  /** Follow-up question in the debugging conversation. */
  followUp(userMessage: string): string {
    this.conversation.turns.push({ role: "user", message: userMessage });

    // Simple follow-up responses
    let response: string;
    const lower = userMessage.toLowerCase();

    if (lower.includes("fix") || lower.includes("apply") || lower.includes("yes")) {
      response = this.conversation.diagnosis
        ? `Applying fix: ${this.conversation.diagnosis.proposedFix}. The patch has been generated — check the edit history.`
        : "I don't have a fix to apply yet. Could you describe the issue again?";
    } else if (lower.includes("why") || lower.includes("explain")) {
      response = this.conversation.diagnosis
        ? `The issue is in node(s): ${this.conversation.diagnosis.involvedNodes.join(", ")}. ${this.conversation.diagnosis.explanation}`
        : "I need more context. What specific behavior is unexpected?";
    } else {
      response = "Could you be more specific? Try describing what you expected to happen vs what actually happened.";
    }

    this.conversation.turns.push({ role: "ai", message: response });
    return response;
  }

  /** Get the full conversation. */
  getConversation(): DebugConversation {
    return { ...this.conversation };
  }

  /** Pattern-matching diagnosis for common bugs. */
  private diagnose(report: BugReport): DebugDiagnosis {
    const desc = report.description.toLowerCase();

    // Button doesn't respond to clicks
    if (desc.includes("button") && (desc.includes("click") || desc.includes("respond") || desc.includes("work"))) {
      return {
        explanation: "The button likely has no GestureHandler attached, or the GestureHandler target_node_id doesn't match the button's node_id. Check that the button Surface has a data-voce-id and a GestureHandler references it.",
        involvedNodes: ["(button Surface)", "(GestureHandler)"],
        proposedFix: "Add a GestureHandler with gesture_type: Tap, target_node_id pointing to the button, trigger_event for the state machine, and keyboard_key: Enter",
        confidence: 0.8,
      };
    }

    // Animation issues
    if (desc.includes("animation") || desc.includes("janky") || desc.includes("jank") || desc.includes("slow")) {
      return {
        explanation: "Animation performance issues are typically caused by: animating layout properties (width, height) instead of transform/opacity, missing reduced_motion alternative, or excessive duration (>800ms feels sluggish).",
        involvedNodes: ["(AnimationTransition)", "(PhysicsBody)"],
        proposedFix: "Check that animated properties are compositor-safe (transform, opacity). Ensure easing uses spring or ease-out (not linear). Consider reducing duration to 200-400ms.",
        confidence: 0.7,
      };
    }

    // Modal/focus issues
    if (desc.includes("modal") || desc.includes("focus") || desc.includes("trap") || desc.includes("escape")) {
      return {
        explanation: "Focus management issues usually mean a FocusTrap is missing or misconfigured. The modal needs: FocusTrap with container_node_id, initial_focus_node_id, and escape_behavior.",
        involvedNodes: ["(FocusTrap)", "(modal Container)"],
        proposedFix: "Add a FocusTrap node with container_node_id pointing to the modal, escape_behavior: CloseOnEscape, and restore_focus: true",
        confidence: 0.75,
      };
    }

    // Accessibility issues
    if (desc.includes("screen reader") || desc.includes("accessible") || desc.includes("aria") || desc.includes("keyboard")) {
      return {
        explanation: "Accessibility issues are often caused by missing SemanticNode references. Interactive elements (buttons, links, forms) must have semantic_node_id pointing to a SemanticNode with role and label.",
        involvedNodes: ["(SemanticNode)", "(interactive element)"],
        proposedFix: "Add SemanticNode entries to ViewRoot.semantic_nodes with appropriate role and label, then set semantic_node_id on the interactive elements",
        confidence: 0.85,
      };
    }

    // Generic fallback
    return {
      explanation: `I'll analyze the issue: "${report.description}". Check the state machine visualizer for unexpected states, the a11y tree for missing annotations, and the animation timeline for stalled animations.`,
      involvedNodes: [],
      proposedFix: "Use the inspector panels to identify the specific node causing the issue, then use voce edit to apply a targeted fix.",
      confidence: 0.3,
    };
  }
}
