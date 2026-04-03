# Sprint 39 — Inspector Core

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S20 (compiler-dom), S38 (v0.4.0)

---

## Goal

Build the core inspector overlay that renders on any Voce-compiled output, enabling click-to-inspect element selection and an IR node property panel.

---

## Deliverables

- `packages/inspector/` TypeScript package
- Inspector overlay injection: toggleable UI layer on compiled output
- Scene graph tree view: hierarchical display of all IR nodes
- Click-to-inspect: click any DOM element, highlight it, show corresponding IR node
- IR node property panel: all properties, computed styles, state, bindings
- Source-map-like mapping: compiled DOM element ↔ IR node ID
- Inspector activation: keyboard shortcut (Ctrl+Shift+I) or `voce preview --inspect`
- Overlay does not interfere with page layout or event handling
- 10+ tests for overlay rendering and node selection

---

## Acceptance Criteria

- [ ] Inspector overlay renders on any Voce-compiled DOM output
- [ ] Click any element to see its IR node properties
- [ ] Scene graph tree view shows complete node hierarchy
- [ ] Selecting a node in tree highlights corresponding DOM element
- [ ] Selecting a DOM element highlights corresponding tree node
- [ ] Inspector can be toggled on/off without page reload
- [ ] Overlay does not shift page layout or intercept user events when hidden
