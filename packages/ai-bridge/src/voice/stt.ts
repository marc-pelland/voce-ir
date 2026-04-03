/**
 * Speech-to-Text provider interface.
 *
 * Pluggable: Whisper API (default), Deepgram, browser SpeechRecognition.
 * The actual audio capture is handled by the PTT engine; STT just
 * transcribes audio buffers.
 */

export interface Transcript {
  /** The transcribed text. */
  text: string;
  /** Whether this is a final transcript (vs partial/streaming). */
  isFinal: boolean;
  /** Confidence score (0-1). */
  confidence: number;
  /** Duration in milliseconds. */
  durationMs: number;
}

export interface SttProvider {
  /** Provider name for display. */
  name: string;
  /** Transcribe an audio buffer. */
  transcribe(audio: Buffer, format?: string): Promise<Transcript>;
  /** Check if the provider is configured (API key present, etc). */
  isAvailable(): boolean;
}

/**
 * Whisper API provider — uses OpenAI's Whisper endpoint.
 * Requires OPENAI_API_KEY environment variable.
 */
export class WhisperSttProvider implements SttProvider {
  name = "whisper";

  isAvailable(): boolean {
    return !!process.env.OPENAI_API_KEY;
  }

  async transcribe(audio: Buffer, format = "wav"): Promise<Transcript> {
    const apiKey = process.env.OPENAI_API_KEY;
    if (!apiKey) throw new Error("OPENAI_API_KEY required for Whisper STT");

    // In production, this would POST to https://api.openai.com/v1/audio/transcriptions
    // For now, return a placeholder that demonstrates the interface
    return {
      text: "[Whisper transcription would appear here]",
      isFinal: true,
      confidence: 0.95,
      durationMs: audio.length / 32, // rough estimate
    };
  }
}

/**
 * Null provider — for text-only mode or when no STT is available.
 */
export class NullSttProvider implements SttProvider {
  name = "none";
  isAvailable(): boolean { return true; }
  async transcribe(): Promise<Transcript> {
    return { text: "", isFinal: true, confidence: 0, durationMs: 0 };
  }
}

/** Get the configured STT provider. */
export function getSttProvider(providerName: string = "whisper"): SttProvider {
  switch (providerName) {
    case "whisper":
      return new WhisperSttProvider();
    default:
      return new NullSttProvider();
  }
}
