/**
 * Voce IR AI Bridge — natural language to validated IR.
 *
 * @example
 * ```ts
 * import { AgentOrchestrator } from "@voce-ir/ai-bridge";
 *
 * const orchestrator = new AgentOrchestrator();
 * const result = await orchestrator.run("a SaaS landing page with hero, features, and pricing");
 *
 * if (result.followUpQuestions.length > 0) {
 *   console.log("Questions:", result.followUpQuestions);
 * } else if (result.success) {
 *   console.log(result.irJson);
 * }
 * ```
 */

// Core
export { ClaudeClient } from "./api/claude-client.js";
export { buildSchemaContext } from "./context/schema-context.js";
export { buildUserPrompt } from "./context/base-prompt.js";
export { IrGenerator, type GenerationResult } from "./generator/ir-generator.js";

// Multi-agent architecture
export { Agent } from "./agents/base-agent.js";
export { DiscoveryAgent } from "./agents/discovery-agent.js";
export { DesignAgent } from "./agents/design-agent.js";
export { GeneratorAgent } from "./agents/generator-agent.js";
export { RepairAgent } from "./agents/repair-agent.js";
export { AgentOrchestrator, type OrchestratorResult } from "./agents/orchestrator.js";
export type { DiscoveryBrief, DesignSpec, RepairResult, PipelineEvent } from "./agents/types.js";

// Conversational design
export { ConversationEngine, type ConversationTurn, type ConversationPhase } from "./conversation/engine.js";
export { BriefBuilder } from "./conversation/brief-builder.js";
export { TOPICS, calculateReadiness } from "./conversation/topics.js";
export { saveSession, loadLatestSession, loadSession } from "./conversation/session.js";

// Style packs & RAG
export type { StylePack, DesignTokens, PackExample } from "./packs/types.js";
export { getAllPacks, getPack, matchPack, formatPackList } from "./packs/loader.js";
export { retrieveExamples, type RetrievalResult } from "./rag/index.js";
export { buildFewShotContext, buildTokenContext } from "./rag/few-shot.js";

// Memory & decisions
export { ensureVoceDir, vocePath, hasVoceDir } from "./memory/directory.js";
export { saveBrief, loadBrief, hasBrief, type SavedBrief } from "./memory/brief.js";
export { appendDecision, listDecisions, findConflicts, type Decision } from "./memory/decisions.js";
export { enforceBreif as enforceBrief, type EnforcementResult } from "./memory/enforcement.js";
export { detectDrift, driftStatus, type DriftReport } from "./memory/drift.js";

// Incremental generation
export { applyPatch, invertPatch, type VocePatch, type PatchOperation } from "./incremental/diff.js";
export { savePatch, loadPatch, listPatches, latestPatchNum } from "./incremental/history.js";
export { PatchAgent } from "./agents/patch-agent.js";
export { buildSessionContext, type SessionContext } from "./context/session-context.js";

// Configuration
export { loadConfig, resolveApiKey, generateConfigTemplate, type VoceConfig } from "./config/provider.js";

// Voice interface
export { getSttProvider, type SttProvider, type Transcript } from "./voice/stt.js";
export { getTtsProvider, type TtsProvider } from "./voice/tts.js";
export { PushToTalkEngine, type VoiceState, type InputMode, type VoiceEvent } from "./voice/ptt-engine.js";
export { tuneForVoice, isVoiceAppropriate, truncateForVoice } from "./voice/voice-prompts.js";

// 3D scene generation
export { resolveMaterial, getMaterialPresets, type PbrParams } from "./three_d/materials.js";
export { resolveLighting, getLightingPresets, type LightConfig, type LightingSetup } from "./three_d/lighting.js";
export { resolveCamera, type CameraConfig } from "./three_d/camera.js";
export { build3DContext } from "./three_d/index.js";
