/**
 * Conversation Engine — stateful multi-turn dialogue manager.
 *
 * Implements the anti-vibe-coding flow: one question at a time,
 * structured topic progression, readiness tracking, plan confirmation.
 */

import type { DiscoveryBrief } from "../agents/types.js";
import { BriefBuilder } from "./brief-builder.js";
import { TOPICS } from "./topics.js";

export type ConversationPhase =
  | "discovery"
  | "confirmation"
  | "generating"
  | "complete";

export interface ConversationTurn {
  /** The AI's response (question, confirmation, or result). */
  message: string;
  /** Current conversation phase. */
  phase: ConversationPhase;
  /** Readiness score (0-100). */
  readiness: number;
  /** Progress through discovery topics (0.0 - 1.0). */
  progress: number;
  /** The topic being asked about (null if in confirmation/generating). */
  currentTopic: string | null;
  /** Whether the conversation is waiting for user input. */
  awaitingInput: boolean;
}

export class ConversationEngine {
  private briefBuilder = new BriefBuilder();
  private phase: ConversationPhase = "discovery";
  private currentTopicId: string | null = null;
  private turnCount = 0;
  private history: Array<{ role: "ai" | "user"; message: string }> = [];

  /** Minimum readiness to proceed to confirmation. */
  readinessThreshold = 75;

  /** Start the conversation — returns the first question. */
  start(): ConversationTurn {
    this.currentTopicId = this.briefBuilder.getNextTopic();
    const prompt = this.currentTopicId
      ? this.briefBuilder.getTopicPrompt(this.currentTopicId)
      : null;

    const message = prompt || "What would you like to build?";
    this.history.push({ role: "ai", message });

    return {
      message,
      phase: "discovery",
      readiness: 0,
      progress: 0,
      currentTopic: this.currentTopicId,
      awaitingInput: true,
    };
  }

  /** Process user input and return the next turn. */
  respond(userInput: string): ConversationTurn {
    this.turnCount++;
    this.history.push({ role: "user", message: userInput });

    if (this.phase === "discovery") {
      return this.handleDiscoveryInput(userInput);
    }

    if (this.phase === "confirmation") {
      return this.handleConfirmationInput(userInput);
    }

    return {
      message: "Generation is in progress.",
      phase: this.phase,
      readiness: this.briefBuilder.getReadiness(),
      progress: 1.0,
      currentTopic: null,
      awaitingInput: false,
    };
  }

  /** Get the accumulated brief. */
  getBrief(): DiscoveryBrief {
    return this.briefBuilder.getBrief();
  }

  /** Get full conversation history. */
  getHistory(): Array<{ role: "ai" | "user"; message: string }> {
    return [...this.history];
  }

  /** Serialize conversation state for session persistence. */
  toJSON(): object {
    return {
      phase: this.phase,
      turnCount: this.turnCount,
      brief: this.briefBuilder.getBrief(),
      history: this.history,
      readiness: this.briefBuilder.getReadiness(),
    };
  }

  private handleDiscoveryInput(input: string): ConversationTurn {
    // Record answer for current topic
    if (this.currentTopicId) {
      this.briefBuilder.addAnswer(this.currentTopicId, input);
    }

    const readiness = this.briefBuilder.getReadiness();
    const answeredCount = TOPICS.filter((t) =>
      this.briefBuilder.getBrief()[t.briefField] !== "" &&
      this.briefBuilder.getBrief()[t.briefField] !== undefined
    ).length;
    const progress = answeredCount / TOPICS.length;

    // Check if ready for confirmation
    if (readiness >= this.readinessThreshold && this.briefBuilder.isComplete()) {
      return this.transitionToConfirmation();
    }

    // Get next topic
    this.currentTopicId = this.briefBuilder.getNextTopic();

    if (!this.currentTopicId) {
      // All topics covered
      return this.transitionToConfirmation();
    }

    const prompt = this.briefBuilder.getTopicPrompt(this.currentTopicId)!;
    this.history.push({ role: "ai", message: prompt });

    return {
      message: prompt,
      phase: "discovery",
      readiness,
      progress,
      currentTopic: this.currentTopicId,
      awaitingInput: true,
    };
  }

  private transitionToConfirmation(): ConversationTurn {
    this.phase = "confirmation";
    const brief = this.briefBuilder.getBrief();

    const plan = `Here's what I'm planning to build:

${brief.purpose}

Sections:
${brief.sections.map((s, i) => `  ${i + 1}. ${s}`).join("\n")}

Call-to-action: ${brief.ctas.join(", ")}
Style: ${brief.tone}
${brief.constraints.length > 0 ? `Constraints: ${brief.constraints.join(", ")}` : ""}

Does this sound right? (yes to proceed, or describe what to change)`;

    this.history.push({ role: "ai", message: plan });

    return {
      message: plan,
      phase: "confirmation",
      readiness: this.briefBuilder.getReadiness(),
      progress: 1.0,
      currentTopic: null,
      awaitingInput: true,
    };
  }

  private handleConfirmationInput(input: string): ConversationTurn {
    const lower = input.toLowerCase().trim();
    const approved =
      lower === "yes" ||
      lower === "y" ||
      lower === "looks good" ||
      lower === "proceed" ||
      lower === "go" ||
      lower.startsWith("yes");

    if (approved) {
      this.phase = "generating";
      const message = "Building your page now...";
      this.history.push({ role: "ai", message });

      return {
        message,
        phase: "generating",
        readiness: this.briefBuilder.getReadiness(),
        progress: 1.0,
        currentTopic: null,
        awaitingInput: false,
      };
    }

    // User wants changes — go back to discovery for the mentioned topic
    this.phase = "discovery";
    const message =
      "Got it — what would you like to change? I'll update the plan.";
    this.history.push({ role: "ai", message });

    return {
      message,
      phase: "discovery",
      readiness: this.briefBuilder.getReadiness(),
      progress: 0.9,
      currentTopic: null,
      awaitingInput: true,
    };
  }
}
