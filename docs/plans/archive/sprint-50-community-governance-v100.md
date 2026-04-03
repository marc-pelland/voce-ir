# Sprint 50 — Community Governance & v1.0.0

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S45-S49 (all Phase 6 sprints)

---

## Goal

Establish community governance, publish a fine-tuned model for IR generation, and release v1.0.0 as a stable community-driven standard.

---

## Deliverables

- Community governance model: RFC process, decision-making framework, maintainer roles
- Contributing guide: code of conduct, PR workflow, review standards
- Fine-tuned model: trained on intent-IR corpus from Phases 1-5 (1000+ pairs)
- Model evaluation: compare fine-tuned vs base model on validity rate, quality, speed
- Schema stability guarantee: v1.0.0 schema is backward-compatible going forward
- Migration tooling: `voce migrate` for upgrading IR from pre-1.0 schemas
- Third-party integration guides: how to build on Voce IR
- Partnership outreach: AI labs, CMS providers, design tool vendors
- v1.0.0 release: all crates, npm packages, documentation site
- Launch blog post and demo showcase

---

## Acceptance Criteria

- [ ] RFC process documented and first community RFC submitted
- [ ] 3+ external contributors have merged PRs
- [ ] Fine-tuned model achieves >98% first-attempt validity (vs >95% base)
- [ ] Schema backward compatibility test suite passes
- [ ] `voce migrate` upgrades v0.x IR to v1.0 format
- [ ] Documentation site live with guides for all 6 compile targets
- [ ] v1.0.0 tagged and published across all package registries
