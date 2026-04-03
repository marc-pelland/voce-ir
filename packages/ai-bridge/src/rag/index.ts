/**
 * RAG retrieval — matches user intent to the closest style pack examples.
 *
 * Phase 1: Tag-based matching (simple, no API dependency).
 * Phase 2+: Embedding-based matching via Claude/OpenAI embeddings.
 */

import type { DiscoveryBrief } from "../agents/types.js";
import type { PackExample, StylePack } from "../packs/types.js";
import { getAllPacks } from "../packs/loader.js";

export interface RetrievalResult {
  /** Style pack this example belongs to. */
  packId: string;
  /** The matched example. */
  example: PackExample;
  /** Match score (0-1). */
  score: number;
}

/**
 * Retrieve the top-k most relevant examples for a given brief.
 *
 * Uses tag overlap scoring: brief keywords vs example tags.
 */
export function retrieveExamples(
  brief: DiscoveryBrief,
  topK: number = 3
): RetrievalResult[] {
  const keywords = extractKeywords(brief);
  const allPacks = getAllPacks();
  const results: RetrievalResult[] = [];

  for (const pack of allPacks) {
    for (const example of pack.examples) {
      const score = scoreMatch(keywords, [
        ...example.tags,
        ...pack.tags,
        ...example.description.toLowerCase().split(/\s+/),
      ]);

      results.push({
        packId: pack.id,
        example,
        score,
      });
    }
  }

  // Sort by score descending, take top-k
  results.sort((a, b) => b.score - a.score);
  return results.slice(0, topK);
}

/**
 * Extract searchable keywords from a DiscoveryBrief.
 */
function extractKeywords(brief: DiscoveryBrief): string[] {
  const words: string[] = [];

  // From purpose
  words.push(...brief.purpose.toLowerCase().split(/\s+/));
  // From sections
  words.push(...brief.sections.map((s) => s.toLowerCase()));
  // From CTAs
  words.push(...brief.ctas.map((c) => c.toLowerCase()));
  // From tone
  words.push(...brief.tone.toLowerCase().split(/\s+/));

  // Filter out common stop words
  const stopWords = new Set([
    "a", "an", "the", "and", "or", "but", "is", "are", "was", "were",
    "be", "been", "being", "have", "has", "had", "do", "does", "did",
    "will", "would", "could", "should", "may", "might", "can", "to",
    "of", "in", "for", "on", "with", "at", "by", "from", "it", "this",
    "that", "which", "who", "what", "how", "when", "where", "why", "i",
    "we", "you", "they", "my", "our", "your", "want", "need", "like",
  ]);

  return words.filter((w) => w.length > 2 && !stopWords.has(w));
}

/**
 * Score the overlap between keywords and tags.
 * Returns 0-1 normalized score.
 */
function scoreMatch(keywords: string[], tags: string[]): number {
  if (keywords.length === 0) return 0;

  const tagSet = new Set(tags.map((t) => t.toLowerCase()));
  let matches = 0;

  for (const keyword of keywords) {
    for (const tag of tagSet) {
      if (tag.includes(keyword) || keyword.includes(tag)) {
        matches++;
        break;
      }
    }
  }

  return matches / keywords.length;
}
