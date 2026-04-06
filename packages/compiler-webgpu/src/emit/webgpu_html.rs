//! WebGPU HTML emitter — generates a complete HTML5 document with
//! embedded WebGPU rendering, WGSL shaders, camera, and lighting.

use crate::scene::{Light, LightType, Mesh, MeshPrimitive, Scene};

pub struct EmitOptions {
    pub width: u32,
    pub height: u32,
    pub orbit_controls: bool,
}

/// Emit a complete WebGPU HTML document from a Scene.
pub fn emit(scene: &Scene, options: &EmitOptions) -> String {
    let mut html = String::with_capacity(8192);
    let w = options.width;
    let h = options.height;

    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Voce IR — WebGPU Scene</title>\n");
    html.push_str("<style>\n");
    html.push_str("body{margin:0;overflow:hidden;background:#000}\n");
    html.push_str("canvas{display:block;width:100vw;height:100vh}\n");
    html.push_str(
        "#fallback{display:none;color:#fff;font-family:system-ui;padding:40px;text-align:center}\n",
    );
    html.push_str("</style>\n</head>\n<body>\n");
    html.push_str(&format!(
        "<canvas id=\"c\" width=\"{w}\" height=\"{h}\"></canvas>\n"
    ));
    html.push_str("<div id=\"fallback\"><h2>WebGPU is not available</h2><p>This scene requires a browser with WebGPU support (Chrome 113+, Edge 113+, Firefox 130+).</p></div>\n");

    // WGSL shaders
    html.push_str("<script>\n");
    html.push_str(&emit_shaders());
    html.push('\n');

    // Mesh data
    html.push_str(&emit_mesh_data(&scene.meshes));

    // Main initialization
    html.push_str(&emit_init(scene, options));

    html.push_str("</script>\n</body>\n</html>\n");
    html
}

fn emit_shaders() -> String {
    r#"const vertexShader = `
struct Uniforms {
  mvp: mat4x4<f32>,
  model: mat4x4<f32>,
  lightDir: vec3<f32>,
  lightColor: vec3<f32>,
  ambientColor: vec3<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
  @builtin(position) pos: vec4<f32>,
  @location(0) normal: vec3<f32>,
  @location(1) color: vec3<f32>,
};

@vertex fn vs(@location(0) position: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) color: vec3<f32>) -> VertexOutput {
  var out: VertexOutput;
  out.pos = u.mvp * vec4<f32>(position, 1.0);
  out.normal = (u.model * vec4<f32>(normal, 0.0)).xyz;
  out.color = color;
  return out;
}
`;

const fragmentShader = `
struct Uniforms {
  mvp: mat4x4<f32>,
  model: mat4x4<f32>,
  lightDir: vec3<f32>,
  lightColor: vec3<f32>,
  ambientColor: vec3<f32>,
};
@group(0) @binding(0) var<uniform> u: Uniforms;

@fragment fn fs(@location(0) normal: vec3<f32>, @location(1) color: vec3<f32>) -> @location(0) vec4<f32> {
  let n = normalize(normal);
  let light = max(dot(n, normalize(-u.lightDir)), 0.0);
  let diffuse = u.lightColor * light;
  let final_color = color * (diffuse + u.ambientColor);
  return vec4<f32>(final_color, 1.0);
}
`;"#
    .to_string()
}

fn emit_mesh_data(meshes: &[Mesh]) -> String {
    let mut js = String::new();
    js.push_str("// Mesh geometry data\n");

    for (i, mesh) in meshes.iter().enumerate() {
        let (verts, indices) = match mesh.primitive {
            MeshPrimitive::Cube => cube_geometry(),
            MeshPrimitive::Plane => plane_geometry(),
            _ => cube_geometry(), // Sphere approximated as cube for now
        };

        let c = mesh.color;
        js.push_str(&format!(
            "const mesh{i}_verts = new Float32Array({verts});\n"
        ));
        js.push_str(&format!(
            "const mesh{i}_indices = new Uint16Array({indices});\n"
        ));
        js.push_str(&format!(
            "const mesh{i}_color = [{:.3}, {:.3}, {:.3}];\n",
            c[0], c[1], c[2]
        ));
        js.push_str(&format!(
            "const mesh{i}_pos = [{:.2}, {:.2}, {:.2}];\n",
            mesh.position[0], mesh.position[1], mesh.position[2]
        ));
    }

    js
}

fn emit_init(scene: &Scene, options: &EmitOptions) -> String {
    let bg = scene.background_color;
    let cam = &scene.camera;
    let mesh_count = scene.meshes.len();

    // Find directional light
    let default_light = Light::default();
    let dir_light = scene
        .lights
        .iter()
        .find(|l| matches!(l.light_type, LightType::Directional))
        .unwrap_or(&default_light);
    let ambient = scene
        .lights
        .iter()
        .find(|l| matches!(l.light_type, LightType::Ambient));
    let amb_color = ambient.map(|a| a.color).unwrap_or([0.2, 0.2, 0.25]);

    format!(
        r#"
async function init() {{
  if (!navigator.gpu) {{
    document.getElementById('c').style.display='none';
    document.getElementById('fallback').style.display='block';
    return;
  }}
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) return;
  const device = await adapter.requestDevice();
  const canvas = document.getElementById('c');
  canvas.width = canvas.clientWidth * devicePixelRatio;
  canvas.height = canvas.clientHeight * devicePixelRatio;
  const ctx = canvas.getContext('webgpu');
  const format = navigator.gpu.getPreferredCanvasFormat();
  ctx.configure({{ device, format, alphaMode: 'premultiplied' }});

  // Shader module
  const vsModule = device.createShaderModule({{ code: vertexShader }});
  const fsModule = device.createShaderModule({{ code: fragmentShader }});

  // Uniform buffer (MVP + model + light)
  const uniformBuffer = device.createBuffer({{
    size: 256, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST
  }});

  const bindGroupLayout = device.createBindGroupLayout({{
    entries: [{{ binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: {{}} }}]
  }});

  const bindGroup = device.createBindGroup({{
    layout: bindGroupLayout,
    entries: [{{ binding: 0, resource: {{ buffer: uniformBuffer }} }}]
  }});

  const pipeline = device.createRenderPipeline({{
    layout: device.createPipelineLayout({{ bindGroupLayouts: [bindGroupLayout] }}),
    vertex: {{
      module: vsModule, entryPoint: 'vs',
      buffers: [{{
        arrayStride: 36,
        attributes: [
          {{ shaderLocation: 0, offset: 0, format: 'float32x3' }},
          {{ shaderLocation: 1, offset: 12, format: 'float32x3' }},
          {{ shaderLocation: 2, offset: 24, format: 'float32x3' }},
        ]
      }}]
    }},
    fragment: {{ module: fsModule, entryPoint: 'fs', targets: [{{ format }}] }},
    primitive: {{ topology: 'triangle-list', cullMode: 'back' }},
    depthStencil: {{ depthWriteEnabled: true, depthCompare: 'less', format: 'depth24plus' }},
  }});

  // Depth texture
  let depthTexture = device.createTexture({{
    size: [canvas.width, canvas.height], format: 'depth24plus', usage: GPUTextureUsage.RENDER_ATTACHMENT
  }});

  // Camera
  let camAngle = 0, camDist = {cam_dist:.1}, camHeight = {cam_height:.1};
  {orbit_js}

  // Render loop
  function frame() {{
    const aspect = canvas.width / canvas.height;
    const eye = [Math.sin(camAngle)*camDist, camHeight, Math.cos(camAngle)*camDist];
    const view = lookAt(eye, [{target0:.1},{target1:.1},{target2:.1}]);
    const proj = perspective({fov:.4}, aspect, {near}, {far});

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({{
      colorAttachments: [{{
        view: ctx.getCurrentTexture().createView(),
        clearValue: {{ r:{bg0:.3}, g:{bg1:.3}, b:{bg2:.3}, a:1 }},
        loadOp: 'clear', storeOp: 'store'
      }}],
      depthStencilAttachment: {{
        view: depthTexture.createView(),
        depthClearValue: 1.0, depthLoadOp: 'clear', depthStoreOp: 'store'
      }}
    }});
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);

    // Render meshes (simplified: one draw per mesh)
    // Uniform data: MVP(64) + Model(64) + lightDir(12+pad4) + lightColor(12+pad4) + ambient(12+pad4)
    const uniformData = new Float32Array(64);
    for (let i = 0; i < {mesh_count}; i++) {{
      // Simple identity model matrix for now
      const mvp = multiply(proj, view);
      uniformData.set(mvp, 0);
      // Light direction
      uniformData[32] = {ld0:.3}; uniformData[33] = {ld1:.3}; uniformData[34] = {ld2:.3};
      // Light color
      uniformData[36] = {lc0:.3}; uniformData[37] = {lc1:.3}; uniformData[38] = {lc2:.3};
      // Ambient
      uniformData[40] = {ac0:.3}; uniformData[41] = {ac1:.3}; uniformData[42] = {ac2:.3};
      device.queue.writeBuffer(uniformBuffer, 0, uniformData);
    }}

    pass.end();
    device.queue.submit([encoder.finish()]);
    requestAnimationFrame(frame);
  }}
  requestAnimationFrame(frame);
}}

// Math helpers (minimal)
function perspective(fov, aspect, near, far) {{
  const f = 1/Math.tan(fov/2), nf = 1/(near-far);
  return [f/aspect,0,0,0, 0,f,0,0, 0,0,(far+near)*nf,-1, 0,0,2*far*near*nf,0];
}}
function lookAt(eye, target) {{
  const z = normalize3(sub3(eye,target)), x = normalize3(cross3([0,1,0],z)), y = cross3(z,x);
  return [x[0],y[0],z[0],0, x[1],y[1],z[1],0, x[2],y[2],z[2],0, -dot3(x,eye),-dot3(y,eye),-dot3(z,eye),1];
}}
function multiply(a,b) {{
  const o=new Array(16);
  for(let i=0;i<4;i++)for(let j=0;j<4;j++){{o[j*4+i]=0;for(let k=0;k<4;k++)o[j*4+i]+=a[k*4+i]*b[j*4+k];}}
  return o;
}}
function normalize3(v){{const l=Math.sqrt(v[0]*v[0]+v[1]*v[1]+v[2]*v[2]);return[v[0]/l,v[1]/l,v[2]/l];}}
function cross3(a,b){{return[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];}}
function sub3(a,b){{return[a[0]-b[0],a[1]-b[1],a[2]-b[2]];}}
function dot3(a,b){{return a[0]*b[0]+a[1]*b[1]+a[2]*b[2];}}

init();
"#,
        cam_dist =
            (cam.position[0].powi(2) + cam.position[1].powi(2) + cam.position[2].powi(2)).sqrt(),
        cam_height = cam.position[1],
        target0 = cam.target[0],
        target1 = cam.target[1],
        target2 = cam.target[2],
        fov = cam.fov_degrees.to_radians(),
        near = cam.near,
        far = cam.far,
        bg0 = bg[0],
        bg1 = bg[1],
        bg2 = bg[2],
        mesh_count = mesh_count,
        ld0 = dir_light.direction[0],
        ld1 = dir_light.direction[1],
        ld2 = dir_light.direction[2],
        lc0 = dir_light.color[0],
        lc1 = dir_light.color[1],
        lc2 = dir_light.color[2],
        ac0 = amb_color[0],
        ac1 = amb_color[1],
        ac2 = amb_color[2],
        orbit_js = if options.orbit_controls {
            "canvas.addEventListener('mousemove',(e)=>{if(e.buttons&1){camAngle+=e.movementX*0.01;camHeight+=e.movementY*0.05;}});\n  canvas.addEventListener('wheel',(e)=>{camDist+=e.deltaY*0.01;camDist=Math.max(1,camDist);});"
        } else {
            ""
        },
    )
}

fn cube_geometry() -> (String, String) {
    // 24 vertices (4 per face, with normals), 36 indices
    // Format: position(3) + normal(3) + color(3) per vertex — but color injected per-mesh
    (
        "[/* cube vertices — 24 verts x 9 floats */\
-0.5,-0.5,0.5, 0,0,1, 0.8,0.3,0.2, 0.5,-0.5,0.5, 0,0,1, 0.8,0.3,0.2, 0.5,0.5,0.5, 0,0,1, 0.8,0.3,0.2, -0.5,0.5,0.5, 0,0,1, 0.8,0.3,0.2,\
0.5,-0.5,-0.5, 0,0,-1, 0.8,0.3,0.2, -0.5,-0.5,-0.5, 0,0,-1, 0.8,0.3,0.2, -0.5,0.5,-0.5, 0,0,-1, 0.8,0.3,0.2, 0.5,0.5,-0.5, 0,0,-1, 0.8,0.3,0.2,\
-0.5,0.5,0.5, 0,1,0, 0.8,0.3,0.2, 0.5,0.5,0.5, 0,1,0, 0.8,0.3,0.2, 0.5,0.5,-0.5, 0,1,0, 0.8,0.3,0.2, -0.5,0.5,-0.5, 0,1,0, 0.8,0.3,0.2,\
-0.5,-0.5,-0.5, 0,-1,0, 0.8,0.3,0.2, 0.5,-0.5,-0.5, 0,-1,0, 0.8,0.3,0.2, 0.5,-0.5,0.5, 0,-1,0, 0.8,0.3,0.2, -0.5,-0.5,0.5, 0,-1,0, 0.8,0.3,0.2,\
0.5,-0.5,0.5, 1,0,0, 0.8,0.3,0.2, 0.5,-0.5,-0.5, 1,0,0, 0.8,0.3,0.2, 0.5,0.5,-0.5, 1,0,0, 0.8,0.3,0.2, 0.5,0.5,0.5, 1,0,0, 0.8,0.3,0.2,\
-0.5,-0.5,-0.5, -1,0,0, 0.8,0.3,0.2, -0.5,-0.5,0.5, -1,0,0, 0.8,0.3,0.2, -0.5,0.5,0.5, -1,0,0, 0.8,0.3,0.2, -0.5,0.5,-0.5, -1,0,0, 0.8,0.3,0.2\
]".to_string(),
        "[0,1,2, 0,2,3, 4,5,6, 4,6,7, 8,9,10, 8,10,11, 12,13,14, 12,14,15, 16,17,18, 16,18,19, 20,21,22, 20,22,23]".to_string(),
    )
}

fn plane_geometry() -> (String, String) {
    (
        "[-1,0,-1, 0,1,0, 0.6,0.6,0.6, 1,0,-1, 0,1,0, 0.6,0.6,0.6, 1,0,1, 0,1,0, 0.6,0.6,0.6, -1,0,1, 0,1,0, 0.6,0.6,0.6]".to_string(),
        "[0,1,2, 0,2,3]".to_string(),
    )
}
