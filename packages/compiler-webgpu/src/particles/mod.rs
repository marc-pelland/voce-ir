//! Particle system — GPU compute shader pipeline for particle effects.
//!
//! ParticleSystem IR nodes compile to:
//! 1. A compute shader that updates particle positions/velocities/lifetimes
//! 2. A vertex shader that renders particles as billboarded quads
//! 3. JavaScript initialization code that sets up buffers and dispatches

/// Particle system configuration extracted from IR.
#[derive(Debug, Clone)]
pub struct ParticleConfig {
    pub id: String,
    pub max_particles: u32,
    pub spawn_rate: f32,
    pub lifetime: f32,
    pub emitter: EmitterShape,
    pub initial_velocity: [f32; 3],
    pub velocity_spread: f32,
    pub gravity: [f32; 3],
    pub color_start: [f32; 4],
    pub color_end: [f32; 4],
    pub size_start: f32,
    pub size_end: f32,
}

#[derive(Debug, Clone, Default)]
pub enum EmitterShape {
    #[default]
    Point,
    Sphere {
        radius: f32,
    },
    Cone {
        angle: f32,
        radius: f32,
    },
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            id: "particles".to_string(),
            max_particles: 1000,
            spawn_rate: 50.0,
            lifetime: 3.0,
            emitter: EmitterShape::Point,
            initial_velocity: [0.0, 2.0, 0.0],
            velocity_spread: 0.5,
            gravity: [0.0, -9.8, 0.0],
            color_start: [1.0, 0.8, 0.2, 1.0],
            color_end: [1.0, 0.2, 0.0, 0.0],
            size_start: 0.1,
            size_end: 0.02,
        }
    }
}

/// Generate the WGSL compute shader for particle simulation.
pub fn particle_compute_shader(config: &ParticleConfig) -> String {
    format!(
        r#"// Particle Compute Shader — updates {max} particles per frame
struct Particle {{
  pos: vec3<f32>,
  vel: vec3<f32>,
  life: f32,
  maxLife: f32,
}};

struct SimParams {{
  deltaTime: f32,
  gravity: vec3<f32>,
  spawnRate: f32,
  time: f32,
}};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle, {max}>;
@group(0) @binding(1) var<uniform> params: SimParams;

@compute @workgroup_size(64)
fn simulate(@builtin(global_invocation_id) id: vec3<u32>) {{
  let i = id.x;
  if (i >= {max}u) {{ return; }}

  var p = particles[i];

  if (p.life <= 0.0) {{
    // Dead particle — respawn if within spawn budget
    if (f32(i) < params.spawnRate * params.deltaTime * {max}.0) {{
      p.pos = vec3<f32>(0.0);
      p.vel = vec3<f32>({vx:.2}, {vy:.2}, {vz:.2}) + vec3<f32>(
        sin(f32(i) * 1.7 + params.time) * {spread:.2},
        cos(f32(i) * 2.3 + params.time) * {spread:.2},
        sin(f32(i) * 3.1 + params.time) * {spread:.2}
      );
      p.life = {lifetime:.2};
      p.maxLife = {lifetime:.2};
    }}
  }} else {{
    // Update living particle
    p.vel = p.vel + params.gravity * params.deltaTime;
    p.pos = p.pos + p.vel * params.deltaTime;
    p.life = p.life - params.deltaTime;
  }}

  particles[i] = p;
}}
"#,
        max = config.max_particles,
        vx = config.initial_velocity[0],
        vy = config.initial_velocity[1],
        vz = config.initial_velocity[2],
        spread = config.velocity_spread,
        lifetime = config.lifetime,
    )
}

/// Generate the WGSL vertex/fragment shader for particle rendering.
pub fn particle_render_shader(config: &ParticleConfig) -> String {
    format!(
        r#"// Particle Render Shader — billboarded quads
struct Particle {{
  pos: vec3<f32>,
  vel: vec3<f32>,
  life: f32,
  maxLife: f32,
}};

struct RenderUniforms {{
  viewProj: mat4x4<f32>,
  cameraRight: vec3<f32>,
  cameraUp: vec3<f32>,
}};

@group(0) @binding(0) var<storage, read> particles: array<Particle, {max}>;
@group(0) @binding(1) var<uniform> u: RenderUniforms;

struct VertexOutput {{
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) uv: vec2<f32>,
}};

@vertex fn vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) ii: u32) -> VertexOutput {{
  let p = particles[ii];
  var out: VertexOutput;

  if (p.life <= 0.0) {{
    out.position = vec4<f32>(0.0, 0.0, -10.0, 1.0); // cull dead particles
    return out;
  }}

  let t = 1.0 - (p.life / p.maxLife);
  let size = mix({size_start:.3}, {size_end:.3}, t);

  // Billboard quad corners
  let offsets = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, 1.0)
  );
  let corners = array<u32, 6>(0u, 1u, 2u, 2u, 1u, 3u);
  let corner = offsets[corners[vi]];

  let worldPos = p.pos + (u.cameraRight * corner.x + u.cameraUp * corner.y) * size;
  out.position = u.viewProj * vec4<f32>(worldPos, 1.0);

  // Color interpolation
  let startColor = vec4<f32>({cs0:.3}, {cs1:.3}, {cs2:.3}, {cs3:.3});
  let endColor = vec4<f32>({ce0:.3}, {ce1:.3}, {ce2:.3}, {ce3:.3});
  out.color = mix(startColor, endColor, t);
  out.uv = corner * 0.5 + 0.5;

  return out;
}}

@fragment fn fs(@location(0) color: vec4<f32>, @location(1) uv: vec2<f32>) -> @location(0) vec4<f32> {{
  // Soft circle falloff
  let dist = length(uv - vec2<f32>(0.5));
  let alpha = 1.0 - smoothstep(0.3, 0.5, dist);
  return vec4<f32>(color.rgb, color.a * alpha);
}}
"#,
        max = config.max_particles,
        size_start = config.size_start,
        size_end = config.size_end,
        cs0 = config.color_start[0],
        cs1 = config.color_start[1],
        cs2 = config.color_start[2],
        cs3 = config.color_start[3],
        ce0 = config.color_end[0],
        ce1 = config.color_end[1],
        ce2 = config.color_end[2],
        ce3 = config.color_end[3],
    )
}
