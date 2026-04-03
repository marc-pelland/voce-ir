/**
 * Voce IR Playground — browser-based IR editor with live compilation.
 *
 * Loads the WASM module lazily, then recompiles on every input change.
 */

import { EXAMPLES } from "./examples.js";

// WASM module interface
interface VoceWasm {
  validate(irJson: string): string;
  compile_dom(irJson: string): string;
  compile_email(irJson: string): string;
  inspect(irJson: string): string;
}

let wasm: VoceWasm | null = null;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// DOM references
const irEditor = document.getElementById("ir-editor") as HTMLTextAreaElement;
const promptEditor = document.getElementById("prompt-editor") as HTMLTextAreaElement;
const previewFrame = document.getElementById("preview-frame") as HTMLIFrameElement;
const validationResults = document.getElementById("validation-results")!;
const htmlOutput = document.getElementById("html-output")!.querySelector("code")!;
const inspectorTree = document.getElementById("inspector-tree")!;
const sizeBadge = document.getElementById("size-badge")!;
const targetSelect = document.getElementById("target-select") as HTMLSelectElement;
const shareBtn = document.getElementById("share-btn") as HTMLButtonElement;
const formatBtn = document.getElementById("format-btn") as HTMLButtonElement;

// --- WASM Loading ---

async function loadWasm(): Promise<VoceWasm> {
  // The WASM pkg is copied to public/ during build
  const mod = await import("../wasm/voce_playground_wasm.js");
  await mod.default();
  return mod as unknown as VoceWasm;
}

async function init() {
  validationResults.textContent = "Loading WASM module...";

  try {
    wasm = await loadWasm();
    validationResults.textContent = "Ready. Paste IR JSON or select an example.";
  } catch (e) {
    validationResults.innerHTML = `<div class="diag diag-error">
      <span class="diag-code">WASM</span> Failed to load: ${e}
    </div>`;
    return;
  }

  // Load from URL hash if present
  const hash = window.location.hash.slice(1);
  if (hash) {
    try {
      const decoded = atob(hash);
      irEditor.value = decoded;
      recompile();
    } catch {
      // Invalid hash, ignore
    }
  }
}

// --- Compilation ---

function recompile() {
  if (!wasm) return;

  const ir = irEditor.value.trim();
  if (!ir) {
    validationResults.textContent = "Paste IR JSON or select an example.";
    htmlOutput.textContent = "";
    previewFrame.srcdoc = "";
    sizeBadge.textContent = "";
    inspectorTree.textContent = "";
    return;
  }

  // Validate
  const valRaw = wasm.validate(ir);
  const val = JSON.parse(valRaw);
  renderValidation(val);

  // Inspect
  const inspRaw = wasm.inspect(ir);
  const insp = JSON.parse(inspRaw);
  renderInspector(insp);

  // Compile
  const target = targetSelect.value;
  const compileRaw = target === "email"
    ? wasm.compile_email(ir)
    : wasm.compile_dom(ir);
  const compile = JSON.parse(compileRaw);

  if (compile.ok) {
    htmlOutput.textContent = compile.html;
    previewFrame.srcdoc = compile.html;
    sizeBadge.textContent = `${(compile.sizeBytes / 1024).toFixed(1)} KB`;
  } else {
    htmlOutput.textContent = compile.error || "Compilation failed";
    previewFrame.srcdoc = "";
    sizeBadge.textContent = "";
  }
}

function renderValidation(val: {
  valid: boolean;
  errorCount: number;
  warningCount: number;
  errors: Array<{ code: string; message: string; path: string; hint?: string }>;
  warnings: Array<{ code: string; message: string; path: string; hint?: string }>;
}) {
  if (val.valid && val.warningCount === 0) {
    validationResults.innerHTML = `<div class="valid-badge">&#10003; Valid — 0 errors, 0 warnings</div>`;
    return;
  }

  let html = "";

  if (val.valid) {
    html += `<div class="valid-badge">&#10003; Valid</div>`;
  }

  for (const err of val.errors) {
    html += `<div class="diag diag-error">
      <span class="diag-code">${esc(err.code)}</span> ${esc(err.message)}
      ${err.path ? `<span class="diag-path">${esc(err.path)}</span>` : ""}
      ${err.hint ? `<span class="diag-hint">${esc(err.hint)}</span>` : ""}
    </div>`;
  }

  for (const warn of val.warnings) {
    html += `<div class="diag diag-warning">
      <span class="diag-code">${esc(warn.code)}</span> ${esc(warn.message)}
      ${warn.path ? `<span class="diag-path">${esc(warn.path)}</span>` : ""}
      ${warn.hint ? `<span class="diag-hint">${esc(warn.hint)}</span>` : ""}
    </div>`;
  }

  validationResults.innerHTML = html;
}

function renderInspector(insp: {
  ok: boolean;
  nodeCount?: number;
  maxDepth?: number;
  nodeCounts?: Record<string, number>;
  schemaVersion?: string;
  error?: string;
}) {
  if (!insp.ok) {
    inspectorTree.textContent = insp.error || "Parse error";
    return;
  }

  let html = `<div><strong>Schema:</strong> v${esc(insp.schemaVersion || "?")}</div>`;
  html += `<div><strong>Nodes:</strong> ${insp.nodeCount} (depth ${insp.maxDepth})</div>`;
  html += `<div style="margin-top:8px"><strong>Node types:</strong></div>`;

  if (insp.nodeCounts) {
    for (const [type, count] of Object.entries(insp.nodeCounts)) {
      html += `<div class="tree-node">
        <span class="tree-type">${esc(type)}</span>
        <span class="tree-count">&times;${count}</span>
      </div>`;
    }
  }

  inspectorTree.innerHTML = html;
}

// --- Tabs ---

document.querySelectorAll(".panel-tabs").forEach((tabBar) => {
  tabBar.querySelectorAll<HTMLButtonElement>(".tab").forEach((tab) => {
    tab.addEventListener("click", () => {
      const panel = tabBar.parentElement!;
      panel.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
      panel.querySelectorAll(".tab-content").forEach((c) => c.classList.remove("active"));
      tab.classList.add("active");
      const target = tab.dataset.tab;
      const content = panel.querySelector(`#tab-${target}`);
      content?.classList.add("active");
    });
  });
});

// --- Events ---

irEditor.addEventListener("input", () => {
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(recompile, 300);
});

targetSelect.addEventListener("change", recompile);

shareBtn.addEventListener("click", () => {
  const ir = irEditor.value.trim();
  if (!ir) return;
  const encoded = btoa(ir);
  const url = `${window.location.origin}${window.location.pathname}#${encoded}`;
  navigator.clipboard.writeText(url).then(() => {
    shareBtn.textContent = "Copied!";
    setTimeout(() => { shareBtn.textContent = "Share"; }, 1500);
  });
});

formatBtn.addEventListener("click", () => {
  try {
    const parsed = JSON.parse(irEditor.value);
    irEditor.value = JSON.stringify(parsed, null, 2);
  } catch {
    // Not valid JSON, ignore
  }
});

// --- Examples ---

document.querySelectorAll<HTMLButtonElement>(".example-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    const name = btn.dataset.example;
    if (name && EXAMPLES[name]) {
      irEditor.value = JSON.stringify(EXAMPLES[name], null, 2);
      recompile();
    }
  });
});

// --- Helpers ---

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// --- Boot ---

init();
