//! Scene ingestion — extracts 3D scene data from IR JSON.

use anyhow::{Context, Result};
use serde_json::Value;

use crate::scene::{Camera, Light, LightType, Mesh, MeshPrimitive, Scene};

/// Extract a Scene from IR JSON. Looks for Scene3D nodes in the tree.
pub fn ingest_scene(json: &str) -> Result<Scene> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR JSON")?;
    let mut scene = Scene::default();

    // Add default lighting if no lights specified
    scene.lights.push(Light {
        light_type: LightType::Directional,
        color: [1.0, 0.95, 0.9],
        intensity: 1.0,
        position: [5.0, 8.0, 5.0],
        direction: [-0.5, -1.0, -0.5],
    });
    scene.lights.push(Light {
        light_type: LightType::Ambient,
        color: [0.3, 0.3, 0.35],
        intensity: 0.4,
        ..Light::default()
    });

    // Walk the IR tree looking for Scene3D and MeshNode
    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            walk_children(children, &mut scene);
        }
    }

    // If no meshes found, add a default cube for demo purposes
    if scene.meshes.is_empty() {
        scene.meshes.push(Mesh {
            id: "default-cube".to_string(),
            primitive: MeshPrimitive::Cube,
            color: [0.8, 0.3, 0.2, 1.0],
            ..Default::default()
        });
    }

    Ok(scene)
}

fn walk_children(children: &[Value], scene: &mut Scene) {
    for child in children {
        let type_name = child
            .get("value_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let value = child.get("value").cloned().unwrap_or(Value::Null);

        match type_name {
            "Scene3D" => {
                // Extract camera settings
                if let Some(cam) = value.get("camera") {
                    scene.camera = extract_camera(cam);
                }
                // Extract background
                if let Some(bg) = value.get("background") {
                    scene.background_color = extract_color4(bg);
                }
                // Walk Scene3D children for meshes
                if let Some(sc) = value.get("children").and_then(|v| v.as_array()) {
                    walk_children(sc, scene);
                }
            }
            "MeshNode" => {
                scene.meshes.push(extract_mesh(&value));
            }
            _ => {
                // Recurse into other container types
                if let Some(gc) = value.get("children").and_then(|v| v.as_array()) {
                    walk_children(gc, scene);
                }
            }
        }
    }
}

fn extract_camera(cam: &Value) -> Camera {
    Camera {
        position: extract_vec3(cam.get("position"), [0.0, 2.0, 5.0]),
        target: extract_vec3(cam.get("target"), [0.0, 0.0, 0.0]),
        fov_degrees: cam.get("fov").and_then(|v| v.as_f64()).unwrap_or(45.0) as f32,
        orbit_enabled: cam.get("orbit").and_then(|v| v.as_bool()).unwrap_or(true),
        ..Default::default()
    }
}

fn extract_mesh(value: &Value) -> Mesh {
    let primitive = match value
        .get("primitive")
        .and_then(|v| v.as_str())
        .unwrap_or("cube")
    {
        "sphere" | "Sphere" => MeshPrimitive::Sphere,
        "plane" | "Plane" => MeshPrimitive::Plane,
        _ => MeshPrimitive::Cube,
    };

    Mesh {
        id: value
            .get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("mesh")
            .to_string(),
        primitive,
        position: extract_vec3(value.get("position"), [0.0, 0.0, 0.0]),
        rotation: extract_vec3(value.get("rotation"), [0.0, 0.0, 0.0]),
        scale: extract_vec3(value.get("scale"), [1.0, 1.0, 1.0]),
        color: extract_color4(value.get("color").unwrap_or(&Value::Null)),
    }
}

fn extract_vec3(val: Option<&Value>, default: [f32; 3]) -> [f32; 3] {
    val.map(|v| {
        [
            v.get("x")
                .and_then(|n| n.as_f64())
                .unwrap_or(default[0] as f64) as f32,
            v.get("y")
                .and_then(|n| n.as_f64())
                .unwrap_or(default[1] as f64) as f32,
            v.get("z")
                .and_then(|n| n.as_f64())
                .unwrap_or(default[2] as f64) as f32,
        ]
    })
    .unwrap_or(default)
}

fn extract_color4(val: &Value) -> [f32; 4] {
    if val.is_null() {
        return [0.8, 0.3, 0.2, 1.0];
    }
    [
        val.get("r").and_then(|n| n.as_f64()).unwrap_or(200.0) as f32 / 255.0,
        val.get("g").and_then(|n| n.as_f64()).unwrap_or(80.0) as f32 / 255.0,
        val.get("b").and_then(|n| n.as_f64()).unwrap_or(60.0) as f32 / 255.0,
        val.get("a").and_then(|n| n.as_f64()).unwrap_or(255.0) as f32 / 255.0,
    ]
}
