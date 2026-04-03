/**
 * Topic graph — the ordered set of questions the Discovery conversation
 * walks through. One topic = one question = one turn.
 */

import type { DiscoveryBrief } from "../agents/types.js";

export interface Topic {
  /** Unique identifier. */
  id: string;
  /** Which brief field this topic populates. */
  briefField: keyof DiscoveryBrief;
  /** Prompt template. The AI asks this (paraphrased, not verbatim). */
  prompt: string;
  /** Weight toward readiness score (0-25). */
  weight: number;
  /** Whether this topic can be skipped. */
  optional: boolean;
}

/**
 * Ordered topic list. The conversation progresses through these
 * one at a time. Each answered topic increases the readiness score.
 */
export const TOPICS: Topic[] = [
  {
    id: "purpose",
    briefField: "purpose",
    prompt: "What are you building? Tell me about the product or service — what problem does it solve?",
    weight: 25,
    optional: false,
  },
  {
    id: "audience",
    briefField: "audience",
    prompt: "Who is the target audience? Who will be visiting this page?",
    weight: 15,
    optional: false,
  },
  {
    id: "sections",
    briefField: "sections",
    prompt: "What sections should the page have? For example: hero, features, pricing, testimonials, contact form.",
    weight: 20,
    optional: false,
  },
  {
    id: "ctas",
    briefField: "ctas",
    prompt: "What should the main call-to-action be? What do you want visitors to do?",
    weight: 15,
    optional: false,
  },
  {
    id: "tone",
    briefField: "tone",
    prompt: "What visual style are you going for? Dark and bold? Clean and minimal? Is there a website whose look you admire?",
    weight: 15,
    optional: true,
  },
  {
    id: "constraints",
    briefField: "constraints",
    prompt: "Any technical constraints I should know about? Forms, authentication, specific integrations?",
    weight: 10,
    optional: true,
  },
];

/** Calculate readiness score from answered topics. */
export function calculateReadiness(answeredTopicIds: Set<string>): number {
  let score = 0;
  for (const topic of TOPICS) {
    if (answeredTopicIds.has(topic.id)) {
      score += topic.weight;
    }
  }
  return Math.min(score, 100);
}
