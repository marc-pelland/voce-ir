/**
 * Text-to-Speech provider interface.
 *
 * Pluggable: OpenAI TTS (default), ElevenLabs, browser SpeechSynthesis.
 */

export interface TtsProvider {
  name: string;
  /** Speak the given text aloud. Returns when playback finishes. */
  speak(text: string): Promise<void>;
  /** Stop current playback. */
  stop(): void;
  /** Check if the provider is configured. */
  isAvailable(): boolean;
}

/**
 * OpenAI TTS provider — uses OpenAI's TTS endpoint.
 */
export class OpenAiTtsProvider implements TtsProvider {
  name = "openai-tts";

  isAvailable(): boolean {
    return !!process.env.OPENAI_API_KEY;
  }

  async speak(text: string): Promise<void> {
    // In production, POST to https://api.openai.com/v1/audio/speech
    // Then pipe the audio stream to a player (sox play, afplay, etc)
    console.error(`[TTS] ${text}`);
  }

  stop(): void {
    // Kill the audio player process
  }
}

/**
 * Null provider — prints to stderr instead of speaking.
 * Used in text-only mode or when TTS is not configured.
 */
export class NullTtsProvider implements TtsProvider {
  name = "none";
  isAvailable(): boolean { return true; }
  async speak(text: string): Promise<void> {
    // Silent — text is already displayed in the terminal
  }
  stop(): void {}
}

/** Get the configured TTS provider. */
export function getTtsProvider(providerName: string = "none"): TtsProvider {
  switch (providerName) {
    case "openai-tts":
      return new OpenAiTtsProvider();
    case "elevenlabs":
      return new OpenAiTtsProvider(); // placeholder — same interface
    default:
      return new NullTtsProvider();
  }
}
