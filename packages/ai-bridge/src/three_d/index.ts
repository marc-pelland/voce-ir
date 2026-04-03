/**
 * 3D AI Bridge — natural language to Scene3D IR mappings.
 *
 * Extends the AI bridge with vocabulary for describing 3D scenes,
 * materials, lighting, and camera behavior.
 */

export { resolveMaterial, getMaterialPresets, type PbrParams } from "./materials.js";
export { resolveLighting, getLightingPresets, type LightConfig, type LightingSetup } from "./lighting.js";
export { resolveCamera, type CameraConfig } from "./camera.js";

/**
 * Build 3D-specific context for the Generator Agent.
 *
 * Injects material presets, lighting presets, and camera options
 * into the system prompt so the AI knows how to generate Scene3D IR.
 */
export function build3DContext(): string {
  return `## 3D Scene Generation

When the user describes a 3D scene, generate Scene3D IR with:

### Materials (PBR metallic/roughness)
Map descriptions to PBR parameters:
- "brushed metal" → metallic: 0.9, roughness: 0.3
- "polished chrome" → metallic: 1.0, roughness: 0.05
- "matte plastic" → metallic: 0.0, roughness: 0.9
- "wood" → metallic: 0.0, roughness: 0.7, baseColor warm brown
- "glass" → metallic: 0.0, roughness: 0.05, baseColor near-white
- "gold" → metallic: 1.0, roughness: 0.2, baseColor [1.0, 0.84, 0.0]

### Lighting Setups
- "studio" → key light + fill light + ambient (default for products)
- "warm sunset" → orange directional + warm ambient
- "dramatic" → single strong directional + dark ambient
- "bright" → multi-directional + strong ambient (white background)
- "product showcase" → 3-point lighting + subtle ambient

### Camera
- "orbit" → user can drag to rotate (default)
- "auto rotate" → slow continuous rotation
- "close up" → shorter focal length, closer position
- "wide angle" → higher FOV, further back

### Scene3D IR Structure
\`\`\`json
{
  "value_type": "Scene3D",
  "value": {
    "node_id": "scene",
    "camera": { "position": {"x":0,"y":2,"z":5}, "target": {"x":0,"y":0,"z":0}, "fov": 45, "orbit": true },
    "background": {"r":20,"g":20,"b":25,"a":255},
    "children": [
      { "value_type": "MeshNode", "value": { "node_id": "product", "primitive": "cube", "position": {"x":0,"y":0,"z":0}, "color": {"r":200,"g":80,"b":60,"a":255} } }
    ]
  }
}
\`\`\`

Always include: camera with position/target, at least one light source, and ReducedMotion alternative for any animations.`;
}
