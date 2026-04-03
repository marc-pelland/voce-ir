/**
 * Base prompt template for IR generation.
 *
 * Wraps the user's intent with instructions for quality output.
 */

export function buildUserPrompt(intent: string): string {
  return `Generate a complete Voce IR JSON document for the following:

${intent}

Requirements:
- Output ONLY the JSON object, no other text
- Include a dark theme with readable contrast
- Include PageMetadata with a descriptive title
- Include SemanticNode for every interactive element
- Include at least one h1 heading
- Make it visually appealing with proper spacing and typography
- Every button/link needs a GestureHandler with keyboard_key: "Enter"
- Include reduced_motion on any animations`;
}
