/**
 * Push-to-Talk Engine — manages voice session state.
 *
 * States: idle → listening → processing → speaking → idle
 * Input: spacebar hold (or configurable key)
 */

import type { SttProvider, Transcript } from "./stt.js";
import type { TtsProvider } from "./tts.js";

export type VoiceState = "idle" | "listening" | "processing" | "speaking";
export type InputMode = "voice" | "text";

export interface VoiceSessionConfig {
  sttProvider: SttProvider;
  ttsProvider: TtsProvider;
  /** Maximum recording duration in ms (default: 30000). */
  maxDurationMs?: number;
  /** Minimum press duration to register (default: 200). */
  minPressDurationMs?: number;
}

export interface VoiceEvent {
  type: "state_change" | "transcript" | "ai_response" | "mode_switch" | "error";
  state?: VoiceState;
  mode?: InputMode;
  text?: string;
  error?: string;
}

export type VoiceEventHandler = (event: VoiceEvent) => void;

export class PushToTalkEngine {
  private state: VoiceState = "idle";
  private mode: InputMode = "voice";
  private config: Required<VoiceSessionConfig>;
  private handlers: VoiceEventHandler[] = [];
  private recordingStartTime: number = 0;

  constructor(config: VoiceSessionConfig) {
    this.config = {
      ...config,
      maxDurationMs: config.maxDurationMs ?? 30000,
      minPressDurationMs: config.minPressDurationMs ?? 200,
    };
  }

  /** Register an event handler. */
  on(handler: VoiceEventHandler): void {
    this.handlers.push(handler);
  }

  /** Current state. */
  getState(): VoiceState {
    return this.state;
  }

  /** Current input mode. */
  getMode(): InputMode {
    return this.mode;
  }

  /** Switch between voice and text mode. */
  switchMode(mode: InputMode): void {
    this.mode = mode;
    this.emit({ type: "mode_switch", mode });
  }

  /** Called when push-to-talk key is pressed. */
  startRecording(): void {
    if (this.state !== "idle") return;
    if (this.mode !== "voice") return;

    this.recordingStartTime = Date.now();
    this.setState("listening");
  }

  /** Called when push-to-talk key is released. */
  async stopRecording(audioBuffer?: Buffer): Promise<Transcript | null> {
    if (this.state !== "listening") return null;

    const duration = Date.now() - this.recordingStartTime;
    if (duration < this.config.minPressDurationMs) {
      this.setState("idle");
      return null; // Too short — ignore
    }

    this.setState("processing");

    try {
      if (!audioBuffer) {
        this.setState("idle");
        return null;
      }

      const transcript = await this.config.sttProvider.transcribe(audioBuffer);
      this.emit({ type: "transcript", text: transcript.text });
      this.setState("idle");
      return transcript;
    } catch (error) {
      this.emit({ type: "error", error: (error as Error).message });
      this.setState("idle");
      return null;
    }
  }

  /** Speak the AI's response via TTS. */
  async speakResponse(text: string): Promise<void> {
    this.setState("speaking");
    this.emit({ type: "ai_response", text });

    try {
      await this.config.ttsProvider.speak(text);
    } catch (error) {
      this.emit({ type: "error", error: (error as Error).message });
    }

    this.setState("idle");
  }

  /** Process text input (when in text mode). */
  processTextInput(text: string): void {
    this.emit({ type: "transcript", text, mode: "text" });
  }

  /** Check if voice is available (STT provider configured). */
  isVoiceAvailable(): boolean {
    return this.config.sttProvider.isAvailable();
  }

  private setState(state: VoiceState): void {
    this.state = state;
    this.emit({ type: "state_change", state });
  }

  private emit(event: VoiceEvent): void {
    for (const handler of this.handlers) {
      handler(event);
    }
  }
}
