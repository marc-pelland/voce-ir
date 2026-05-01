/**
 * Voce site-hero — live pipeline visualization.
 *
 * On prompt click: load cached fixture → run WASM validate + compile_dom in
 * the browser → animate the reveal. The pipeline runs fast (~few ms total);
 * the cinematic pacing is intentional. See build journal F-008.
 */

import indexJson from "../fixtures/index.json";
import type {
  Fixture,
  FixtureIndex,
  FixtureIndexEntry,
  ValidationDiagnostic,
  VoceWasm,
  WasmCompileResult,
  WasmValidateResult,
} from "./types.js";
import { VALIDATION_PASSES } from "./types.js";

const FIXTURE_INDEX = indexJson as FixtureIndex;

// Animation timing constants (ms). Honors prefers-reduced-motion: when the
// user has asked for reduced motion, all stages collapse to instant (10ms).
const REDUCED_MOTION =
  typeof window !== "undefined" &&
  window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

const TIMING = REDUCED_MOTION
  ? { irFadeIn: 10, prePass: 10, perPass: 10, postPass: 10, renderFadeIn: 10 }
  : { irFadeIn: 800, prePass: 200, perPass: 900, postPass: 300, renderFadeIn: 600 };

// DOM refs
const $ = <T extends HTMLElement = HTMLElement>(id: string): T =>
  document.getElementById(id) as T;
const status = $("status");
const buttonsEl = $("prompt-buttons");
const promptLabel = $("prompt-label");
const colIr = $("col-ir");
const colValidate = $("col-validate");
const colRender = $("col-render");
const irOut = $<HTMLPreElement>("ir-out");
const irMeta = $("ir-meta");
const passList = $<HTMLUListElement>("pass-list");
const validateMeta = $("validate-meta");
const verdictEl = $("verdict");
const renderOut = $<HTMLIFrameElement>("render-out");
const renderMeta = $("render-meta");
const skipBtn = $<HTMLButtonElement>("skip-btn");

let wasm: VoceWasm | null = null;
let activeRun: AbortController | null = null;

// ── WASM loading ──────────────────────────────────────────────────────────────

async function loadWasm(): Promise<VoceWasm> {
  const mod = await import("../wasm/voce_playground_wasm.js");
  await mod.default();
  return mod as unknown as VoceWasm;
}

async function loadFixture(id: string): Promise<Fixture> {
  const mod = await import(`../fixtures/${id}.json`);
  return mod.default as Fixture;
}

// ── UI building ───────────────────────────────────────────────────────────────

function setStatus(text: string, kind: "ok" | "err" | "" = ""): void {
  status.textContent = text;
  status.className = kind;
}

function renderPromptButtons(index: FixtureIndex): void {
  buttonsEl.innerHTML = "";
  for (const entry of index.prompts) {
    const btn = document.createElement("button");
    btn.className = "prompt-btn";
    btn.textContent = entry.label;
    btn.dataset.id = entry.id;
    btn.disabled = !entry.available;
    if (!entry.available && entry.pending) btn.title = entry.pending;
    btn.addEventListener("click", () => onPromptClick(entry));
    buttonsEl.appendChild(btn);
  }
}

function setActiveButton(id: string): void {
  for (const b of buttonsEl.querySelectorAll<HTMLButtonElement>(".prompt-btn")) {
    b.classList.toggle("active", b.dataset.id === id);
  }
}

function resetColumns(): void {
  for (const col of [colIr, colValidate, colRender]) col.classList.remove("idle");
  irOut.classList.remove("visible");
  irOut.textContent = "";
  passList.innerHTML = "";
  verdictEl.classList.remove("visible", "ok", "err");
  verdictEl.textContent = "";
  renderOut.classList.remove("visible");
  renderOut.srcdoc = "";
  irMeta.textContent = "cached fixture";
  validateMeta.textContent = "live · in browser";
  renderMeta.textContent = "live · in browser";
}

// ── Diagnostic grouping ───────────────────────────────────────────────────────

interface PassStatus {
  pass: string;
  errors: ValidationDiagnostic[];
  warnings: ValidationDiagnostic[];
  result: "ok" | "warn" | "err";
}

function groupByPass(v: WasmValidateResult): PassStatus[] {
  const map = new Map<string, PassStatus>();
  for (const p of VALIDATION_PASSES) {
    map.set(p, { pass: p, errors: [], warnings: [], result: "ok" });
  }
  for (const e of v.errors) {
    const slot = map.get(e.pass) ?? { pass: e.pass, errors: [], warnings: [], result: "ok" };
    slot.errors.push(e);
    slot.result = "err";
    map.set(e.pass, slot);
  }
  for (const w of v.warnings) {
    const slot = map.get(w.pass) ?? { pass: w.pass, errors: [], warnings: [], result: "ok" };
    slot.warnings.push(w);
    if (slot.result !== "err") slot.result = "warn";
    map.set(w.pass, slot);
  }
  // Preserve canonical order
  return VALIDATION_PASSES.map((p) => map.get(p)!);
}

// ── Animation primitives ──────────────────────────────────────────────────────

function delay(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal.aborted) return reject(new DOMException("aborted", "AbortError"));
    const t = setTimeout(resolve, ms);
    signal.addEventListener(
      "abort",
      () => {
        clearTimeout(t);
        reject(new DOMException("aborted", "AbortError"));
      },
      { once: true }
    );
  });
}

function describeStatus(s: PassStatus): string {
  if (s.result === "err") {
    const n = s.errors.length;
    return `${n} error${n === 1 ? "" : "s"}`;
  }
  if (s.result === "warn") {
    const n = s.warnings.length;
    return `${n} warning${n === 1 ? "" : "s"}`;
  }
  return "ok";
}

// ── Main flow ─────────────────────────────────────────────────────────────────

async function onPromptClick(entry: FixtureIndexEntry): Promise<void> {
  if (!entry.available || !wasm) return;

  // Cancel any in-flight animation
  activeRun?.abort();
  const run = new AbortController();
  activeRun = run;

  setActiveButton(entry.id);
  resetColumns();

  const fixture = await loadFixture(entry.id);

  promptLabel.innerHTML = `<span class="quote">“${escapeHtml(fixture.prompt)}”</span>`;

  // Run the real pipeline ONCE up front. The animation paces the *display*,
  // not the execution.
  const irJsonString = JSON.stringify(fixture.ir);

  const validateStart = performance.now();
  const validation = JSON.parse(wasm.validate(irJsonString)) as WasmValidateResult;
  const validateMs = performance.now() - validateStart;

  const compileStart = performance.now();
  const compile = JSON.parse(wasm.compile_dom(irJsonString)) as WasmCompileResult;
  const compileMs = performance.now() - compileStart;

  // Pre-populate static content so it's ready for fade-in
  irOut.textContent = JSON.stringify(fixture.ir, null, 2);
  irMeta.textContent = `${fixture.sizeBytes} B output · ${countLines(irOut.textContent)} lines`;

  const passes = groupByPass(validation);
  passList.innerHTML = "";
  for (const p of passes) {
    const li = document.createElement("li");
    li.dataset.pass = p.pass;
    li.dataset.final = p.result;
    li.dataset.detail = p.result === "ok" ? "" : describeStatus(p);
    li.innerHTML = `<span class="pass-name">${p.pass}</span><span class="pass-detail" hidden></span>`;
    passList.appendChild(li);
  }

  if (compile.ok) {
    renderOut.srcdoc = compile.html;
  } else {
    renderOut.srcdoc = `<pre style="font:13px monospace;color:#b91c1c;padding:16px">compile error: ${escapeHtml(compile.error ?? "unknown")}</pre>`;
  }
  renderMeta.textContent = compile.ok
    ? `${compile.sizeBytes} B · ${compileMs.toFixed(1)} ms`
    : `compile failed`;
  validateMeta.textContent = `9 passes · ${validateMs.toFixed(1)} ms`;

  // Pre-populate verdict text so skip can show it without waiting for stage D
  const errCount = validation.errors.length;
  const warnCount = validation.warnings.length;
  if (validation.valid) {
    verdictEl.textContent =
      warnCount > 0
        ? `✓ valid · ${warnCount} warning${warnCount === 1 ? "" : "s"}`
        : `✓ valid · all 9 passes clean`;
    verdictEl.classList.add("ok");
  } else {
    verdictEl.textContent = `✗ ${errCount} error${errCount === 1 ? "" : "s"} · compilation blocked`;
    verdictEl.classList.add("err");
  }

  skipBtn.hidden = false;

  try {
    await runAnimation(passes, run.signal);
  } catch (e) {
    if ((e as Error).name !== "AbortError") throw e;
  } finally {
    if (activeRun === run) {
      activeRun = null;
      skipBtn.hidden = true;
    }
  }
}

async function runAnimation(passes: PassStatus[], signal: AbortSignal): Promise<void> {
  // Stage A — IR fades in
  irOut.classList.add("visible");
  await delay(TIMING.irFadeIn, signal);

  // Stage B — pre-pass pause
  await delay(TIMING.prePass, signal);

  // Stage C — pass-by-pass reveal
  for (const status of passes) {
    const li = passList.querySelector<HTMLLIElement>(`[data-pass="${status.pass}"]`)!;
    li.classList.add("running");
    await delay(TIMING.perPass, signal);
    li.classList.remove("running");
    li.classList.add(status.result);
    const detail = li.querySelector<HTMLSpanElement>(".pass-detail")!;
    if (status.result !== "ok") {
      detail.textContent = describeStatus(status);
      detail.hidden = false;
    }
  }

  // Stage D — verdict reveal (text + result class were set at run start)
  await delay(TIMING.postPass, signal);
  verdictEl.classList.add("visible");

  // Stage E — render
  await delay(TIMING.postPass, signal);
  renderOut.classList.add("visible");
  await delay(TIMING.renderFadeIn, signal);
}

function skipAnimation(): void {
  if (!activeRun) return;
  activeRun.abort();

  irOut.classList.add("visible");
  for (const li of passList.querySelectorAll<HTMLLIElement>("li")) {
    li.classList.remove("running");
    const final = (li.dataset.final ?? "ok") as "ok" | "warn" | "err";
    if (!li.classList.contains(final)) li.classList.add(final);
    const detail = li.querySelector<HTMLSpanElement>(".pass-detail")!;
    const detailText = li.dataset.detail ?? "";
    if (detailText) {
      detail.textContent = detailText;
      detail.hidden = false;
    }
  }
  verdictEl.classList.add("visible");
  renderOut.classList.add("visible");
  skipBtn.hidden = true;
  activeRun = null;
}

// ── Utilities ─────────────────────────────────────────────────────────────────

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function countLines(s: string): number {
  let n = 1;
  for (let i = 0; i < s.length; i += 1) if (s.charCodeAt(i) === 10) n += 1;
  return n;
}

// ── Init ──────────────────────────────────────────────────────────────────────

async function init(): Promise<void> {
  setStatus("Loading WASM…");
  try {
    wasm = await loadWasm();
  } catch (e) {
    setStatus(`WASM load failed: ${e}`, "err");
    return;
  }
  const ready = FIXTURE_INDEX.prompts.filter((p) => p.available).length;
  const total = FIXTURE_INDEX.prompts.length;
  setStatus(
    `WASM ready · ${ready} of ${total} prompts cached · pick one to run the pipeline`,
    "ok"
  );
  renderPromptButtons(FIXTURE_INDEX);
  skipBtn.addEventListener("click", skipAnimation);

  // Click anywhere on the columns area to skip
  $("columns").addEventListener("click", (e) => {
    if (activeRun && !(e.target as HTMLElement).closest("iframe")) {
      skipAnimation();
    }
  });
}

init();
