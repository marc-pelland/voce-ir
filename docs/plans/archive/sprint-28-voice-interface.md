# Sprint 28 — Voice Interface

**Status:** Planned
**Goal:** Add speech-to-text and text-to-speech integration so users can design UIs by talking. Implement `voce talk` for push-to-talk voice sessions with seamless text/voice switching and visible transcripts. After this sprint, the full design conversation can happen by voice — the most natural input for creative work.
**Depends on:** Sprint 27 (MCP server, provider configuration)

---

## Deliverables

1. STT integration: microphone input transcribed to text via provider API
2. TTS integration: AI responses spoken aloud via provider API
3. `voce talk` CLI command: push-to-talk voice design session
4. Seamless mode switching: voice and text interleaved in the same session
5. Live transcript display: spoken words appear as text in real-time
6. Voice-specific conversation tuning: shorter responses, confirmation prompts
7. Audio device selection and noise gate configuration

---

## Tasks

### 1. STT Provider (`src/voice/stt.ts`)

Abstract speech-to-text interface with pluggable providers. Initial implementation: Whisper API (via OpenAI endpoint). Interface: `startListening(): AsyncIterator<Transcript>`, `stopListening()`. Handle: partial transcripts (streaming), final transcript, confidence scores. Audio capture via Node.js native addon or `sox`/`rec` subprocess.

### 2. TTS Provider (`src/voice/tts.ts`)

Abstract text-to-speech interface. Initial implementation: OpenAI TTS API (or ElevenLabs). Interface: `speak(text): Promise<void>`, `stop()`. Configure: voice selection, speed, volume. Play audio via `play` command (sox) or Node.js audio output. Queue responses — don't overlap with user speech.

### 3. Push-to-Talk Engine (`src/voice/ptt-engine.ts`)

Core voice session manager. States: idle, listening, processing, speaking. Push-to-talk triggered by spacebar hold (or configurable key). Visual indicators: recording (red dot), processing (spinner), speaking (speaker icon). Debounce: ignore presses shorter than 200ms. Auto-stop after 30s silence.

### 4. Talk CLI Command (`src/cli/talk-command.ts`)

`voce talk` enters a voice design session. Wraps the ConversationEngine (S23) with voice I/O. Display: live transcript, phase indicator, readiness score. Keyboard shortcuts: Space (push-to-talk), T (switch to text input), M (mute TTS), Q (quit). Falls back to text-only if no microphone detected.

### 5. Mode Switching (`src/voice/mode-switch.ts`)

Allow seamless switching between voice and text within one session. Press T to type a response instead of speaking. Press Space to return to voice. Both modes feed into the same ConversationEngine. Transcript shows both voice and typed inputs with mode indicators ([voice] / [text]).

### 6. Voice-Tuned Prompts (`src/voice/voice-prompts.ts`)

Modify agent system prompts for voice context: shorter responses (under 50 words per turn), explicit confirmation before actions ("I'll add a hero section. Sound good?"), spell out ambiguous terms ("Did you say 'gray' the color or 'great'?"). Avoid lists and formatting that doesn't translate to speech.

### 7. Audio Configuration (`src/voice/audio-config.ts`)

Auto-detect available audio devices. Allow selection via `voce talk --device <name>`. Noise gate threshold to ignore background noise. Sample rate and format configuration. Test command: `voce talk --test` records 3 seconds and plays back to verify setup. Store preferences in `voce.config.toml`.

---

## Files to Create

- `packages/ai-bridge/src/voice/stt.ts`
- `packages/ai-bridge/src/voice/tts.ts`
- `packages/ai-bridge/src/voice/ptt-engine.ts`
- `packages/ai-bridge/src/voice/mode-switch.ts`
- `packages/ai-bridge/src/voice/voice-prompts.ts`
- `packages/ai-bridge/src/voice/audio-config.ts`
- `packages/ai-bridge/src/cli/talk-command.ts`
- `tests/ai-bridge/voice/ptt-engine.test.ts`
- `tests/ai-bridge/voice/mode-switch.test.ts`

---

## Acceptance Criteria

- [ ] `voce talk` starts a voice session with push-to-talk via spacebar
- [ ] Spoken words appear as live transcript text in the terminal
- [ ] AI responses are spoken aloud via TTS and displayed as text
- [ ] Pressing T switches to text input mode within the same session
- [ ] Pressing Space returns to voice mode seamlessly
- [ ] Voice session integrates with ConversationEngine (same flow as `voce design`)
- [ ] Voice-tuned prompts produce shorter, confirmation-oriented responses
- [ ] `voce talk --test` verifies microphone and speaker setup
- [ ] `--device` flag selects specific audio input device
- [ ] Graceful fallback to text-only mode when no microphone is available
- [ ] Session transcript (including voice turns) saved to `.voce/sessions/`
