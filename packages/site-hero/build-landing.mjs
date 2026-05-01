#!/usr/bin/env node
// Day 4 — assemble the production homepage by splicing IR-compiled landing
// content (nav, pipeline, features, footer) around the WASM-driven live hero.
//
// Pipeline:
//   1. `voce compile examples/production/landing.voce.json`
//      The IR has a TextNode with content "__VOCE_HERO_SLOT__" sitting where
//      the static hero used to be. After compile it becomes <p>__VOCE_HERO_SLOT__</p>.
//   2. Extract the body innerHTML from the compiled output.
//   3. Split it into "above slot" (nav) and "below slot" (everything else).
//   4. Read packages/site-hero/dist/index.html (must run `vite build` first).
//   5. Replace <!-- VOCE_LANDING_TOP --> and <!-- VOCE_LANDING_BOTTOM --> with
//      the two halves.
//   6. Write the final combined HTML to packages/site-hero/dist-integrated/index.html.
//      All site-hero JS/WASM/fixture assets are copied alongside.
//
// Run from packages/site-hero (after `npm run build`):
//   node build-landing.mjs

import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(HERE, "..", "..");
const LANDING_IR = join(REPO_ROOT, "examples/production/landing.voce.json");
const SITE_HERO_DIST = join(HERE, "dist");
const OUT_DIR = join(HERE, "dist-integrated");
const HERO_SLOT_RE = /<p[^>]*>__VOCE_HERO_SLOT__<\/p>/;

// 1 — compile the landing IR fresh (no cache, deterministic)
const tmpHtml = join(HERE, ".landing-tmp.html");
execFileSync("voce", ["compile", LANDING_IR, "-o", tmpHtml, "--no-cache"], {
  encoding: "utf8",
});
const compiledFull = readFileSync(tmpHtml, "utf8");
rmSync(tmpHtml, { force: true });

// 2 — extract body innerHTML
const bodyMatch = compiledFull.match(/<body[^>]*>([\s\S]*)<\/body>/);
if (!bodyMatch) {
  console.error("build-landing: compiled HTML has no <body> tag");
  process.exit(1);
}
const bodyInner = bodyMatch[1];

// 3 — split at the marker
const slotMatch = bodyInner.match(HERO_SLOT_RE);
if (!slotMatch) {
  console.error(
    "build-landing: __VOCE_HERO_SLOT__ marker not found in compiled body. " +
      "Check that landing.voce.json still contains the marker TextNode."
  );
  process.exit(1);
}
const slotIdx = bodyInner.indexOf(slotMatch[0]);
const above = bodyInner.slice(0, slotIdx).trim();
const below = bodyInner.slice(slotIdx + slotMatch[0].length).trim();

// 4 — read site-hero dist (must have been built first)
const siteHeroHtmlPath = join(SITE_HERO_DIST, "index.html");
let siteHeroHtml;
try {
  siteHeroHtml = readFileSync(siteHeroHtmlPath, "utf8");
} catch {
  console.error(
    "build-landing: site-hero dist not found. Run `npm run build` in packages/site-hero first."
  );
  process.exit(1);
}

if (!siteHeroHtml.includes("<!-- VOCE_LANDING_TOP -->")) {
  console.error("build-landing: site-hero index.html missing VOCE_LANDING_TOP marker");
  process.exit(1);
}
if (!siteHeroHtml.includes("<!-- VOCE_LANDING_BOTTOM -->")) {
  console.error("build-landing: site-hero index.html missing VOCE_LANDING_BOTTOM marker");
  process.exit(1);
}

// 5 — splice
const combined = siteHeroHtml
  .replace("<!-- VOCE_LANDING_TOP -->", above)
  .replace("<!-- VOCE_LANDING_BOTTOM -->", below);

// 6 — write to OUT_DIR with all sibling assets
rmSync(OUT_DIR, { recursive: true, force: true });
mkdirSync(OUT_DIR, { recursive: true });
cpSync(SITE_HERO_DIST, OUT_DIR, { recursive: true });
writeFileSync(join(OUT_DIR, "index.html"), combined);

const finalSize = Buffer.byteLength(combined, "utf8");
console.log(
  `\n✓ wrote ${join("dist-integrated", "index.html")} (${finalSize} bytes)\n` +
    `   above-slot:  ${above.length} bytes  (nav)\n` +
    `   live hero:   site-hero scaffold\n` +
    `   below-slot:  ${below.length} bytes  (pipeline · features · cta · footer)\n`
);
