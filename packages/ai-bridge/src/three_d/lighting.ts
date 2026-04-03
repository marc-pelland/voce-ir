/**
 * Lighting description → light configuration mapping.
 *
 * Maps natural language lighting descriptions to concrete light
 * positions, colors, and intensities for the IR.
 */

export interface LightConfig {
  type: "directional" | "point" | "ambient";
  color: [number, number, number];
  intensity: number;
  position: [number, number, number];
  direction: [number, number, number];
}

export interface LightingSetup {
  lights: LightConfig[];
  background: [number, number, number];
}

/** Known lighting presets. */
const LIGHTING_PRESETS: Record<string, LightingSetup> = {
  "studio": {
    lights: [
      { type: "directional", color: [1.0, 0.98, 0.95], intensity: 1.0, position: [5, 8, 5], direction: [-0.5, -1, -0.5] },
      { type: "directional", color: [0.6, 0.65, 0.8], intensity: 0.4, position: [-3, 4, -2], direction: [0.3, -0.5, 0.3] },
      { type: "ambient", color: [0.3, 0.3, 0.35], intensity: 0.3, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.08, 0.08, 0.1],
  },
  "warm sunset": {
    lights: [
      { type: "directional", color: [1.0, 0.7, 0.3], intensity: 1.2, position: [-8, 3, 2], direction: [0.8, -0.3, -0.2] },
      { type: "ambient", color: [0.3, 0.2, 0.15], intensity: 0.4, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.15, 0.08, 0.05],
  },
  "cool blue": {
    lights: [
      { type: "directional", color: [0.7, 0.8, 1.0], intensity: 1.0, position: [3, 10, 5], direction: [-0.3, -1, -0.5] },
      { type: "ambient", color: [0.15, 0.18, 0.25], intensity: 0.3, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.05, 0.06, 0.1],
  },
  "dramatic": {
    lights: [
      { type: "directional", color: [1.0, 0.95, 0.9], intensity: 1.5, position: [4, 6, 0], direction: [-0.6, -0.8, 0] },
      { type: "ambient", color: [0.05, 0.05, 0.08], intensity: 0.1, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.02, 0.02, 0.03],
  },
  "bright": {
    lights: [
      { type: "directional", color: [1.0, 1.0, 1.0], intensity: 1.0, position: [5, 10, 5], direction: [-0.5, -1, -0.5] },
      { type: "directional", color: [0.8, 0.85, 0.9], intensity: 0.5, position: [-5, 5, -5], direction: [0.5, -0.5, 0.5] },
      { type: "ambient", color: [0.4, 0.4, 0.42], intensity: 0.4, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.92, 0.93, 0.95],
  },
  "product showcase": {
    lights: [
      { type: "directional", color: [1.0, 0.98, 0.96], intensity: 1.0, position: [3, 8, 5], direction: [-0.3, -0.8, -0.5] },
      { type: "directional", color: [0.7, 0.75, 0.85], intensity: 0.3, position: [-4, 4, -3], direction: [0.4, -0.4, 0.3] },
      { type: "point", color: [1.0, 0.95, 0.9], intensity: 0.5, position: [0, 3, 3], direction: [0, 0, 0] },
      { type: "ambient", color: [0.25, 0.25, 0.28], intensity: 0.25, position: [0, 0, 0], direction: [0, 0, 0] },
    ],
    background: [0.1, 0.1, 0.12],
  },
};

/**
 * Resolve a lighting description to a concrete setup.
 */
export function resolveLighting(description: string): LightingSetup {
  const lower = description.toLowerCase();

  // Direct preset match
  for (const [key, setup] of Object.entries(LIGHTING_PRESETS)) {
    if (lower.includes(key)) return structuredClone(setup);
  }

  // Keyword matching
  if (lower.includes("sunset") || lower.includes("warm") || lower.includes("golden")) {
    return structuredClone(LIGHTING_PRESETS["warm sunset"]);
  }
  if (lower.includes("dramatic") || lower.includes("moody") || lower.includes("dark")) {
    return structuredClone(LIGHTING_PRESETS["dramatic"]);
  }
  if (lower.includes("bright") || lower.includes("light") || lower.includes("clean")) {
    return structuredClone(LIGHTING_PRESETS["bright"]);
  }
  if (lower.includes("product") || lower.includes("showcase") || lower.includes("display")) {
    return structuredClone(LIGHTING_PRESETS["product showcase"]);
  }
  if (lower.includes("cool") || lower.includes("blue") || lower.includes("night")) {
    return structuredClone(LIGHTING_PRESETS["cool blue"]);
  }

  // Default: studio
  return structuredClone(LIGHTING_PRESETS["studio"]);
}

/** Get all available lighting preset names. */
export function getLightingPresets(): string[] {
  return Object.keys(LIGHTING_PRESETS);
}
