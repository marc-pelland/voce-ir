//! Node index — fast lookup structure built from the deserialized IR tree.
//!
//! Built by a single recursive walk of the IR after deserialization.
//! Provides O(1) lookup by node ID and node type filtering.

use std::collections::{HashMap, HashSet};

use crate::ir::{ChildNode, VoceIr};

/// Entry for a single node in the index.
#[derive(Debug, Clone)]
pub struct NodeEntry {
    /// JSON-pointer-style path (e.g., "/root/children/0").
    pub path: String,
    /// Node type name (e.g., "Container", "TextNode").
    pub type_name: String,
    /// Parent node ID (None for ViewRoot).
    pub parent_id: Option<String>,
}

/// Fast lookup index over the IR tree.
#[derive(Debug, Default)]
pub struct NodeIndex {
    /// node_id -> NodeEntry
    nodes: HashMap<String, NodeEntry>,
    /// Duplicate node_ids found during indexing.
    pub duplicates: Vec<(String, String, String)>, // (id, path1, path2)
    /// All node IDs of a given type.
    by_type: HashMap<String, Vec<String>>,
}

impl NodeIndex {
    /// Build the index from a deserialized IR document.
    pub fn build(ir: &VoceIr) -> Self {
        let mut index = NodeIndex::default();

        if let Some(ref root) = ir.root {
            if let Some(ref id) = root.node_id {
                index.insert(id.clone(), "/root", "ViewRoot", None);
            }

            // Index semantic nodes
            if let Some(ref sems) = root.semantic_nodes {
                for (i, sem) in sems.iter().enumerate() {
                    if let Some(ref id) = sem.node_id {
                        let path = format!("/root/semantic_nodes/{i}");
                        index.insert(id.clone(), &path, "SemanticNode", root.node_id.clone());
                    }
                }
            }

            // Index children recursively
            if let Some(ref children) = root.children {
                index.walk_children(children, "/root/children", root.node_id.clone());
            }
        }

        index
    }

    fn walk_children(
        &mut self,
        children: &[ChildNode],
        parent_path: &str,
        parent_id: Option<String>,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");
            let type_name = child.type_name().to_string();

            if let Some(id) = child.node_id() {
                self.insert(id.clone(), &path, &type_name, parent_id.clone());

                // Recurse into children if present
                if let Some(grandchildren) = child.children() {
                    let child_path = format!("{path}/children");
                    self.walk_children(&grandchildren, &child_path, Some(id));
                }
            }
        }
    }

    fn insert(&mut self, id: String, path: &str, type_name: &str, parent_id: Option<String>) {
        if let Some(existing) = self.nodes.get(&id) {
            self.duplicates
                .push((id.clone(), existing.path.clone(), path.to_string()));
        }

        self.nodes.insert(
            id.clone(),
            NodeEntry {
                path: path.to_string(),
                type_name: type_name.to_string(),
                parent_id,
            },
        );

        self.by_type
            .entry(type_name.to_string())
            .or_default()
            .push(id);
    }

    /// Look up a node by ID.
    pub fn get(&self, id: &str) -> Option<&NodeEntry> {
        self.nodes.get(id)
    }

    /// Check if a node ID exists.
    pub fn contains(&self, id: &str) -> bool {
        self.nodes.contains_key(id)
    }

    /// Get all node IDs of a given type.
    pub fn by_type(&self, type_name: &str) -> &[String] {
        self.by_type.get(type_name).map_or(&[], |v| v.as_slice())
    }

    /// Get all node IDs.
    pub fn all_ids(&self) -> HashSet<&str> {
        self.nodes.keys().map(|s| s.as_str()).collect()
    }

    /// Get the path for a node ID (for diagnostic messages).
    pub fn path(&self, id: &str) -> &str {
        self.nodes.get(id).map_or("<unknown>", |e| e.path.as_str())
    }
}
