//! Scene graph types for the WebGPU compiler.
//!
//! These are the compiler's internal representation of 3D scenes,
//! extracted from the IR during ingestion.

/// A 3D scene extracted from the IR.
#[derive(Debug, Default)]
pub struct Scene {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub meshes: Vec<Mesh>,
    pub background_color: [f32; 4],
}

/// Camera configuration.
#[derive(Debug)]
pub struct Camera {
    pub projection: Projection,
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub fov_degrees: f32,
    pub near: f32,
    pub far: f32,
    pub orbit_enabled: bool,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::Perspective,
            position: [0.0, 2.0, 5.0],
            target: [0.0, 0.0, 0.0],
            fov_degrees: 45.0,
            near: 0.1,
            far: 100.0,
            orbit_enabled: true,
        }
    }
}

#[derive(Debug, Default)]
pub enum Projection {
    #[default]
    Perspective,
    Orthographic,
}

/// A light in the scene.
#[derive(Debug)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub intensity: f32,
    pub position: [f32; 3],
    pub direction: [f32; 3],
}

#[derive(Debug)]
pub enum LightType {
    Directional,
    Point,
    Ambient,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            position: [5.0, 5.0, 5.0],
            direction: [-0.5, -1.0, -0.5],
        }
    }
}

/// A mesh to render.
#[derive(Debug)]
pub struct Mesh {
    pub id: String,
    pub primitive: MeshPrimitive,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Debug, Default)]
pub enum MeshPrimitive {
    #[default]
    Cube,
    Sphere,
    Plane,
    Custom,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            id: String::new(),
            primitive: MeshPrimitive::Cube,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            color: [0.8, 0.2, 0.2, 1.0],
        }
    }
}
