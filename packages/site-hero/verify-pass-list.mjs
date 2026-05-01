#!/usr/bin/env node
// Assert that the front-end VALIDATION_PASSES list matches the Rust engine's
// canonical pass list (order + names). Catches drift between
// packages/validator/src/passes/* and packages/site-hero/src/types.ts.
//
// Run from packages/site-hero:   node verify-pass-list.mjs

import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..", "..");
const PASSES_DIR = join(REPO_ROOT, "packages/validator/src/passes");
const TYPES_PATH = join(HERE, "src/types.ts");

// Engine: parse mod.rs::all_passes() to extract the module-name execution order
const modSource = readFileSync(join(PASSES_DIR, "mod.rs"), "utf8");
const orderMatches = [...modSource.matchAll(/Box::new\((\w+)::/g)];
if (orderMatches.length === 0) {
  console.error("Could not parse all_passes() — no Box::new(...::...) entries found");
  process.exit(1);
}
const moduleOrder = orderMatches.map((m) => m[1]);

// For each module, parse `fn name(...) -> &'static str { "..." }` to get the canonical name
const engineList = moduleOrder.map((mod) => {
  const path = join(PASSES_DIR, `${mod}.rs`);
  const src = readFileSync(path, "utf8");
  const m = src.match(/fn name\([^)]*\)\s*->\s*&'static str\s*\{\s*"([^"]+)"\s*\}/);
  if (!m) {
    console.error(`Could not extract pass name from ${path}`);
    process.exit(1);
  }
  return m[1];
});

// Front-end: parse types.ts::VALIDATION_PASSES
const typesSource = readFileSync(TYPES_PATH, "utf8");
const tsBlock = typesSource.match(/VALIDATION_PASSES\s*=\s*\[([^\]]+)\]/);
if (!tsBlock) {
  console.error("Could not find VALIDATION_PASSES = [...] in types.ts");
  process.exit(1);
}
const tsList = [...tsBlock[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);

// Compare
const eq =
  engineList.length === tsList.length &&
  engineList.every((p, i) => p === tsList[i]);

console.log("engine    :", engineList.join(", "));
console.log("front-end :", tsList.join(", "));
console.log();

if (eq) {
  console.log(`✓ ${engineList.length} passes, names and order match.`);
  process.exit(0);
}

console.error("✗ DRIFT — engine and front-end pass lists disagree.");
const max = Math.max(engineList.length, tsList.length);
for (let i = 0; i < max; i += 1) {
  const e = engineList[i] ?? "—";
  const t = tsList[i] ?? "—";
  if (e !== t) console.error(`  [${i}]  engine=${e}  front-end=${t}`);
}
process.exit(1);
