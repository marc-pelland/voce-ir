// Tool definitions — single source of truth for the schemas presented to
// any consumer (MCP clients, the cli-chat tool-use loop, future surfaces).
// Descriptions encode Voce's conversational pillars; the budget is roughly
// 170 bytes per tool — see prior commits for the rationale.

import type { ToolDefinition } from "./types.js";

export const TOOL_DEFINITIONS: readonly ToolDefinition[] = [
  // ── Pipeline tools ─────────────────────────────────────────────
  {
    name: "voce_validate",
    description:
      "Validate a Voce IR document. Returns per-pass diagnostics (severity, code, path, hint). Run before compile — a11y, security, SEO are errors in Voce, not warnings. Fix every error before declaring IR done.",
    inputSchema: {
      type: "object",
      properties: { ir_json: { type: "string", description: "Voce IR JSON to validate" } },
      required: ["ir_json"],
    },
  },
  {
    name: "voce_compile",
    description:
      "Compile validated Voce IR to HTML. Run only after voce_validate passes — never present output from invalid IR as final. Result has zero runtime JS.",
    inputSchema: {
      type: "object",
      properties: { ir_json: { type: "string", description: "Voce IR JSON to compile" } },
      required: ["ir_json"],
    },
  },
  {
    name: "voce_inspect",
    description:
      "Structured summary of an IR document — node counts, semantic tree, features. Run before compile to confirm intent and spot missing pillars (semantics, error/loading/empty states).",
    inputSchema: {
      type: "object",
      properties: { ir_json: { type: "string", description: "Voce IR JSON to inspect" } },
      required: ["ir_json"],
    },
  },
  {
    name: "voce_schema",
    description:
      "Schema docs for a Voce IR node type, or all types if omitted. Covers layout, state, motion, navigation, a11y, theming, data, forms, SEO, i18n.",
    inputSchema: {
      type: "object",
      properties: {
        node_type: {
          type: "string",
          description: "Node type, e.g. Container, FormNode. Omit for the overview.",
        },
      },
    },
  },
  {
    name: "voce_examples",
    description:
      "List or retrieve reference IR that compiles and validates cleanly. Start from an example and modify, rather than authoring from scratch.",
    inputSchema: {
      type: "object",
      properties: {
        name: { type: "string", description: "Example name, e.g. landing-page. Omit to list all." },
      },
    },
  },
  {
    name: "voce_generate",
    description:
      "One-shot legacy entry. Prefer the workflow: voce_generate_start → answer (×N) → propose → refine → finalize. Skipping discovery violates the conversational pillars.",
    inputSchema: {
      type: "object",
      properties: {
        prompt: { type: "string", description: "Concrete brief built from prior discovery turns" },
      },
      required: ["prompt"],
    },
  },
  // ── Memory tools ───────────────────────────────────────────────
  {
    name: "voce_brief_get",
    description:
      "Read the project brief — the north star every generation is checked against. Returns null content when no brief exists yet.",
    inputSchema: { type: "object", properties: {} },
  },
  {
    name: "voce_brief_set",
    description:
      "Replace the project brief with new markdown. Atomic write. Confirm with the user BEFORE invoking — overwriting the brief is consequential and the storage layer does not prompt.",
    inputSchema: {
      type: "object",
      properties: { brief_md: { type: "string", description: "New brief content (markdown)" } },
      required: ["brief_md"],
    },
  },
  {
    name: "voce_decisions_list",
    description:
      "List decisions, oldest first. Read this before proposing IR — drift checks reference these entries. Optional ISO-8601 since filter.",
    inputSchema: {
      type: "object",
      properties: { since: { type: "string", description: "ISO-8601 cutoff (inclusive)" } },
    },
  },
  {
    name: "voce_decisions_log",
    description:
      "Append a decision to the log. Use when the conversation produces a durable choice (architecture, scope, anti-pattern). Include rationale; future drift checks reference it.",
    inputSchema: {
      type: "object",
      properties: {
        summary: { type: "string", description: "One-line decision summary" },
        rationale: { type: "string", description: "Why this decision was made" },
        supersedes: { type: "string", description: "id of a prior decision this replaces" },
        conflicts_with: { type: "string", description: "id of a decision this knowingly overrides" },
      },
      required: ["summary", "rationale"],
    },
  },
  {
    name: "voce_session_resume",
    description:
      "Resume a prior session — returns conversation entries, the most recent ir_snapshot as current_ir, and the last decision id. Pass session_id to target a specific session, or omit for the most recent.",
    inputSchema: {
      type: "object",
      properties: {
        session_id: { type: "string", description: "Specific session id; omit for most recent" },
      },
    },
  },
  {
    name: "voce_check_drift",
    description:
      "Detect potential conflicts between a proposed IR and prior decisions. v1 is a keyword heuristic — false positives expected, judge each report. Run before declaring an IR final.",
    inputSchema: {
      type: "object",
      properties: { proposed_ir: { type: "string", description: "Voce IR JSON to check" } },
      required: ["proposed_ir"],
    },
  },
  // ── Generation workflow ───────────────────────────────────────
  {
    name: "voce_generate_start",
    description:
      "Open a generation session. Records user_intent, returns session_id. Use this BEFORE asking any discovery questions — every subsequent phase tool needs the session_id.",
    inputSchema: {
      type: "object",
      properties: {
        user_intent: { type: "string", description: "The user's initial brief, verbatim" },
      },
      required: ["user_intent"],
    },
  },
  {
    name: "voce_generate_answer",
    description:
      "Record one (question, answer) discovery turn. Pass ready: true when you have enough context to propose an IR. Server does not auto-flip ready — that's your call.",
    inputSchema: {
      type: "object",
      properties: {
        session_id: { type: "string", description: "Session id from voce_generate_start" },
        question: { type: "string", description: "The question you asked the user" },
        answer: { type: "string", description: "The user's answer, verbatim" },
        ready: { type: "boolean", description: "True when discovery is complete" },
      },
      required: ["session_id", "question", "answer", "ready"],
    },
  },
  {
    name: "voce_generate_propose",
    description:
      "Submit your generated IR for review. BLOCKS if readiness < 70 — keep doing discovery first. Returns readiness, completeness pillar check, and records the IR as the session's current snapshot.",
    inputSchema: {
      type: "object",
      properties: {
        session_id: { type: "string", description: "Session id" },
        ir_json: { type: "string", description: "The IR JSON you generated" },
      },
      required: ["session_id", "ir_json"],
    },
  },
  {
    name: "voce_generate_refine",
    description:
      "Apply user feedback to a proposed IR. Records the feedback turn and updates the snapshot. Run completeness checks after — the new IR has to clear the same gates as the original proposal.",
    inputSchema: {
      type: "object",
      properties: {
        session_id: { type: "string", description: "Session id" },
        feedback: { type: "string", description: "The user's feedback" },
        ir_json: { type: "string", description: "Updated IR JSON" },
      },
      required: ["session_id", "feedback", "ir_json"],
    },
  },
  {
    name: "voce_generate_finalize",
    description:
      "Validate, compile, and seal the session. BLOCKS on missing pillars or any validation error — full-stack-completeness gate. Returns html, validation summary, and deployment hints.",
    inputSchema: {
      type: "object",
      properties: { session_id: { type: "string", description: "Session id" } },
      required: ["session_id"],
    },
  },
  // ── Quality gates ──────────────────────────────────────────────
  {
    name: "voce_generation_readiness",
    description:
      "Score how ready a session is to propose IR (0–100). Same gate voce_generate_propose enforces — call this proactively to know whether more discovery is needed.",
    inputSchema: {
      type: "object",
      properties: {
        session_id: { type: "string", description: "Session id from voce_generate_start" },
      },
      required: ["session_id"],
    },
  },
  {
    name: "voce_feature_completeness",
    description:
      "Check an IR for missing pillars (a11y, validation, error/loading/empty states, SEO). Same gate voce_generate_finalize enforces — run between propose and finalize to surface gaps early.",
    inputSchema: {
      type: "object",
      properties: { ir_json: { type: "string", description: "Voce IR JSON to inspect" } },
      required: ["ir_json"],
    },
  },
];
