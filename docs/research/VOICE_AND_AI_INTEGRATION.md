# Voce IR — Voice Interface & AI Tool Integration

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how Voce IR supports voice-based conversation and integrates with any AI tool the creator prefers.

---

## 0. Two Key Principles

1. **The conversation medium should match the creator.** Some people type. Some people talk. Some people sketch. Voce IR should meet them where they are.
2. **The AI is pluggable, the platform is not.** The validator, compiler, schema, and CLI are Voce IR. The AI that generates IR can be anything — Claude Code, Cursor, a custom agent, a fine-tuned model, or the built-in Voce AI bridge.

---

## 1. Voice Interface

### 1.1 Why Voice Matters for Voce IR

The project is literally named from "sotto voce" — quiet input, extraordinary output. Voice-based building is the purest expression of the thesis: you describe what you want, and it appears.

Voice also naturally produces better input than typing:
- People describe things more richly when speaking ("I want it to feel warm and inviting, like a coffee shop on a rainy day") vs typing ("warm UI, coffee theme")
- Voice discourages the lazy one-liners that produce bad AI output
- It's more accessible — people with motor disabilities, people who aren't comfortable typing in English
- It reinforces the collaborative feel — talking with the AI, not typing commands at it

### 1.2 Architecture

```
Creator speaks
    │
    ▼
[Speech-to-Text] — Whisper API / Deepgram / browser SpeechRecognition
    │
    ▼
[Text intent] — same as typed input from here
    │
    ▼
[Discovery/Design/Generator agents] — process normally
    │
    ▼
[Response text]
    │
    ▼
[Text-to-Speech] — ElevenLabs / browser SpeechSynthesis / OpenAI TTS
    │
    ▼
Creator hears response
```

The key insight: voice is just an I/O layer. The entire conversational system (discovery agent, design agent, one-question-at-a-time) works identically whether the input is typed or spoken. Voice doesn't change the architecture — it changes the interface.

### 1.3 Implementation Options

**Option A: Browser-based (for web playground/inspector)**
- `SpeechRecognition` API (Chrome/Edge, good accuracy for English)
- `SpeechSynthesis` API (all browsers, robotic but functional)
- Zero backend needed for basic version
- Limitation: browser speech APIs are English-centric and less accurate for technical terms

**Option B: API-based (for CLI and production)**
- **Input:** Whisper (OpenAI) or Deepgram for transcription — high accuracy, multi-language, handles technical vocabulary
- **Output:** ElevenLabs or OpenAI TTS for natural-sounding voice responses
- Better quality, supports all languages, works in CLI via microphone
- Requires API keys and network

**Option C: Hybrid (recommended)**
- Browser APIs for the web playground (zero setup)
- API-based for the CLI and production AI bridge (high quality)
- Configurable: the creator chooses their STT/TTS provider in `voce.config`

### 1.4 The CLI Voice Experience

```bash
# Start a voice session
voce talk

🎤 Listening... (press space to talk, release to send)

Creator: "I want to build a landing page for my coffee 
          subscription service, something dark and warm"

AI: "Nice — a coffee subscription. When you say dark and warm,
     are you thinking something like the Verve Coffee or Blue 
     Bottle aesthetic, or more like a cozy café vibe?"

Creator: "More Blue Bottle, clean but warm"

AI: "Got it — minimal, lots of whitespace, warm accent colors. 
     What's the first thing a visitor should do on the page?"

# ... conversation continues naturally ...

# Switch to text anytime
Creator: [presses 't' to switch to text mode]
> Actually let me type this part — the pricing is $24/mo 
  for Explorer and $42/mo for Connoisseur

# Switch back to voice
Creator: [presses space to talk again]
```

Key UX details:
- **Push-to-talk** (not always-listening) — respectful, avoids false triggers
- **Seamless text/voice switching** — type when precision matters (prices, URLs, technical details), talk when describing feel/vision
- **Transcript visible** — everything spoken is shown as text so the creator can verify the AI heard correctly
- **Voice response is optional** — some people want to hear the AI, some just want to read. Configurable

### 1.5 Voice for the Visual Inspector

The inspector could also support voice:
- Creator looks at the preview, says "make that heading bigger"
- Combined with pointer: creator hovers over an element and says "change this to blue"
- "What state is the cart button in right now?" → AI inspects and responds verbally

This is the most natural debugging interface imaginable — point and talk.

---

## 2. AI Tool Integration — The Pluggable AI Layer

### 2.1 The Architecture Principle

Voce IR has two distinct layers:

```
┌─────────────────────────────────────────┐
│  AI Layer (pluggable)                    │
│  Any tool that can produce valid IR JSON │
│  - Voce AI Bridge (built-in)            │
│  - Claude Code + Voce MCP server        │
│  - Cursor + Voce extension              │
│  - Custom agent via Voce SDK            │
│  - Any LLM via the IR JSON schema       │
└──────────────┬──────────────────────────┘
               │ IR JSON
               ▼
┌─────────────────────────────────────────┐
│  Platform Layer (Voce IR core)           │
│  Schema, Validator, Compiler, CLI        │
│  - voce validate                        │
│  - voce compile                         │
│  - voce test / report / manifest        │
│  - voce preview / deploy                │
└─────────────────────────────────────────┘
```

The platform doesn't care who generated the IR. It validates, compiles, tests, and deploys. The AI layer is an interface to the platform, not part of it.

### 2.2 Integration Paths

#### Path 1: Claude Code + MCP Server (Most Natural for Current Users)

An MCP (Model Context Protocol) server that gives Claude Code direct access to Voce IR tools:

```
MCP Server: voce-ir-server
Tools exposed:
  - voce_validate(ir_json) → validation result
  - voce_compile(ir_json, target) → compiled output path
  - voce_preview(ir_json) → preview URL
  - voce_inspect(ir_json) → human-readable IR summary
  - voce_test(ir_json) → test report
  - voce_schema() → current FlatBuffers schema (for AI context)
  - voce_examples(pattern) → matching example IRs (for few-shot)
  - voce_style_packs() → available style packs
  
Resources exposed:
  - voce://schema/layout — layout node documentation
  - voce://schema/state — state node documentation
  - voce://examples/landing-page — example IR
  - voce://project/manifest — current project's manifest
  - voce://project/history — intent history
```

The workflow in Claude Code:
```
User: "Build a landing page for my coffee subscription"

Claude Code: [uses voce_schema() to load IR schema]
             [uses voce_examples("landing-page") for few-shot]
             [generates IR JSON through conversation]
             [calls voce_validate() to check]
             [calls voce_preview() to show result]
             
User: "The hero needs more punch"

Claude Code: [reads current IR via project manifest]
             [modifies the hero section]
             [validates and previews]
```

**This is the fastest path to integration.** Claude Code users get Voce IR as a tool without leaving their workflow. The MCP server handles all the platform operations.

#### Path 2: Cursor / VS Code Extension

A VS Code extension that provides:
- IR JSON syntax highlighting and validation (red squiggles on invalid IR)
- Schema-aware autocomplete for IR JSON editing
- Preview pane (renders compiled output alongside the IR)
- "Generate from prompt" command (uses configured AI provider to generate IR)
- State machine visualization (inline diagram for StateMachine nodes)
- Tree view of IR node hierarchy

The extension talks to the `voce` CLI under the hood. Any AI (Cursor's built-in, Copilot, Claude via extension) can generate IR JSON — the extension validates and previews it.

#### Path 3: Voce SDK (for Custom Agents)

A TypeScript/Python SDK for building custom AI agents that generate Voce IR:

```typescript
import { VoceSDK } from '@voce-ir/sdk';

const voce = new VoceSDK();

// Load schema for AI context
const schema = await voce.getSchema();

// Validate generated IR
const result = await voce.validate(irJson);

// Compile
const output = await voce.compile(irJson, { target: 'dom' });

// Preview
const url = await voce.preview(irJson);

// Full pipeline
const { compiled, report, manifest } = await voce.build(irJson, {
  target: 'dom',
  runTests: true,
  generateManifest: true,
});
```

This SDK lets anyone build a custom AI agent that generates Voce IR. Want to use GPT-4? Gemini? A local model? An agent framework like LangChain or CrewAI? Just generate valid IR JSON and pipe it through the SDK.

#### Path 4: Raw CLI (for Any Tool)

The simplest integration — any tool that can write a JSON file can use Voce IR:

```bash
# Any tool generates IR JSON
my-custom-ai-tool generate "landing page" > output.voce.json

# Voce CLI does the rest
voce validate output.voce.json
voce compile output.voce.json --target dom -o dist/
voce deploy dist/
```

This is the "universal adapter" — if your tool can write files, it can use Voce IR.

### 2.3 The Schema as the Contract

The FlatBuffers `.fbs` schema files are the contract between the AI layer and the platform. Any AI tool that can produce JSON conforming to this schema can generate Voce IR.

To help AI tools generate valid IR:
- **Published JSON Schema** — derived from FlatBuffers, published as a standard JSON Schema that LLMs understand natively
- **Schema documentation** — every node type documented with examples
- **Example library** — (intent, IR) pairs that any AI tool can use as few-shot examples
- **Validation API** — any tool can call `voce validate` to check its output before compiling

### 2.4 Provider Configuration

```toml
# voce.config.toml

[ai]
# Which AI provider to use for `voce generate` and `voce patch`
provider = "claude"  # claude | openai | custom

[ai.claude]
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"

[ai.openai]
model = "gpt-4o"
api_key_env = "OPENAI_API_KEY"

[ai.custom]
# Any OpenAI-compatible endpoint
endpoint = "http://localhost:11434/v1"  # e.g., Ollama
model = "llama3"

[voice]
enabled = true
stt_provider = "whisper"  # whisper | deepgram | browser
tts_provider = "elevenlabs"  # elevenlabs | openai-tts | browser | none
tts_voice = "warm-professional"

[deploy]
default_target = "vercel"  # vercel | cloudflare | netlify | static
```

The creator chooses their AI, their voice provider, and their deployment target. Voce IR is the platform that connects them all.

---

## 3. How This Changes the Product Positioning

### Before: Voce IR = AI bridge + compiler
The AI bridge is a specific implementation. You use our AI to generate IR.

### After: Voce IR = platform + protocol
The schema is the protocol. The validator/compiler/CLI is the platform. Any AI is welcome. We provide a reference AI bridge, but Claude Code, Cursor, custom agents — they all work.

This is strategically powerful:
- **No lock-in anxiety.** "What if I don't like your AI?" → Use any AI you want
- **Community contribution path.** People can build better AI bridges without touching the compiler
- **Ecosystem growth.** Every AI tool that supports Voce IR expands the ecosystem
- **Defense against competition.** If another AI lab builds a better UI generator, it can target Voce IR's schema rather than building a competing platform

The parallel to SPIR-V deepens: SPIR-V doesn't care which compiler generated the shader IR. It validates and compiles whatever it receives. Voce IR doesn't care which AI generated the UI IR. It validates and compiles whatever it receives.

---

## 4. Integration Priority

| Integration | Effort | Impact | Phase |
| ----------- | ------ | ------ | ----- |
| Raw CLI (JSON in → compiled out) | Low | Medium | 1 |
| MCP Server for Claude Code | Medium | High | 2-3 |
| TypeScript/Python SDK | Medium | High | 3 |
| VS Code extension | High | Medium | 3-4 |
| Voice interface (CLI) | Medium | Medium | 3 |
| Voice interface (web playground) | Low | High | 3 |
| Provider configuration system | Low | Medium | 3 |

**Phase 1 gets the raw CLI for free** — it's just `voce validate` and `voce compile` accepting JSON files. Any AI tool that writes JSON can use Voce IR from day one.

**Phase 2-3 is where integrations matter** — the MCP server for Claude Code is the highest-impact integration because it meets users where they already are.

---

*This document should be read alongside `CONVERSATIONAL_DESIGN.md` for the full picture of how creators interact with Voce IR.*
