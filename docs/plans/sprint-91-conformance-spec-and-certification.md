# Sprint 91 — Voce Conformance Specification & Certification Suite

**Phase:** 7 — Production Readiness → Ecosystem
**Status:** Planned
**Goal:** Turn Voce from "one implementation with internal parity tests" into **an open standard with a portable, versioned conformance suite anyone can certify against**. S68 proves *our* 7 compilers agree internally. S91 generalizes that into a public artifact: a formal IR-semantics specification, a language-agnostic conformance test kit, a certification harness with capability profiles and conformance levels, a public conformance dashboard at `voce-ir.xyz/conformance`, and a self-certification + badge program. This is the open-source leverage play — a single vendor (Vercel) ships one toolchain; an open conformance standard lets *the whole ecosystem* build conformant compilers, adapters, and tooling and prove it, which a closed product structurally cannot match.

**Depends on:** S68 (cross-target parity — provides the fixture set + `SemanticSummary` extractor this generalizes), S79 (agent contract — provides `contract_version`, the versioning policy, and the JSON Schema discipline this reuses). Schema (S02–S05) and validator (S06–S07) as the normative reference. Should follow S68 and S79; can start the spec authoring (D1) in parallel with S68.

---

## Motivation

The project's pitch is "one IR, every platform" and its differentiator is being an **open** AI-native UI substrate, not a vendor product. Today nothing makes that real for third parties: there is no document that says *what conformant Voce behavior is*, and no kit that lets someone who writes a Voce→Qt compiler, a Voce→React adapter, or an alternative validator **prove** it is conformant. Without that, "open standard" is marketing; with it, the ecosystem can out-produce any single vendor because contributions are verifiable.

S68 builds the internal machinery (fixtures + semantic-summary equivalence). S91's insight: that machinery, made portable, versioned, and public, *is* the standard. SPIR-V has a conformance test suite; WebGL/WebGPU have CTS; Wasm has the spec test suite. Voce needs the same to be credible as infrastructure rather than an app. This sprint is deliberately ambitious — it is the moat.

---

## Deliverables

### D1. Normative conformance specification (`docs/spec/CONFORMANCE.md`)

The single normative document. RFC 2119 language (MUST / SHOULD / MAY). Sections:

- **Scope & versioning** — bound to `schema_version` and the S79 `contract_version`. The conformance spec carries its own `conformance_version` (semver); a numbered erratum process.
- **The IR semantic model** — for every node-type union variant, the *observable semantic contract* a conformant compiler MUST preserve (heading hierarchy, interactive affordances, accessible name/role, form field semantics, state-machine reachability, data binding intent, i18n message resolution, motion-reduction honoring). This is the heart: it defines *meaning*, not representation.
- **Required vs. optional behavior** — what every target MUST do, what MAY degrade, and the rules for degradation (a conformant target that cannot represent a feature MUST emit a documented compile-time diagnostic — silent drop is non-conformance).
- **Determinism & idempotence requirements** — same IR + same target + same options ⇒ byte-stable artifact; recompiling a compiled-then-decompiled IR is semantically idempotent.
- **The oracle** — the Reference Implementation (this repo's DOM compiler + validator) is normative where the prose is ambiguous, and the spec says so explicitly, with a defect-reporting path when oracle and prose disagree.

This document is authored by hand, reviewed, and is the thing the suite tests *against* — not generated from the suite (that would be circular).

### D2. Portable conformance test kit (`conformance/` — distributable, not just `tests/`)

A standalone, language-agnostic kit any implementer can run, not a Rust-only integration test:

- **Golden corpus** — the S68 fixtures, expanded to full union coverage (every node type, every enum variant, every required/optional field combination, plus adversarial cases: deeply nested, max-size, Unicode/RTL, reduced-motion, dangling-but-tolerated references). Target ≥ 60 fixtures grouped by **profile** (see D4).
- **Expected semantic summaries** — each fixture ships a canonical `expected/semantic-summary.json` (the S68 extractor's output, frozen and reviewed), versioned with `conformance_version`. This is the portable contract: an external implementer compiles the fixture with their tool, runs the published summary extractor (D3) on their artifact, and diffs.
- **Manifest** — `conformance/manifest.json` listing every fixture, its profile, the features it exercises, and which conformance level/profiles it gates. Machine-consumable; validates against a published JSON Schema (S79 discipline).
- **Distribution** — the kit is versioned and downloadable as a release artifact (tarball) decoupled from the Rust workspace, so a Go or TypeScript implementer can consume it without building Voce.

### D3. Conformance harness + summary extractor as a portable contract

The S68 `SemanticSummary` extractor is currently internal Rust over our own artifacts. S91 splits it into a **specified algorithm** plus reference implementations:

- **`docs/spec/SEMANTIC_SUMMARY.md`** — the normative algorithm for deriving a semantic summary from a *compiled artifact* (HTML/SwiftUI/Compose/…), per target family. Specified precisely enough to reimplement.
- **Reference extractor** — the Rust implementation, exposed via `voce conformance extract --target <id> <artifact>` so external toolchains can use ours rather than reimplementing (lowers the certification barrier — the friction Vercel's closed stack imposes is exactly what we remove).
- **`voce conformance run`** — the certification command: takes a target adapter (built-in or external via a documented adapter protocol — a CLI contract: "given this fixture, produce an artifact at this path"), runs the corpus, extracts summaries, diffs against expected, emits a contract-versioned `conformance-report.json` (passes, failures with diffs, profile/level achieved).

### D4. Conformance levels & capability profiles

Not pass/fail binary — a capability lattice, so a niche target (e.g., Email HTML, e-paper, TUI) can be *legitimately conformant within its profile*:

- **Levels:** `Core` (layout, text, semantics, a11y — every conformant target MUST pass) → `Standard` (forms, theming, i18n, navigation) → `Full` (state machines, motion, data binding, gestures).
- **Profiles:** capability tags a target declares it supports — `static`, `interactive`, `animated`, `data-bound`, `3d`. A target certifies as e.g. *"Core+Standard, profiles: static, interactive — Email family"*. The matrix from S68 becomes the *result* of running profiles, not a hand-kept doc.
- **Degradation conformance:** a target may pass a fixture by *correctly degrading* (documented diagnostic + semantically-reduced output that the spec permits for that profile). The harness distinguishes `pass` / `pass-degraded` / `fail` / `n-a (out of profile)`.

### D5. Self-certification & badge program

The ecosystem-leverage payload:

- **`voce conformance certify`** — runs the suite for a declared level+profile set and produces a signed (sigstore/cosign or detached signature) `conformance-attestation.json` pinned to `conformance_version` + corpus hash. Reproducible by anyone.
- **Badge + registry** — a published JSON registry (`conformance/registry.json` in-repo, surfaced at the dashboard) of known conformant implementations with their attestation, level, profiles, and date. Third parties submit via PR with their attestation; CI re-runs the suite against their adapter protocol endpoint to verify before merge (trust, but verify).
- **README badge SVG** — `Voce Conformant — Core+Full` style badges generated from the attestation.

### D6. Public conformance dashboard (`voce-ir.xyz/conformance`)

Generated from the latest CI run + registry:

- The full fixture × target × level/profile matrix (supersedes `docs/compatibility-matrix.md` from S68 — that becomes a generated view).
- Per-target conformance over time (regressions visible historically).
- The registry of external conformant implementations.
- A "run it yourself" quickstart (download kit, run harness, read report) — make external certification a 10-minute path.

### D7. CI: conformance as a release gate

- Job `conformance-suite` runs the full corpus across all built-in targets on every PR; fails on any regression below the recorded baseline level/profile.
- A release MUST NOT ship if the Reference Implementation drops conformance level (the oracle cannot regress the standard).
- Corpus hash + `conformance_version` are emitted into release notes and the S79 `voce skills` manifest, so an agent can discover *which conformance version this build satisfies*.

### D8. Spec ↔ suite consistency guard

A meta-test asserting the spec and suite cannot silently drift: every normative MUST in `CONFORMANCE.md` is tagged with a stable clause ID (e.g., `CONF-A11Y-014`); every fixture declares which clause IDs it exercises; CI fails if any MUST clause has zero covering fixtures (an untested normative requirement is a spec bug, not just a coverage gap).

---

## Conformance Versioning Contract

- `conformance_version` is semver, independent of `voce_version`, aligned with S79 `contract_version` major.
- Adding fixtures or tightening the *extractor* is a **minor** bump; changing a normative MUST or expected summary is **major**.
- Each release pins: `conformance_version`, corpus SHA-256, oracle `voce_version`. Attestations are only valid against the pinned tuple.
- Deprecation policy: a normative clause is `deprecated` for one major before removal; never silently changed.

---

## Acceptance Criteria

- [ ] `docs/spec/CONFORMANCE.md` exists, RFC-2119, with stable clause IDs and the oracle clause
- [ ] `docs/spec/SEMANTIC_SUMMARY.md` specifies the extraction algorithm precisely enough to reimplement
- [ ] `conformance/` kit: ≥ 60 fixtures, frozen expected summaries, `manifest.json` (schema-validated), released as a standalone versioned tarball
- [ ] `voce conformance run | extract | certify` implemented; reports are `contract_version`-stamped and schema-valid
- [ ] Levels (Core/Standard/Full) and profiles implemented; harness reports `pass | pass-degraded | fail | n-a`
- [ ] All 7 built-in targets certified at their honest level/profile; results match S68's matrix (S68 matrix now generated from this)
- [ ] Self-certification produces a reproducible signed attestation; `registry.json` + PR-verify flow exists with ≥ 1 reference entry (the RI itself)
- [ ] `voce-ir.xyz/conformance` dashboard live, generated from CI + registry
- [ ] CI `conformance-suite` gates regressions; release blocked on RI level regression
- [ ] Spec↔suite guard: every MUST clause has ≥ 1 covering fixture; CI enforces
- [ ] Conformance version + corpus hash surfaced in `voce skills` (S79) and release notes

---

## Relationship to S68 and S79 (no duplication)

- **S68** stays the *internal parity sprint*: it builds the fixture seed set and the first `SemanticSummary` extractor and proves our 7 compilers agree. S91 **consumes and generalizes** it — expands the corpus to full profile coverage, freezes/reviews the expected summaries, and specifies the extractor as a portable algorithm. After S91, `docs/compatibility-matrix.md` is a *generated view*, not hand-maintained.
- **S79** provides the contract plumbing: `contract_version`, JSON Schema discipline, stability policy, and the `voce` agent-command pattern. S91's `voce conformance` family is a member of that contract; the conformance version is surfaced through S79's `voce skills`. S79's A5 runner skeleton is fully realized here.
- Net: S68 = "do our compilers agree?", S79 = "can an agent discover and drive us?", S91 = "can the world build conformant Voce tooling and prove it?"

---

## Risks

1. **Spec authoring is the long pole.** Normative semantics for every union variant is real spec work, not codegen. Mitigation: D1 can start during S68; scope Core profile first, ship Standard/Full clauses as minor `conformance_version` bumps rather than blocking the sprint.
2. **The extractor is itself a compiler-shaped artifact.** A bug there masks or invents non-conformance. Mitigation (carried from S68): every fixture's expected summary is hand-reviewed at freeze time; the spec↔suite guard (D8) catches uncovered clauses; oracle clause resolves prose ambiguity.
3. **Profiles can become a loophole** ("we conform — to the empty profile"). Mitigation: `Core` is mandatory for *any* conformance claim; badges always render level *and* profiles; registry PRs are CI-verified, not self-asserted.
4. **External adapter protocol is a new public API surface.** Keep it minimal (stdin/stdout or file-path CLI contract), versioned with `conformance_version`, documented with a reference adapter shim.
5. **Signing infrastructure scope.** If sigstore integration over-runs, ship detached-signature + corpus-hash attestation first; cosign/transparency-log is a fast-follow, not a blocker.
6. **Schema gaps surface as "non-conformance."** If a fixture can't express a needed semantic, that's an S72 finding. Document, mark the clause `provisional`, defer — do not weaken the spec to pass.

---

## Out of Scope

- Visual/pixel regression across targets (semantic conformance only — pixel parity is explicitly *not* a Voce goal)
- Performance conformance (size/speed budgets are S71; a slow-but-correct target is conformant)
- New target compilers (the suite tests targets; it does not add them)
- Hosted certification-as-a-service (the kit is self-serve and offline-capable by design; a hosted runner is post-v1.1 if ever)
- Legal/trademark policy for the badge (flagged for `docs/` + the S60 community launch, not engineered here)
