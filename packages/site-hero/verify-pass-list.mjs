#!/usr/bin/env node
// Assert that the front-end VALIDATION_PASSES list matches the Rust engine's
// canonical pass list (order + names). Catches drift between
// packages/validator/src/passes/* and packages/site-hero/src/types.ts.
//
// Uses `voce validate --list-passes` (S67 Day 1) as the authoritative source
// instead of parsing Rust source files — robust to refactors, formatting, or
// any future changes that don't alter the actual pass enumeration.
//
// Run from packages/site-hero:   node verify-pass-list.mjs

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..", "..");
const TYPES_PATH = join(HERE, "src/types.ts");

const LOCAL_VOCE = join(REPO_ROOT, "target/release/voce");
const VOCE = existsSync(LOCAL_VOCE) ? LOCAL_VOCE : "voce";

// Engine: ask the CLI directly. Falls back to source-parsing if the CLI is
// older than S67 Day 1 (doesn't recognize --list-passes).
let engineList;
try {
  const out = execFileSync(VOCE, ["validate", "--list-passes"], { encoding: "utf8" });
  engineList = JSON.parse(out).passes;
  if (!Array.isArray(engineList)) throw new Error("expected { passes: [...] }");
} catch (err) {
  // Detect "unknown argument" failures so we can fall back gracefully on older binaries
  const probe = spawnSync(VOCE, ["validate", "--list-passes"], { encoding: "utf8" });
  const looksLikeUnknownArg = (probe.stderr ?? "").includes("--list-passes");
  if (!looksLikeUnknownArg) {
    console.error(`voce --list-passes failed: ${err.message}`);
    process.exit(1);
  }
  console.warn(
    "warning: voce CLI predates --list-passes flag (S67 Day 1). Falling back to source parsing."
  );
  const PASSES_DIR = join(REPO_ROOT, "packages/validator/src/passes");
  const modSource = readFileSync(join(PASSES_DIR, "mod.rs"), "utf8");
  const orderMatches = [...modSource.matchAll(/Box::new\((\w+)::/g)];
  engineList = orderMatches
    .map((m) => m[1])
    .map((mod) => {
      const src = readFileSync(join(PASSES_DIR, `${mod}.rs`), "utf8");
      const nameMatch = src.match(
        /fn name\([^)]*\)\s*->\s*&'static str\s*\{\s*"([^"]+)"\s*\}/
      );
      return nameMatch?.[1] ?? mod;
    });
}

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
