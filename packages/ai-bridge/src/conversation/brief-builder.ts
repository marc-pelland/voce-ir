/**
 * Brief builder — incrementally constructs a DiscoveryBrief from
 * conversation answers. Each answer is slotted into the appropriate field.
 */

import type { DiscoveryBrief } from "../agents/types.js";
import { TOPICS, calculateReadiness } from "./topics.js";

export class BriefBuilder {
  private brief: DiscoveryBrief = {
    purpose: "",
    audience: "",
    sections: [],
    ctas: [],
    tone: "clean and professional",
    constraints: [],
    readinessScore: 0,
    followUpQuestions: [],
  };

  private answeredTopics = new Set<string>();

  /** Record an answer for a specific topic. */
  addAnswer(topicId: string, answer: string): void {
    const topic = TOPICS.find((t) => t.id === topicId);
    if (!topic) return;

    this.answeredTopics.add(topicId);

    switch (topic.briefField) {
      case "purpose":
        this.brief.purpose = answer;
        break;
      case "audience":
        this.brief.audience = answer;
        break;
      case "sections":
        this.brief.sections = parseSections(answer);
        break;
      case "ctas":
        this.brief.ctas = answer.split(",").map((s) => s.trim()).filter(Boolean);
        if (this.brief.ctas.length === 0) {
          this.brief.ctas = [answer.trim()];
        }
        break;
      case "tone":
        this.brief.tone = answer;
        break;
      case "constraints":
        this.brief.constraints = answer.split(",").map((s) => s.trim()).filter(Boolean);
        if (this.brief.constraints.length === 0 && answer.trim()) {
          this.brief.constraints = [answer.trim()];
        }
        break;
    }

    this.brief.readinessScore = calculateReadiness(this.answeredTopics);
  }

  /** Get the next unanswered topic, or null if all covered. */
  getNextTopic(): string | null {
    for (const topic of TOPICS) {
      if (!this.answeredTopics.has(topic.id)) {
        return topic.id;
      }
    }
    return null;
  }

  /** Get the current readiness score. */
  getReadiness(): number {
    return this.brief.readinessScore;
  }

  /** Get the accumulated brief. */
  getBrief(): DiscoveryBrief {
    return { ...this.brief };
  }

  /** Check if all required topics are answered. */
  isComplete(): boolean {
    return TOPICS.filter((t) => !t.optional).every((t) =>
      this.answeredTopics.has(t.id)
    );
  }

  /** Get topic prompt by ID. */
  getTopicPrompt(topicId: string): string | null {
    return TOPICS.find((t) => t.id === topicId)?.prompt ?? null;
  }

  /** Export as YAML-like string for .voce/brief.yaml. */
  toYaml(): string {
    const b = this.brief;
    return `project:
  name: "${b.purpose.split(" ").slice(0, 5).join(" ")}"
  version: 1

vision: "${b.purpose}"

target_audience:
  primary: "${b.audience}"

success_criteria:
${b.sections.map((s) => `  - "${s} section implemented"`).join("\n")}

non_negotiables:
  - "Accessibility — WCAG AA"
  - "Performance — <10KB output"

style_direction:
  feel: "${b.tone}"

features:
${b.sections.map((s, i) => `  - id: "F${String(i + 1).padStart(3, "0")}"\n    name: "${s}"\n    status: "planned"\n    priority: "must-have"`).join("\n")}
`;
  }
}

/** Parse section names from a natural language answer. */
function parseSections(answer: string): string[] {
  // Handle comma-separated, period-separated, or line-separated
  const sections = answer
    .split(/[,\n]/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0)
    .map((s) => s.replace(/^[-•*]\s*/, "")); // Remove bullet markers

  return sections.length > 0 ? sections : [answer.trim()];
}
