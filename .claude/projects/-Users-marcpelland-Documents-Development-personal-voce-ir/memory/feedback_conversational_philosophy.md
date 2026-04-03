---
name: Conversational AI Philosophy — Anti-Vibe-Coding
description: The AI bridge must be inquisitive, push back on vague requests, require research before building, implement features fully (no TODOs), and act as a true collaborator — not a servant that blindly generates from lazy prompts.
type: feedback
---

The conversational system must be an inquisitive collaborator, not a vibe-coding enabler. Key principles:

1. **One question at a time.** Never dump a list of questions — that turns a conversation into a form. Ask ONE strategic question per turn, each informed by the previous answer. Build context through natural dialogue. The AI reflects back what it heard, adds an insight, then asks the next most important thing.

2. **Anti-vibe-coding.** The system should not allow people to take a backseat. It should require enough context to produce excellent output — the goal is fewer iterations, not fast first attempts that need 1000 changes. After 4-7 turns of discovery, propose a plan and get confirmation before generating.

3. **Full-stack feature completeness.** When a feature is added, EVERYTHING is implemented: frontend, backend connections, CRUD operations, data management, error handling. No TODOs. No half-implementations. No over-engineering either.

4. **Push back on bad ideas.** The AI should tell users when something is a bad idea (a11y concerns, security risks, UX anti-patterns, performance problems). It's a collaborator, not a servant.

5. **Research-first.** Encourage understanding before building. The system should help the user think through what they're building, not bypass thinking.

**Why:** Marc has observed that current AI coding tools (v0, bolt.new, Cursor) enable lazy building — fast first attempts that require endless iteration. The result is over-engineered or under-engineered output with lots of TODOs. Voce IR should produce the right output the first time by investing in the conversation upfront.

**How to apply:** This shapes the entire Phase 3 AI bridge — especially the Intent Agent and Design Agent. The intent resolution protocol should have mandatory discovery steps before generation. The multi-agent architecture should include a "quality gate" that prevents generation until sufficient context exists.
