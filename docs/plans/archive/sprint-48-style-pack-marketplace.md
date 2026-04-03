# Sprint 48 — Style Pack Marketplace

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S47 (plugin system), S24 (style packs foundation)

---

## Goal

Build a marketplace for community-contributed style packs with preview, install, and revenue sharing for creators.

---

## Deliverables

- Style pack specification: required files, metadata schema, validation rules
- Contribution workflow: submit → review → approve → publish
- Marketplace web UI: browse, search, preview, install style packs
- Preview generation: auto-compile reference page with each style pack
- One-click install: `voce style install <pack-name>` CLI command
- Style pack versioning: semver, dependency tracking, update notifications
- Revenue share model: creator gets percentage of paid pack sales
- Free tier: community packs always free, premium packs for advanced themes
- Quality gates: automated validation, a11y compliance check, performance check
- 5 launch style packs from community contributors

---

## Acceptance Criteria

- [ ] Style packs can be submitted, reviewed, and published
- [ ] Marketplace UI shows previews of each style pack
- [ ] `voce style install` downloads and configures a style pack
- [ ] Style pack updates notify users of new versions
- [ ] Revenue share tracking works for paid packs
- [ ] All marketplace packs pass automated quality gates
- [ ] 5+ style packs available at launch
