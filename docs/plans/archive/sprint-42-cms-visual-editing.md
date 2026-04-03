# Sprint 42 — CMS Visual Editing

**Status:** Planned
**Phase:** 5 (Visual Inspector & Tooling)
**Depends on:** S39 (inspector core)

---

## Goal

Enable content editing directly on compiled output: click text to edit, swap images, preview changes, and push updates back to the IR via a CMS bridge.

---

## Deliverables

- Content click-to-edit: click any TextNode to edit inline
- Image replacement: click MediaNode to upload/select replacement
- Content change → IR patch generation (JSON Patch delta)
- CMS bridge protocol: headless CMS adapter interface
- Preview/publish flow: edit → preview → confirm → persist
- Content versioning: undo/redo for content edits
- Bulk content editing mode: edit multiple text nodes in a form view
- ContentSlot integration: editable content regions marked in IR
- Role-based access: content editors vs full IR editors

---

## Acceptance Criteria

- [ ] Click any text to edit it inline with rich text controls
- [ ] Image replacement updates the preview immediately
- [ ] Content edits generate valid IR patches
- [ ] Preview shows unsaved changes with visual diff indicators
- [ ] Undo/redo works for content edits (10+ levels)
- [ ] CMS bridge adapter interface is documented and testable
- [ ] Content editor role cannot modify layout or state (only content)
