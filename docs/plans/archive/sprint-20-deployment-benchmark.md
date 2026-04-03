# Sprint 20 — Deployment & Benchmark

**Status:** Planned
**Goal:** Build deployment adapters (static, Cloudflare), add `voce compile`, `voce preview`, and `voce deploy` commands, benchmark against v0/bolt.new, and tag v0.2.0. After this sprint, Phase 2 is complete: IR compiles to deployable production HTML that beats framework alternatives on every metric.
**Depends on:** Sprint 19 (testing, reports, CLI infrastructure)

---

## Deliverables

1. `adapter-static`: pure HTML + assets in `dist/` directory
2. `adapter-cloudflare`: static + edge functions for ActionNode handlers
3. `voce compile` command: IR → compiled output
4. `voce preview` command: compile + serve + watch for IR changes
5. `voce deploy` command: interactive platform selection + deploy
6. Benchmark: compare TTI/size/Lighthouse/axe-core against v0/bolt.new equivalents
7. v0.2.0 release tag

---

## Tasks

### 1. Adapter Trait (`adapter/mod.rs`)

Define the deployment adapter interface:
