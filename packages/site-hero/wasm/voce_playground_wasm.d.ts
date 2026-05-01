/* tslint:disable */
/* eslint-disable */

/**
 * Compile IR JSON to HTML (DOM target). Returns a JSON string with the result.
 *
 * Result shape: `{ "ok": bool, "html": string, "sizeBytes": number, "error"?: string }`
 */
export function compile_dom(ir_json: string): string;

/**
 * Compile IR JSON to email HTML. Returns a JSON string with the result.
 */
export function compile_email(ir_json: string): string;

/**
 * Inspect IR JSON — returns a summary.
 */
export function inspect(ir_json: string): string;

/**
 * Validate IR JSON. Returns a JSON string with validation results.
 *
 * Result shape: `{ "valid": bool, "errors": [...], "warnings": [...] }`
 */
export function validate(ir_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly compile_dom: (a: number, b: number) => [number, number];
    readonly compile_email: (a: number, b: number) => [number, number];
    readonly inspect: (a: number, b: number) => [number, number];
    readonly validate: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
