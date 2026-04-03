# Sprint 36 — Integration: AI Bridge for 3D

**Status:** Planned
**Phase:** 4 (Multi-Target Compilation)
**Depends on:** S32 (shaders/particles), S35 (degradation)

---

## Goal

Extend the AI bridge to support 3D scene descriptions in natural language. Users describe 3D content conversationally, and the system generates Scene3D IR.

---

## Deliverables

- 3D intent vocabulary: natural language → Scene3D/MeshNode/ShaderNode mappings
- Discovery Agent extensions: 3D-specific questions (lighting mood, camera behavior, interactivity)
- Generator prompt updates: Scene3D examples in few-shot library
- 20+ intent-IR golden pairs for 3D scenes
- Style pack extensions: 3D presets (product showcase, architectural, abstract art)
- Material description → PBR parameter mapping ("brushed metal" → metallic:0.9, roughness:0.3)
- Lighting description → light configuration ("warm sunset" → directional + ambient)
- Camera behavior from intent ("orbit around the product" → orbit controls)
- Repair agent handles WebGPU-specific validation errors

---

## Acceptance Criteria

- [ ] "Show me a rotating product on a dark background" generates valid Scene3D IR
- [ ] Material descriptions map to correct PBR parameters
- [ ] Lighting descriptions produce appropriate light configurations
- [ ] 3D golden pairs achieve >90% first-attempt validity
- [ ] Repair agent fixes common 3D IR issues (missing camera, no lights)
- [ ] End-to-end: natural language → 3D scene → compiled WebGPU output
