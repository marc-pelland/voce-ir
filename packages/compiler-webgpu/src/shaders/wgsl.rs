//! WGSL code generation — produces vertex, fragment, and compute shaders.

use super::material::{Material, MaterialType};

/// Generate a PBR fragment shader in WGSL.
pub fn pbr_fragment_shader() -> String {
    r#"// PBR Fragment Shader (metallic/roughness workflow)
struct MaterialUniforms {
  baseColor: vec4<f32>,
  metallic: f32,
  roughness: f32,
  emissive: vec3<f32>,
};

struct SceneUniforms {
  mvp: mat4x4<f32>,
  model: mat4x4<f32>,
  cameraPos: vec3<f32>,
  lightDir: vec3<f32>,
  lightColor: vec3<f32>,
  ambientColor: vec3<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniforms;
@group(1) @binding(0) var<uniform> material: MaterialUniforms;

@fragment fn fs(
  @location(0) worldPos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {
  let N = normalize(normal);
  let V = normalize(scene.cameraPos - worldPos);
  let L = normalize(-scene.lightDir);
  let H = normalize(V + L);

  // Diffuse (Lambert)
  let NdotL = max(dot(N, L), 0.0);
  let diffuse = material.baseColor.rgb * NdotL * scene.lightColor;

  // Specular (Blinn-Phong approximation of Cook-Torrance)
  let NdotH = max(dot(N, H), 0.0);
  let shininess = mix(8.0, 256.0, 1.0 - material.roughness);
  let specular = pow(NdotH, shininess) * mix(vec3<f32>(0.04), material.baseColor.rgb, material.metallic);

  // Ambient
  let ambient = scene.ambientColor * material.baseColor.rgb;

  // Emissive
  let emissive = material.emissive;

  let color = diffuse + specular * scene.lightColor + ambient + emissive;
  return vec4<f32>(color, material.baseColor.a);
}
"#
    .to_string()
}

/// Generate an unlit fragment shader.
pub fn unlit_fragment_shader() -> String {
    r#"struct MaterialUniforms {
  baseColor: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: MaterialUniforms;

@fragment fn fs(
  @location(0) worldPos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>
) -> @location(0) vec4<f32> {
  return material.baseColor;
}
"#
    .to_string()
}

/// Generate the appropriate fragment shader for a material.
pub fn fragment_shader_for_material(material: &Material) -> String {
    match material.material_type {
        MaterialType::Pbr => pbr_fragment_shader(),
        MaterialType::Unlit => unlit_fragment_shader(),
        MaterialType::Custom => {
            // Custom shaders would be provided directly via ShaderNode
            pbr_fragment_shader() // fallback
        }
    }
}

/// Generate a standard vertex shader with position, normal, and UV output.
pub fn standard_vertex_shader() -> String {
    r#"struct SceneUniforms {
  mvp: mat4x4<f32>,
  model: mat4x4<f32>,
  cameraPos: vec3<f32>,
  lightDir: vec3<f32>,
  lightColor: vec3<f32>,
  ambientColor: vec3<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniforms;

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) worldPos: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>,
};

@vertex fn vs(
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
  @location(2) uv: vec2<f32>
) -> VertexOutput {
  var out: VertexOutput;
  let worldPos = (scene.model * vec4<f32>(position, 1.0)).xyz;
  out.position = scene.mvp * vec4<f32>(position, 1.0);
  out.worldPos = worldPos;
  out.normal = (scene.model * vec4<f32>(normal, 0.0)).xyz;
  out.uv = uv;
  return out;
}
"#
    .to_string()
}
