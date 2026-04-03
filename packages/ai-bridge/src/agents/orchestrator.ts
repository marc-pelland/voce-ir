/**
 * Agent Orchestrator — chains Discovery → Design → Generator → Validate → Repair.
 *
 * The orchestrator manages context passing between agents and enforces
 * the quality gate (Discovery must pass before generation starts).
 */

import { ClaudeClient } from "../api/claude-client.js";
import { DiscoveryAgent } from "./discovery-agent.js";
import { DesignAgent } from "./design-agent.js";
import { GeneratorAgent } from "./generator-agent.js";
import { RepairAgent } from "./repair-agent.js";
import type {
  DiscoveryBrief,
  DesignSpec,
  PipelineEvent,
  RepairResult,
} from "./types.js";

export interface OrchestratorResult {
  success: boolean;
  irJson: string | null;
  brief: DiscoveryBrief | null;
  design: DesignSpec | null;
  repairResult: RepairResult | null;
  /** If discovery gate failed, these are the questions to ask the user. */
  followUpQuestions: string[];
  events: PipelineEvent[];
}

export class AgentOrchestrator {
  private discoveryAgent: DiscoveryAgent;
  private designAgent: DesignAgent;
  private generatorAgent: GeneratorAgent;
  private repairAgent: RepairAgent;
  private events: PipelineEvent[] = [];

  /** Minimum readiness score to proceed past discovery. */
  readinessThreshold = 70;

  constructor(apiKey?: string) {
    const client = new ClaudeClient(apiKey);
    this.discoveryAgent = new DiscoveryAgent(client);
    this.designAgent = new DesignAgent(client);
    this.generatorAgent = new GeneratorAgent(client);
    this.repairAgent = new RepairAgent(client);
  }

  /**
   * Run the full pipeline: prompt → validated IR JSON.
   *
   * If the prompt is too vague (readiness < threshold), returns
   * follow-up questions instead of generating.
   */
  async run(prompt: string): Promise<OrchestratorResult> {
    this.events = [];

    // Phase 1: Discovery
    this.emit("discovery", "started", "Analyzing requirements...");
    const brief = await this.discoveryAgent.execute(prompt);
    this.emit("discovery", "completed", `Readiness: ${brief.readinessScore}/100`);

    // Quality gate
    if (brief.readinessScore < this.readinessThreshold) {
      return {
        success: false,
        irJson: null,
        brief,
        design: null,
        repairResult: null,
        followUpQuestions: brief.followUpQuestions,
        events: this.events,
      };
    }

    // Phase 2: Design
    this.emit("design", "started", "Designing layout and visual system...");
    const design = await this.designAgent.execute(brief);
    this.emit(
      "design",
      "completed",
      `${design.layout.sections.length} sections, ${design.layout.type} layout`
    );

    // Phase 3: Generation
    this.emit("generation", "started", "Generating IR...");
    const irJson = await this.generatorAgent.execute({ brief, design });
    this.emit("generation", "completed", `Generated ${irJson.length} chars`);

    // Phase 4: Validation + Repair
    this.emit("validation", "started", "Validating IR...");
    const repairResult = await this.repairAgent.repairLoop(irJson);

    if (repairResult.allFixed) {
      this.emit("validation", "completed", "Valid — 0 errors");
    } else {
      this.emit(
        "repair",
        "completed",
        `${repairResult.remainingErrors.length} errors remaining after ${repairResult.cyclesUsed} repair cycles`
      );
    }

    this.emit("complete", "completed", "Pipeline complete");

    return {
      success: repairResult.allFixed,
      irJson: repairResult.irJson,
      brief,
      design,
      repairResult,
      followUpQuestions: [],
      events: this.events,
    };
  }

  private emit(
    phase: PipelineEvent["phase"],
    status: PipelineEvent["status"],
    message: string
  ) {
    this.events.push({ phase, status, message });
  }
}
