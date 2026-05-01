/** Shape of an entry in fixtures/index.json. */
export interface FixtureIndexEntry {
  id: string;
  label: string;
  available: boolean;
  pending?: string | null;
  sizeBytes?: number;
  validation?: { valid: boolean; errors: number; warnings: number };
  timings?: { validateMs: number; compileMs: number };
}

export interface FixtureIndex {
  generatedAt: string;
  prompts: FixtureIndexEntry[];
}

/** Shape of a per-prompt fixture file (e.g. fixtures/hero-section.json). */
export interface Fixture {
  id: string;
  label: string;
  prompt: string;
  ir: unknown;
  validation: ValidationReport;
  html: string;
  sizeBytes: number;
  timings: { validateMs: number; compileMs: number };
  generatedAt: string;
}

export interface ValidationDiagnostic {
  severity?: string;
  code: string;
  message: string;
  path: string;
  pass: string;
  hint?: string | null;
}

export interface ValidationReport {
  valid: boolean;
  errors: number;
  warnings: number;
  diagnostics: ValidationDiagnostic[];
}

/** Subset of the playground-wasm API used by site-hero. Each function returns
 * a JSON-encoded envelope string — see WasmCompileResult / WasmValidateResult. */
export interface VoceWasm {
  validate(irJson: string): string;
  compile_dom(irJson: string): string;
  inspect(irJson: string): string;
}

/** Envelope shape returned by playground-wasm's compile_dom / compile_email.
 * Note: the CLI writes raw HTML to disk; the WASM wraps it in this envelope.
 * See build journal finding #2 (S61 Day 2). */
export interface WasmCompileResult {
  ok: boolean;
  html: string;
  sizeBytes: number;
  error?: string;
}

/** Envelope shape returned by playground-wasm's validate.
 * Note: `errors` and `warnings` are diagnostic *arrays* here. The CLI's
 * --format json returns counts under those names with diagnostics in a
 * separate `diagnostics` field. See build journal finding #2 (S61 Day 2). */
export interface WasmValidateResult {
  valid: boolean;
  errors: ValidationDiagnostic[];
  warnings: ValidationDiagnostic[];
}

/** The 9 validation passes the validator runs. Synthesized for the visualization
 * because the current `voce validate --format json` output is summary-only —
 * see build journal finding #1 (S61 Day 1). */
export const VALIDATION_PASSES = [
  "structural-completeness",
  "reference-resolution",
  "state-machine",
  "accessibility",
  "motion-safety",
  "security",
  "seo",
  "forms",
  "i18n",
] as const;
