/**
 * Voice-tuned prompt modifications.
 *
 * When in voice mode, agent responses should be:
 * - Shorter (under 50 words per turn)
 * - Confirmation-oriented ("I'll add X. Sound good?")
 * - Avoiding lists and formatting that doesn't translate to speech
 */

/** Modify a system prompt for voice mode. */
export function tuneForVoice(systemPrompt: string): string {
  return `${systemPrompt}

VOICE MODE ACTIVE — adjust your responses:
- Keep responses under 50 words. Be concise.
- End with a simple confirmation question ("Sound good?" / "Ready to proceed?")
- Don't use bullet points, numbered lists, or markdown formatting.
- Spell out abbreviations ("call to action" not "CTA").
- If something is ambiguous, ask for clarification rather than guessing.
- Use natural spoken language, not written documentation style.`;
}

/** Check if a response is appropriate for voice (not too long). */
export function isVoiceAppropriate(response: string): boolean {
  const wordCount = response.split(/\s+/).length;
  return wordCount <= 80; // Allow some flexibility beyond the 50-word target
}

/** Truncate a response for voice delivery if too long. */
export function truncateForVoice(response: string, maxWords: number = 60): string {
  const words = response.split(/\s+/);
  if (words.length <= maxWords) return response;

  // Find a natural break point (sentence ending) near the limit
  const truncated = words.slice(0, maxWords).join(" ");
  const lastPeriod = truncated.lastIndexOf(".");
  const lastQuestion = truncated.lastIndexOf("?");
  const breakPoint = Math.max(lastPeriod, lastQuestion);

  if (breakPoint > truncated.length * 0.5) {
    return truncated.slice(0, breakPoint + 1);
  }

  return truncated + "...";
}
