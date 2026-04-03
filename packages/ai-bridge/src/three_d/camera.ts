/**
 * Camera behavior mapping — natural language to camera configuration.
 */

export interface CameraConfig {
  type: "perspective" | "orthographic";
  position: [number, number, number];
  target: [number, number, number];
  fov: number;
  orbitEnabled: boolean;
  autoRotate: boolean;
  autoRotateSpeed: number;
}

const CAMERA_PRESETS: Record<string, CameraConfig> = {
  "orbit": {
    type: "perspective", position: [0, 2, 5], target: [0, 0, 0],
    fov: 45, orbitEnabled: true, autoRotate: false, autoRotateSpeed: 0,
  },
  "auto rotate": {
    type: "perspective", position: [0, 2, 5], target: [0, 0, 0],
    fov: 45, orbitEnabled: true, autoRotate: true, autoRotateSpeed: 1.0,
  },
  "top down": {
    type: "orthographic", position: [0, 10, 0], target: [0, 0, 0],
    fov: 45, orbitEnabled: false, autoRotate: false, autoRotateSpeed: 0,
  },
  "close up": {
    type: "perspective", position: [0, 1, 2.5], target: [0, 0.5, 0],
    fov: 35, orbitEnabled: true, autoRotate: false, autoRotateSpeed: 0,
  },
  "wide angle": {
    type: "perspective", position: [0, 3, 8], target: [0, 0, 0],
    fov: 65, orbitEnabled: true, autoRotate: false, autoRotateSpeed: 0,
  },
};

/**
 * Resolve a camera description to configuration.
 */
export function resolveCamera(description: string): CameraConfig {
  const lower = description.toLowerCase();

  for (const [key, config] of Object.entries(CAMERA_PRESETS)) {
    if (lower.includes(key)) return { ...config };
  }

  if (lower.includes("rotate") || lower.includes("spin") || lower.includes("revolve")) {
    return { ...CAMERA_PRESETS["auto rotate"] };
  }
  if (lower.includes("orbit") || lower.includes("drag") || lower.includes("interactive")) {
    return { ...CAMERA_PRESETS["orbit"] };
  }
  if (lower.includes("close") || lower.includes("detail") || lower.includes("zoom")) {
    return { ...CAMERA_PRESETS["close up"] };
  }

  // Default: orbit
  return { ...CAMERA_PRESETS["orbit"] };
}
