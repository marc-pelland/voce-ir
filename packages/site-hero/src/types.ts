/** Shape of an entry in fixtures/index.json. Kept deterministic — no
 * timestamps or run-time-dependent values, so CI can diff for drift. */
export interface FixtureIndexEntry {
  id: string;
  label: string;
  available: boolean;
  pending?: string | null;
  sizeBytes?: number;
  validation?: { valid: boolean; errors: number; warnings: number };
}

export interface FixtureIndex {
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
 * a JSON-encoded envelope string — see WasmCompileResult / WasmValidateResult.
 * `validate_verbose` (S67) is preferred where per-pass telemetry is needed. */
export interface VoceWasm {
  validate(irJson: string): string;
  validate_verbose(irJson: string): string;
  compile_dom(irJson: string): string;
  inspect(irJson: string): string;
}

/** Per-pass telemetry from `validate_verbose`. Real numbers, not synthesized. */
export interface PassTelemetry {
  name: string;
  durationUs: number;
  errors: number;
  warnings: number;
  codes: string[];
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

/** Verbose envelope from `validate_verbose` — superset of WasmValidateResult. */
export interface WasmValidateVerboseResult extends WasmValidateResult {
  passes: PassTelemetry[];
}

/** Canonical list of validation passes, mirroring `passes/mod.rs::all_passes()`
 * order and `ValidationPass::name()` strings. Drift between the two is caught
 * by `verify-pass-list.mjs` in CI. */
export const VALIDATION_PASSES = [
  "structural",
  "references",
  "state-machine",
  "accessibility",
  "security",
  "seo",
  "forms",
  "i18n",
  "motion",
] as const;
