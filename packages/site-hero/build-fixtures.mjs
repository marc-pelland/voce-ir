#!/usr/bin/env node
// Build cached fixtures for the voce-ir.xyz live hero (S61).
// Reads packages/site-hero/starter-prompts.json, runs each available prompt's
// IR through `voce validate` and `voce compile`, and writes one fixture JSON
// per prompt plus an index.json. Skipped prompts (available: false) are
// recorded in the index so the UI can render a "coming soon" state.
//
// Run from the repo root:   node packages/site-hero/build-fixtures.mjs

import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..", "..");
const FIXTURES_DIR = join(HERE, "fixtures");
const MANIFEST_PATH = join(HERE, "starter-prompts.json");

const manifest = JSON.parse(readFileSync(MANIFEST_PATH, "utf8"));
const tmp = mkdtempSync(join(tmpdir(), "voce-hero-"));

const indexEntries = [];
let availableCount = 0;
let skippedCount = 0;

for (const prompt of manifest.prompts) {
  if (!prompt.available) {
    indexEntries.push({
      id: prompt.id,
      label: prompt.label,
      available: false,
      pending: prompt.pending ?? null,
    });
    skippedCount += 1;
    continue;
  }

  const intentDir = resolve(REPO_ROOT, prompt.intentDir);
  const irPath = join(intentDir, "ir.voce.json");
  const intentPath = join(intentDir, "intent.md");

  const ir = JSON.parse(readFileSync(irPath, "utf8"));
  const intentText = readFileSync(intentPath, "utf8").trim();

  const validateStart = Date.now();
  const validateOut = execFileSync(
    "voce",
    ["validate", "--format", "json", irPath],
    { encoding: "utf8" }
  );
  const validateMs = Date.now() - validateStart;
  const validation = JSON.parse(validateOut);
  delete validation.file; // absolute path leaks the build host

  const htmlPath = join(tmp, `${prompt.id}.html`);
  const compileStart = Date.now();
  execFileSync("voce", ["compile", irPath, "-o", htmlPath], {
    encoding: "utf8",
  });
  const compileMs = Date.now() - compileStart;
  const html = readFileSync(htmlPath, "utf8");

  const fixture = {
    id: prompt.id,
    label: prompt.label,
    prompt: intentText,
    ir,
    validation,
    html,
    sizeBytes: Buffer.byteLength(html, "utf8"),
    timings: { validateMs, compileMs },
    generatedAt: new Date().toISOString(),
  };

  const fixturePath = join(FIXTURES_DIR, `${prompt.id}.json`);
  writeFileSync(fixturePath, JSON.stringify(fixture, null, 2) + "\n");

  indexEntries.push({
    id: prompt.id,
    label: prompt.label,
    available: true,
    sizeBytes: fixture.sizeBytes,
    validation: { valid: validation.valid, errors: validation.errors, warnings: validation.warnings },
    timings: fixture.timings,
  });

  availableCount += 1;
  console.log(
    `  ✓ ${prompt.id.padEnd(16)}  validate ${validateMs}ms  compile ${compileMs}ms  ${fixture.sizeBytes}B`
  );
}

const indexPath = join(FIXTURES_DIR, "index.json");
writeFileSync(
  indexPath,
  JSON.stringify(
    { generatedAt: new Date().toISOString(), prompts: indexEntries },
    null,
    2
  ) + "\n"
);

rmSync(tmp, { recursive: true, force: true });

console.log(
  `\n${availableCount} fixture(s) written, ${skippedCount} pending. Index: ${indexPath}`
);
