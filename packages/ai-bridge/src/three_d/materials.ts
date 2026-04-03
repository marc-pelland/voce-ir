/**
 * Material description → PBR parameter mapping.
 *
 * Maps natural language material descriptions to concrete
 * metallic/roughness/color values for the IR.
 */

export interface PbrParams {
  baseColor: [number, number, number];
  metallic: number;
  roughness: number;
  emissive: [number, number, number];
}

/** Known material presets mapped from natural language. */
const MATERIAL_PRESETS: Record<string, PbrParams> = {
  // Metals
  "brushed metal": { baseColor: [0.8, 0.8, 0.85], metallic: 0.9, roughness: 0.3, emissive: [0, 0, 0] },
  "polished chrome": { baseColor: [0.9, 0.9, 0.95], metallic: 1.0, roughness: 0.05, emissive: [0, 0, 0] },
  "gold": { baseColor: [1.0, 0.84, 0.0], metallic: 1.0, roughness: 0.2, emissive: [0, 0, 0] },
  "copper": { baseColor: [0.72, 0.45, 0.2], metallic: 1.0, roughness: 0.25, emissive: [0, 0, 0] },
  "rusty metal": { baseColor: [0.55, 0.3, 0.15], metallic: 0.7, roughness: 0.8, emissive: [0, 0, 0] },

  // Non-metals
  "matte plastic": { baseColor: [0.8, 0.2, 0.2], metallic: 0.0, roughness: 0.9, emissive: [0, 0, 0] },
  "glossy plastic": { baseColor: [0.8, 0.2, 0.2], metallic: 0.0, roughness: 0.1, emissive: [0, 0, 0] },
  "rubber": { baseColor: [0.15, 0.15, 0.15], metallic: 0.0, roughness: 0.95, emissive: [0, 0, 0] },
  "wood": { baseColor: [0.55, 0.35, 0.2], metallic: 0.0, roughness: 0.7, emissive: [0, 0, 0] },
  "marble": { baseColor: [0.95, 0.93, 0.9], metallic: 0.0, roughness: 0.3, emissive: [0, 0, 0] },
  "concrete": { baseColor: [0.6, 0.58, 0.55], metallic: 0.0, roughness: 0.85, emissive: [0, 0, 0] },
  "glass": { baseColor: [0.9, 0.95, 1.0], metallic: 0.0, roughness: 0.05, emissive: [0, 0, 0] },
  "ceramic": { baseColor: [0.95, 0.95, 0.92], metallic: 0.0, roughness: 0.4, emissive: [0, 0, 0] },

  // Special
  "neon": { baseColor: [0.1, 0.1, 0.1], metallic: 0.0, roughness: 0.5, emissive: [0.5, 1.0, 0.5] },
  "lava": { baseColor: [0.1, 0.02, 0.0], metallic: 0.0, roughness: 0.8, emissive: [1.0, 0.3, 0.0] },
};

/**
 * Map a material description to PBR parameters.
 * Returns the closest matching preset, or defaults if no match.
 */
export function resolveMaterial(description: string): PbrParams {
  const lower = description.toLowerCase();

  // Direct preset match
  for (const [key, params] of Object.entries(MATERIAL_PRESETS)) {
    if (lower.includes(key)) return { ...params };
  }

  // Keyword matching
  if (lower.includes("metal") || lower.includes("steel")) {
    return { ...MATERIAL_PRESETS["brushed metal"] };
  }
  if (lower.includes("shiny") || lower.includes("glossy") || lower.includes("polished")) {
    return { baseColor: [0.8, 0.8, 0.8], metallic: 0.5, roughness: 0.1, emissive: [0, 0, 0] };
  }
  if (lower.includes("matte") || lower.includes("flat")) {
    return { baseColor: [0.6, 0.6, 0.6], metallic: 0.0, roughness: 0.9, emissive: [0, 0, 0] };
  }
  if (lower.includes("glow") || lower.includes("neon") || lower.includes("emit")) {
    return { ...MATERIAL_PRESETS["neon"] };
  }

  // Default: neutral matte
  return { baseColor: [0.7, 0.7, 0.7], metallic: 0.0, roughness: 0.5, emissive: [0, 0, 0] };
}

/** Get all available material preset names. */
export function getMaterialPresets(): string[] {
  return Object.keys(MATERIAL_PRESETS);
}
