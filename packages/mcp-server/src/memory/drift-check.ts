// Drift detection — v1 conservative heuristic.
//
// Goal: surface decisions that *might* conflict with a proposed IR, never
// claim certainty. The agent reads the decision text + the matched terms and
// decides whether the drift is real. False positives are tolerable; false
// negatives are the explicit failure mode we accept until v2 ships a real
// rule engine.
//
// Algorithm:
//   1. For each decision, tokenize its summary + rationale into "term tokens":
//      lowercased words ≥ 5 chars, not in a small stopword list.
//   2. Substring-match each token against the lowercased IR JSON.
//   3. If any tokens match, emit a DriftReport with the decision metadata
//      and the matched terms. The report explicitly notes it's a heuristic.
//
// What this catches:
//   - "no Modal" vs IR that contains a Modal node — yes, "modal" matches.
//   - "always SSR" vs IR with cache: Dynamic — no, "ssr" and "dynamic" don't
//     overlap in tokens. v2 with semantic rules would catch this.
// What this misses:
//   - Renamed fields, paraphrased decisions, semantic conflicts that share
//     no surface vocabulary. By design — see file header.

import type { Decision } from "./types.js";

export interface DriftReport {
  decision_id: string;
  decision_summary: string;
  decision_timestamp: string;
  matched_terms: string[];
  /** Always set to the same v1 disclaimer; preserved in the wire format so
   *  downstream agents inherit the "review me" framing. */
  note: string;
}

export interface DriftCheckResult {
  drift: DriftReport[];
  decisions_referenced: string[];
}

const HEURISTIC_NOTE =
  "v1 keyword heuristic — terms from this decision appear in the proposed IR. " +
  "Read the decision and judge whether the conflict is real. False positives expected.";

const STOPWORDS = new Set([
  // Pronouns / determiners
  "their", "there", "these", "those", "which", "where", "while",
  // Modal / auxiliary verbs
  "should", "would", "could", "shall", "might", "could",
  // Generic English
  "thing", "things", "every", "after", "before", "about", "above", "below",
  "between", "within", "without", "during", "again", "further",
  // Common project / process words that aren't useful drift signal
  "always", "never", "still", "also", "only", "other", "another",
  "across", "around", "first", "later", "really", "though",
]);

const MIN_TOKEN_LEN = 5;

function termTokens(text: string): string[] {
  const seen = new Set<string>();
  const out: string[] = [];
  for (const raw of text.split(/[^A-Za-z0-9]+/)) {
    if (raw.length < MIN_TOKEN_LEN) continue;
    const lower = raw.toLowerCase();
    if (STOPWORDS.has(lower)) continue;
    if (seen.has(lower)) continue;
    seen.add(lower);
    out.push(lower);
  }
  return out;
}

/**
 * Detect potential drift between a proposed IR (raw JSON string) and a list
 * of prior decisions. Pure function — no I/O.
 */
export function detectDrift(
  proposedIr: string,
  decisions: readonly Decision[],
): DriftCheckResult {
  const irLower = proposedIr.toLowerCase();
  const drift: DriftReport[] = [];
  const referenced = new Set<string>();

  for (const d of decisions) {
    const tokens = termTokens(`${d.summary} ${d.rationale}`);
    const matched = tokens.filter((t) => irLower.includes(t));
    if (matched.length === 0) continue;
    drift.push({
      decision_id: d.id,
      decision_summary: d.summary,
      decision_timestamp: d.timestamp,
      matched_terms: matched,
      note: HEURISTIC_NOTE,
    });
    referenced.add(d.id);
  }

  return { drift, decisions_referenced: [...referenced] };
}
