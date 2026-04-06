//! Material system — PBR and basic material definitions.
//!
//! Materials are compiled to WGSL shader code. The PBR material uses
//! a metallic/roughness workflow compatible with glTF standards.

/// A material definition extracted from the IR.
#[derive(Debug, Clone)]
pub struct Material {
    pub id: String,
    pub material_type: MaterialType,
    /// Base color (RGBA, linear space).
    pub base_color: [f32; 4],
    /// Metallic factor (0.0 = dielectric, 1.0 = metal).
    pub metallic: f32,
    /// Roughness factor (0.0 = mirror, 1.0 = fully rough).
    pub roughness: f32,
    /// Emissive color (for self-illuminating surfaces).
    pub emissive: [f32; 3],
    /// Opacity (1.0 = opaque).
    pub opacity: f32,
}

#[derive(Debug, Clone, Default)]
pub enum MaterialType {
    /// Physically-based rendering (metallic/roughness).
    #[default]
    Pbr,
    /// Simple unlit material (just base color, no lighting).
    Unlit,
    /// Custom shader (ShaderNode provides WGSL directly).
    Custom,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            material_type: MaterialType::Pbr,
            base_color: [0.8, 0.3, 0.2, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            opacity: 1.0,
        }
    }
}

impl Material {
    /// Generate the WGSL uniform struct fields for this material.
    pub fn wgsl_uniforms(&self) -> String {
        "  baseColor: vec4<f32>,\n  metallic: f32,\n  roughness: f32,\n  emissive: vec3<f32>,"
            .to_string()
    }

    /// Generate the WGSL uniform data as a JavaScript Float32Array.
    pub fn js_uniform_data(&self) -> String {
        format!(
            "[{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, 0.0]",
            self.base_color[0],
            self.base_color[1],
            self.base_color[2],
            self.base_color[3],
            self.metallic,
            self.roughness,
            self.emissive[0],
            self.emissive[1],
            self.emissive[2],
        )
    }
}
