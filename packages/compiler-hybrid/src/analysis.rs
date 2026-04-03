//! Target analysis — determines the optimal compile target for each IR node.
//!
//! Heuristics:
//! - Scene3D, MeshNode, ParticleSystem, ShaderNode → WebGPU (if available)
//! - StateMachine with >10 states or ComputeNode with complex expression → WASM
//! - Everything else → DOM

use anyhow::{Context, Result};
use serde_json::Value;

use crate::device::DeviceProfile;
use crate::CompileTarget;

/// A node with its assigned compile target.
#[derive(Debug, Clone)]
pub struct NodeTargetAssignment {
    pub node_id: String,
    pub node_type: String,
    pub target: CompileTarget,
    pub reason: String,
}

/// Analyze all nodes and assign compile targets.
pub fn analyze_targets(
    json: &str,
    device: &DeviceProfile,
    force: Option<CompileTarget>,
) -> Result<Vec<NodeTargetAssignment>> {
    let doc: Value = serde_json::from_str(json).context("Failed to parse IR")?;
    let mut assignments = Vec::new();

    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            walk_and_assign(children, device, force, &mut assignments);
        }
    }

    Ok(assignments)
}

fn walk_and_assign(
    children: &[Value],
    device: &DeviceProfile,
    force: Option<CompileTarget>,
    assignments: &mut Vec<NodeTargetAssignment>,
) {
    for child in children {
        let type_name = child
            .get("value_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let value = child.get("value").cloned().unwrap_or(Value::Null);
        let node_id = value
            .get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let (target, reason) = if let Some(forced) = force {
            (forced, "forced by compile option".to_string())
        } else {
            select_target(&type_name, &value, device)
        };

        assignments.push(NodeTargetAssignment {
            node_id,
            node_type: type_name,
            target,
            reason,
        });

        // Recurse
        if let Some(gc) = value.get("children").and_then(|v| v.as_array()) {
            walk_and_assign(gc, device, force, assignments);
        }
    }
}

/// Select the optimal compile target for a single node.
fn select_target(
    type_name: &str,
    value: &Value,
    device: &DeviceProfile,
) -> (CompileTarget, String) {
    match type_name {
        // 3D nodes → WebGPU (if available)
        "Scene3D" | "MeshNode" | "ShaderNode" | "ParticleSystem" => {
            if device.has_webgpu {
                (CompileTarget::WebGpu, "3D node, WebGPU available".to_string())
            } else {
                (CompileTarget::Dom, "3D node, WebGPU not available — Canvas 2D fallback".to_string())
            }
        }

        // Complex state machines → WASM
        "StateMachine" => {
            let state_count = value
                .get("states")
                .and_then(|v| v.as_array())
                .map_or(0, |a| a.len());
            let transition_count = value
                .get("transitions")
                .and_then(|v| v.as_array())
                .map_or(0, |a| a.len());

            if state_count > 10 || transition_count > 20 {
                (CompileTarget::Wasm, format!("complex state machine ({state_count} states, {transition_count} transitions)"))
            } else {
                (CompileTarget::Dom, format!("simple state machine ({state_count} states) — JS is sufficient"))
            }
        }

        // Compute nodes → WASM if expression is complex
        "ComputeNode" => {
            let expr = value
                .get("expression")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let is_complex = expr.len() > 50 || expr.matches('+').count() + expr.matches('*').count() > 3;

            if is_complex {
                (CompileTarget::Wasm, "complex compute expression".to_string())
            } else {
                (CompileTarget::Dom, "simple compute — inline JS".to_string())
            }
        }

        // Everything else → DOM
        _ => (CompileTarget::Dom, "layout/visual node".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::DeviceProfile;

    #[test]
    fn scene3d_targets_webgpu_on_desktop() {
        let (target, _) = select_target("Scene3D", &Value::Null, &DeviceProfile::desktop());
        assert_eq!(target, CompileTarget::WebGpu);
    }

    #[test]
    fn scene3d_falls_back_to_dom_on_mobile_low() {
        let (target, _) = select_target("Scene3D", &Value::Null, &DeviceProfile::mobile_low());
        assert_eq!(target, CompileTarget::Dom);
    }

    #[test]
    fn simple_state_machine_stays_dom() {
        let sm = serde_json::json!({
            "states": [{"name": "a"}, {"name": "b"}],
            "transitions": [{"from": "a", "to": "b", "event": "click"}]
        });
        let (target, _) = select_target("StateMachine", &sm, &DeviceProfile::desktop());
        assert_eq!(target, CompileTarget::Dom);
    }

    #[test]
    fn complex_state_machine_targets_wasm() {
        let states: Vec<Value> = (0..15)
            .map(|i| serde_json::json!({"name": format!("s{i}")}))
            .collect();
        let sm = serde_json::json!({ "states": states, "transitions": [] });
        let (target, _) = select_target("StateMachine", &sm, &DeviceProfile::desktop());
        assert_eq!(target, CompileTarget::Wasm);
    }

    #[test]
    fn container_always_dom() {
        let (target, _) = select_target("Container", &Value::Null, &DeviceProfile::desktop());
        assert_eq!(target, CompileTarget::Dom);
    }
}
