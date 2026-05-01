#!/usr/bin/env node
// Build cached fixtures for the voce-ir.xyz live hero (S61).
// Reads packages/site-hero/starter-prompts.json, runs each available prompt's
// IR through `voce validate` and `voce compile`, and writes one fixture JSON
// per prompt plus an index.json. Skipped prompts (available: false) are
// recorded in the index so the UI can render a "coming soon" state.
//
// Run from the repo root:   node packages/site-hero/build-fixtures.mjs

import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..", "..");
const FIXTURES_DIR = join(HERE, "fixtures");
const MANIFEST_PATH = join(HERE, "starter-prompts.json");

// Prefer the locally-built voce CLI over a PATH-installed one — a stale
// Homebrew-installed `voce` will silently emit pre-rebuild output and quietly
// poison fixture regeneration after compiler changes (F-021).
const LOCAL_VOCE = join(REPO_ROOT, "target/release/voce");
const VOCE = existsSync(LOCAL_VOCE) ? LOCAL_VOCE : "voce";

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

  // Fixture content is deterministic — no timestamps or wall-clock timings,
  // so CI can detect drift via `git diff --quiet`.
  const validateOut = execFileSync(
    VOCE,
    ["validate", "--format", "json", irPath],
    { encoding: "utf8" }
  );
  const validation = JSON.parse(validateOut);
  delete validation.file; // absolute path leaks the build host

  const htmlPath = join(tmp, `${prompt.id}.html`);
  // --no-cache: the CLI compilation cache is keyed on IR content, not compiler
  // version, so without this a fixture rebuild after a compiler change can
  // silently write stale HTML. Round-trip verification then fails (F-021).
  execFileSync(VOCE, ["compile", irPath, "-o", htmlPath, "--no-cache"], {
    encoding: "utf8",
  });
  const html = readFileSync(htmlPath, "utf8");

  const fixture = {
    id: prompt.id,
    label: prompt.label,
    prompt: intentText,
    ir,
    validation,
    html,
    sizeBytes: Buffer.byteLength(html, "utf8"),
  };

  const fixturePath = join(FIXTURES_DIR, `${prompt.id}.json`);
  writeFileSync(fixturePath, JSON.stringify(fixture, null, 2) + "\n");

  indexEntries.push({
    id: prompt.id,
    label: prompt.label,
    available: true,
    sizeBytes: fixture.sizeBytes,
    validation: { valid: validation.valid, errors: validation.errors, warnings: validation.warnings },
  });

  availableCount += 1;
  console.log(`  ✓ ${prompt.id.padEnd(16)}  ${fixture.sizeBytes}B`);
}

const indexPath = join(FIXTURES_DIR, "index.json");
writeFileSync(
  indexPath,
  JSON.stringify({ prompts: indexEntries }, null, 2) + "\n"
);

rmSync(tmp, { recursive: true, force: true });

console.log(
  `\n${availableCount} fixture(s) written, ${skippedCount} pending. Index: ${indexPath}`
);
