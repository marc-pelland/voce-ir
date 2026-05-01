#!/usr/bin/env node
// Verify each cached fixture round-trips through playground-wasm.
// Loads the wasm artifact directly (no browser, no fetch) and calls
// validate / compile_dom against the cached IR, comparing the live
// output to the cached output byte-for-byte.
//
// Run from packages/site-hero:   node verify-roundtrip.mjs

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));

const wasmGlue = await import(
  "./wasm/voce_playground_wasm.js"
);
const wasmBytes = readFileSync(
  join(HERE, "wasm", "voce_playground_wasm_bg.wasm")
);

// initSync accepts {module} where module is a buffer or WebAssembly.Module.
wasmGlue.initSync({ module: wasmBytes });

const indexPath = join(HERE, "fixtures", "index.json");
const index = JSON.parse(readFileSync(indexPath, "utf8"));

let pass = 0;
let fail = 0;

for (const entry of index.prompts) {
  if (!entry.available) {
    console.log(`  · ${entry.id.padEnd(16)}  pending — skipped`);
    continue;
  }

  const fixturePath = join(HERE, "fixtures", `${entry.id}.json`);
  const fixture = JSON.parse(readFileSync(fixturePath, "utf8"));
  const irJson = JSON.stringify(fixture.ir);

  const validateRaw = wasmGlue.validate(irJson);
  const validateLive = JSON.parse(validateRaw);

  const compileRaw = wasmGlue.compile_dom(irJson);
  const compileLive = JSON.parse(compileRaw);

  const liveValid = validateLive.valid === true;
  const cachedValid = fixture.validation.valid === true;
  const validateAgrees = liveValid === cachedValid;

  const compileOk = compileLive.ok === true;
  const liveHtml = compileLive.html ?? "";
  const htmlMatches = liveHtml === fixture.html;

  const allOk = validateAgrees && compileOk && htmlMatches;
  if (allOk) pass += 1;
  else fail += 1;

  const status = allOk ? "✓" : "✗";
  console.log(
    `  ${status} ${entry.id.padEnd(16)}  ` +
      `validate(live=${liveValid} cached=${cachedValid})  ` +
      `compile.ok=${compileOk}  ` +
      `html.match=${htmlMatches}  ` +
      `(live=${liveHtml.length}B cached=${fixture.html.length}B)`
  );

  if (!htmlMatches && compileOk) {
    // Find first divergence point to make debugging easier
    let firstDiff = -1;
    const min = Math.min(liveHtml.length, fixture.html.length);
    for (let i = 0; i < min; i += 1) {
      if (liveHtml[i] !== fixture.html[i]) {
        firstDiff = i;
        break;
      }
    }
    if (firstDiff < 0) firstDiff = min;
    console.log(
      `      first divergence at offset ${firstDiff}:\n` +
        `        live:   …${JSON.stringify(liveHtml.slice(Math.max(0, firstDiff - 20), firstDiff + 30))}\n` +
        `        cached: …${JSON.stringify(fixture.html.slice(Math.max(0, firstDiff - 20), firstDiff + 30))}`
    );
  }
}

console.log(
  `\n${pass} fixture(s) round-tripped cleanly, ${fail} divergence(s).`
);
process.exit(fail > 0 ? 1 : 0);
