//! S79 A3 — `voce graph`: exported IR semantic graph.
//!
//! The differentiator-half of the agent contract. A general-purpose
//! language toolchain (Zero) can export a module dependency graph;
//! Voce can export the actual *UI semantic graph* — composition,
//! typed reference edges, and state-machine reachability — because
//! the IR encodes UI meaning. This is the data agents need to reason
//! about reachability, dangling refs, and unbound interactivity
//! without re-walking the IR or re-deriving the index.
//!
//! Reuses the existing `NodeIndex` for reference resolution so the
//! "is this id valid" answer here matches what the validator's
//! reference pass (`REF*`) sees — single source of truth.

use std::collections::{BTreeSet, VecDeque};

use serde::Serialize;

use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};

/// One node in the composition tree.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GraphNode {
    pub id: String,
    pub type_name: String,
    pub path: String,
    pub parent_id: Option<String>,
}

/// Composition tree edge — emitted explicitly so a graph consumer
/// doesn't have to derive it by re-traversing `parent_id`.
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct CompositionEdge {
    pub parent: String,
    pub child: String,
}

/// Typed reference edge. `to_resolved` mirrors the validator's
/// reference-pass result (`true` ⇒ target exists in the document).
#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct ReferenceEdge {
    pub from: String,
    pub from_path: String,
    pub kind: ReferenceKind,
    pub to: String,
    pub to_resolved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceKind {
    Semantic,
    AnimationTarget,
    GestureTarget,
    ScrollTarget,
    PhysicsTarget,
    LiveRegionTarget,
    FocusTrapContainer,
    FocusTrapInitialFocus,
    SubscriptionTarget,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StateNode {
    pub name: String,
    pub initial: bool,
    pub terminal: bool,
    pub reachable: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StateTransition {
    pub event: Option<String>,
    pub from: String,
    pub to: String,
    /// Both endpoints reference declared states.
    pub valid: bool,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct StateMachineGraph {
    pub node_id: Option<String>,
    pub name: Option<String>,
    pub initial_state: Option<String>,
    pub states: Vec<StateNode>,
    pub transitions: Vec<StateTransition>,
    pub unreachable_states: Vec<String>,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GraphSummary {
    pub node_count: usize,
    pub composition_edge_count: usize,
    pub reference_edge_count: usize,
    pub dangling_reference_count: usize,
    pub state_machine_count: usize,
    pub unreachable_state_count: usize,
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct GraphExport {
    pub contract_version: &'static str,
    pub nodes: Vec<GraphNode>,
    pub composition_edges: Vec<CompositionEdge>,
    pub reference_edges: Vec<ReferenceEdge>,
    pub state_machines: Vec<StateMachineGraph>,
    pub summary: GraphSummary,
}

/// Build the graph export. Reuses `NodeIndex` for reference resolution
/// so the validator's reference-pass verdict and this export agree.
pub fn build(ir: &VoceIr, index: &NodeIndex) -> GraphExport {
    let mut nodes = Vec::new();
    let mut composition_edges = Vec::new();
    let mut reference_edges = Vec::new();
    let mut state_machines = Vec::new();

    if let Some(root) = &ir.root {
        if let Some(root_id) = &root.node_id {
            nodes.push(GraphNode {
                id: root_id.clone(),
                type_name: "ViewRoot".to_string(),
                path: "/root".to_string(),
                parent_id: None,
            });
        }

        // SemanticNodes live at root.semantic_nodes (not in children tree)
        // — surface them as graph nodes so reference edges can target them
        // and the consumer sees them.
        if let Some(sems) = &root.semantic_nodes {
            for (i, sem) in sems.iter().enumerate() {
                if let Some(id) = &sem.node_id {
                    nodes.push(GraphNode {
                        id: id.clone(),
                        type_name: "SemanticNode".to_string(),
                        path: format!("/root/semantic_nodes/{i}"),
                        parent_id: root.node_id.clone(),
                    });
                    if let Some(parent) = &root.node_id {
                        composition_edges.push(CompositionEdge {
                            parent: parent.clone(),
                            child: id.clone(),
                        });
                    }
                }
            }
        }

        if let Some(children) = &root.children {
            walk(
                children,
                "/root/children",
                root.node_id.as_deref(),
                index,
                &mut nodes,
                &mut composition_edges,
                &mut reference_edges,
                &mut state_machines,
            );
        }
    }

    let dangling = reference_edges.iter().filter(|e| !e.to_resolved).count();
    let unreachable: usize = state_machines.iter().map(|sm| sm.unreachable_states.len()).sum();

    GraphExport {
        summary: GraphSummary {
            node_count: nodes.len(),
            composition_edge_count: composition_edges.len(),
            reference_edge_count: reference_edges.len(),
            dangling_reference_count: dangling,
            state_machine_count: state_machines.len(),
            unreachable_state_count: unreachable,
        },
        contract_version: crate::skills::CONTRACT_VERSION,
        nodes,
        composition_edges,
        reference_edges,
        state_machines,
    }
}

#[allow(clippy::too_many_arguments)]
fn walk(
    children: &[ChildNode],
    parent_path: &str,
    parent_id: Option<&str>,
    index: &NodeIndex,
    nodes: &mut Vec<GraphNode>,
    comp: &mut Vec<CompositionEdge>,
    refs: &mut Vec<ReferenceEdge>,
    state_machines: &mut Vec<StateMachineGraph>,
) {
    for (i, child) in children.iter().enumerate() {
        let path = format!("{parent_path}/{i}");
        let Some(id) = child.node_id() else {
            continue;
        };
        let type_name = child.type_name().to_string();

        nodes.push(GraphNode {
            id: id.clone(),
            type_name: type_name.clone(),
            path: path.clone(),
            parent_id: parent_id.map(String::from),
        });
        if let Some(p) = parent_id {
            comp.push(CompositionEdge {
                parent: p.to_string(),
                child: id.clone(),
            });
        }

        // Semantic reference is universal across visual node types.
        if let Some(sem) = child.semantic_node_id() {
            if !sem.is_empty() {
                refs.push(ReferenceEdge {
                    from: id.clone(),
                    from_path: path.clone(),
                    kind: ReferenceKind::Semantic,
                    to_resolved: index.contains(&sem),
                    to: sem,
                });
            }
        }

        // Type-specific edges. The set mirrors the validator's
        // references pass so the resolved/dangling verdict matches.
        match type_name.as_str() {
            "AnimationTransition" => {
                if let Some(n) = child.as_type::<crate::ir::AnimationTransition>() {
                    push_target(refs, &id, &path, ReferenceKind::AnimationTarget, &n.target_node_id, index);
                }
            }
            "GestureHandler" => {
                if let Some(n) = child.as_type::<crate::ir::GestureHandler>() {
                    push_target(refs, &id, &path, ReferenceKind::GestureTarget, &n.target_node_id, index);
                }
            }
            "ScrollBinding" => {
                if let Some(n) = child.as_type::<crate::ir::ScrollBinding>() {
                    push_target(refs, &id, &path, ReferenceKind::ScrollTarget, &n.target_node_id, index);
                }
            }
            "PhysicsBody" => {
                if let Some(n) = child.as_type::<crate::ir::PhysicsBody>() {
                    push_target(refs, &id, &path, ReferenceKind::PhysicsTarget, &n.target_node_id, index);
                }
            }
            "LiveRegion" => {
                if let Some(n) = child.as_type::<crate::ir::LiveRegion>() {
                    push_target(refs, &id, &path, ReferenceKind::LiveRegionTarget, &n.target_node_id, index);
                }
            }
            "FocusTrap" => {
                if let Some(n) = child.as_type::<crate::ir::FocusTrap>() {
                    push_target(refs, &id, &path, ReferenceKind::FocusTrapContainer, &n.container_node_id, index);
                    push_target(refs, &id, &path, ReferenceKind::FocusTrapInitialFocus, &n.initial_focus_node_id, index);
                }
            }
            "SubscriptionNode" => {
                if let Some(n) = child.as_type::<crate::ir::SubscriptionNode>() {
                    push_target(refs, &id, &path, ReferenceKind::SubscriptionTarget, &n.target_data_node_id, index);
                }
            }
            "StateMachine" => {
                if let Some(sm) = child.as_type::<crate::ir::StateMachine>() {
                    state_machines.push(build_state_machine(&id, &sm));
                }
            }
            _ => {}
        }

        if let Some(grandchildren) = child.children() {
            walk(
                &grandchildren,
                &format!("{path}/children"),
                Some(&id),
                index,
                nodes,
                comp,
                refs,
                state_machines,
            );
        }
    }
}

fn push_target(
    refs: &mut Vec<ReferenceEdge>,
    from: &str,
    from_path: &str,
    kind: ReferenceKind,
    target: &Option<String>,
    index: &NodeIndex,
) {
    if let Some(t) = target {
        if !t.is_empty() {
            refs.push(ReferenceEdge {
                from: from.to_string(),
                from_path: from_path.to_string(),
                kind,
                to_resolved: index.contains(t),
                to: t.clone(),
            });
        }
    }
}

fn build_state_machine(node_id: &str, sm: &crate::ir::StateMachine) -> StateMachineGraph {
    let states_decl = sm.states.as_deref().unwrap_or_default();
    let transitions_decl = sm.transitions.as_deref().unwrap_or_default();

    // Set of declared state names — what reachability and edge validity
    // are computed against.
    let declared: BTreeSet<&str> = states_decl
        .iter()
        .filter_map(|s| s.name.as_deref())
        .collect();

    // Reachability via BFS from the (first) initial state.
    let initial = states_decl
        .iter()
        .find(|s| s.initial)
        .and_then(|s| s.name.as_deref());

    let mut reachable: BTreeSet<&str> = BTreeSet::new();
    if let Some(start) = initial {
        let mut queue: VecDeque<&str> = VecDeque::from([start]);
        reachable.insert(start);
        while let Some(cur) = queue.pop_front() {
            for t in transitions_decl {
                if t.from.as_deref() == Some(cur) {
                    if let Some(to) = t.to.as_deref() {
                        if declared.contains(to) && reachable.insert(to) {
                            queue.push_back(to);
                        }
                    }
                }
            }
        }
    }

    let states: Vec<StateNode> = states_decl
        .iter()
        .filter_map(|s| {
            s.name.as_ref().map(|n| StateNode {
                name: n.clone(),
                initial: s.initial,
                terminal: s.terminal,
                reachable: reachable.contains(n.as_str()),
            })
        })
        .collect();

    let transitions: Vec<StateTransition> = transitions_decl
        .iter()
        .filter_map(|t| {
            let from = t.from.clone()?;
            let to = t.to.clone()?;
            let valid =
                declared.contains(from.as_str()) && declared.contains(to.as_str());
            Some(StateTransition {
                event: t.event.clone(),
                from,
                to,
                valid,
            })
        })
        .collect();

    let unreachable_states: Vec<String> = states
        .iter()
        .filter(|s| !s.reachable)
        .map(|s| s.name.clone())
        .collect();

    StateMachineGraph {
        node_id: Some(node_id.to_string()),
        name: sm.name.clone(),
        initial_state: initial.map(String::from),
        states,
        transitions,
        unreachable_states,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> (VoceIr, NodeIndex) {
        let ir: VoceIr = serde_json::from_str(json).expect("parse");
        let idx = NodeIndex::build(&ir);
        (ir, idx)
    }

    #[test]
    fn empty_doc_yields_empty_graph() {
        let (ir, idx) = parse(r#"{ "root": { "node_id": "r" } }"#);
        let g = build(&ir, &idx);
        assert_eq!(g.summary.node_count, 1); // ViewRoot itself
        assert!(g.composition_edges.is_empty());
        assert!(g.reference_edges.is_empty());
        assert!(g.state_machines.is_empty());
    }

    #[test]
    fn semantic_reference_classified_resolved_or_dangling() {
        let resolved = r#"{ "root": { "node_id": "r",
            "semantic_nodes": [{ "node_id": "s", "role": "button", "label": "Go" }],
            "children": [{ "value_type": "Surface", "value": { "node_id": "btn", "semantic_node_id": "s" } }]
        } }"#;
        let (ir, idx) = parse(resolved);
        let g = build(&ir, &idx);
        let semref = g
            .reference_edges
            .iter()
            .find(|e| e.kind == ReferenceKind::Semantic)
            .expect("semantic edge");
        assert!(semref.to_resolved);
        assert_eq!(semref.from, "btn");
        assert_eq!(semref.to, "s");
        assert_eq!(g.summary.dangling_reference_count, 0);

        let dangling = r#"{ "root": { "node_id": "r",
            "children": [{ "value_type": "Surface", "value": { "node_id": "btn", "semantic_node_id": "missing" } }]
        } }"#;
        let (ir, idx) = parse(dangling);
        let g = build(&ir, &idx);
        assert_eq!(g.summary.dangling_reference_count, 1);
    }

    #[test]
    fn state_machine_reachability_flags_unreachable() {
        let json = r#"{ "root": { "node_id": "r", "children": [{
            "value_type": "StateMachine",
            "value": {
              "node_id": "sm",
              "name": "checkout",
              "states": [
                { "name": "idle",   "initial": true },
                { "name": "loading" },
                { "name": "orphan" }
              ],
              "transitions": [
                { "event": "submit", "from": "idle",    "to": "loading" }
              ]
            }
        }] } }"#;
        let (ir, idx) = parse(json);
        let g = build(&ir, &idx);
        let sm = &g.state_machines[0];
        assert_eq!(sm.initial_state.as_deref(), Some("idle"));
        assert!(sm.states.iter().find(|s| s.name == "idle").unwrap().reachable);
        assert!(sm.states.iter().find(|s| s.name == "loading").unwrap().reachable);
        assert!(!sm.states.iter().find(|s| s.name == "orphan").unwrap().reachable);
        assert_eq!(sm.unreachable_states, vec!["orphan".to_string()]);
        assert!(sm.transitions[0].valid);
    }

    #[test]
    fn composition_edges_track_parent_child_nesting() {
        let json = r#"{ "root": { "node_id": "r", "children": [
            { "value_type": "Container", "value": { "node_id": "outer", "children": [
                { "value_type": "TextNode", "value": { "node_id": "t", "content": "hi" } }
            ] } }
        ] } }"#;
        let (ir, idx) = parse(json);
        let g = build(&ir, &idx);
        assert!(g.composition_edges.iter().any(|e| e.parent == "r" && e.child == "outer"));
        assert!(g.composition_edges.iter().any(|e| e.parent == "outer" && e.child == "t"));
    }

    #[test]
    fn graph_serializes_with_contract_version() {
        let (ir, idx) = parse(r#"{ "root": { "node_id": "r" } }"#);
        let g = build(&ir, &idx);
        let v = serde_json::to_value(&g).unwrap();
        assert_eq!(v["contract_version"], crate::skills::CONTRACT_VERSION);
        assert!(v["summary"]["node_count"].is_number());
    }
}
