#!/usr/bin/env node
// Cold-start benchmark for voce-chat. The S66 acceptance criteria pin a
// 500ms ceiling on developer-laptop hardware. This script spawns the dist
// binary, kills it as soon as the banner reaches stdout, and prints the
// elapsed wall-clock time. Exits non-zero if the threshold is exceeded so
// the script can gate releases.
//
// Usage:
//   node scripts/bench-startup.mjs           # measure once
//   node scripts/bench-startup.mjs --runs 5  # average over N runs
//   node scripts/bench-startup.mjs --max-ms 500  # custom threshold
//
// The startup path tested:
//   import every dep → resolve session → build system prompt → print banner.
// ANTHROPIC_API_KEY is unset so the model isn't initialized.

import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const BIN = resolve(HERE, "..", "dist", "index.js");

const args = process.argv.slice(2);
const runs = pickFlag(args, "--runs", 3);
const maxMs = pickFlag(args, "--max-ms", 500);

function pickFlag(argv, name, fallback) {
  const i = argv.indexOf(name);
  if (i === -1) return fallback;
  const v = parseInt(argv[i + 1], 10);
  return Number.isFinite(v) ? v : fallback;
}

async function measureOnce() {
  // Each run gets a fresh project root so the session-resolve path is
  // measured cold (no pre-existing sessions to scan).
  const sandbox = mkdtempSync(`${tmpdir()}/voce-chat-bench-`);
  return new Promise((resolveOuter, reject) => {
    const start = process.hrtime.bigint();
    const child = spawn(process.execPath, [BIN], {
      env: {
        ...process.env,
        ANTHROPIC_API_KEY: "",
        VOCE_PROJECT_ROOT: sandbox,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });

    let banner = "";
    let resolved = false;
    const cleanup = () => {
      if (resolved) return;
      resolved = true;
      child.kill("SIGTERM");
      rmSync(sandbox, { recursive: true, force: true });
    };

    child.stdout.on("data", (buf) => {
      banner += buf.toString();
      // The banner lands as soon as main() reaches `console.log` after the
      // session is resolved. Once we see "Voce IR" we know startup is done.
      if (banner.includes("Voce IR")) {
        const end = process.hrtime.bigint();
        const ms = Number(end - start) / 1e6;
        cleanup();
        resolveOuter(ms);
      }
    });

    child.on("error", (err) => {
      cleanup();
      reject(err);
    });

    setTimeout(() => {
      if (!resolved) {
        cleanup();
        reject(new Error("startup did not complete within 5000ms"));
      }
    }, 5000);
  });
}

const samples = [];
for (let i = 0; i < runs; i++) {
  samples.push(await measureOnce());
}
const avg = samples.reduce((a, b) => a + b, 0) / samples.length;
const min = Math.min(...samples);
const max = Math.max(...samples);

console.log(`voce-chat cold start (${runs} runs):`);
for (let i = 0; i < samples.length; i++) {
  console.log(`  run ${i + 1}: ${samples[i].toFixed(0)}ms`);
}
console.log(`  min ${min.toFixed(0)}ms · avg ${avg.toFixed(0)}ms · max ${max.toFixed(0)}ms`);
console.log(`  threshold: ${maxMs}ms`);

if (avg > maxMs) {
  console.error(`\nFAIL: average ${avg.toFixed(0)}ms exceeds ${maxMs}ms threshold`);
  process.exit(1);
}
console.log(`\nOK: under ${maxMs}ms.`);
