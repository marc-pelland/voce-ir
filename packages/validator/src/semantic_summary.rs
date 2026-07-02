//! Semantic summary — a representation-independent description of a UI's
//! *meaning* (S68 cross-target parity; the algorithm S91 promotes to a
//! normative, portable contract).
//!
//! The same `SemanticSummary` can be derived two ways:
//! - [`SemanticSummary::from_ir`] — the contract, computed from the IR
//!   (the source of truth).
//! - [`SemanticSummary::from_html`] — observed, scraped from an HTML-family
//!   compiled artifact (DOM, Email, WebGPU shell, Hybrid).
//!
//! A conformant target preserves the IR's semantics: the observed summary
//! must match the IR-derived one on the dimensions that target is required
//! to preserve. Divergence is either a compiler bug or a documented,
//! medium-imposed degradation — never silent.

use std::collections::BTreeSet;

use crate::ir::{ChildNode, VoceIr};

/// Landmark roles that structure a page for assistive technology.
const LANDMARK_ROLES: &[&str] = &[
    "navigation",
    "main",
    "banner",
    "contentinfo",
    "complementary",
    "search",
    "region",
    "form",
];

/// A representation-independent snapshot of a UI's semantic structure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SemanticSummary {
    /// Heading levels (1–6) in document order. Order is semantically
    /// significant — screen-reader navigation depends on it.
    pub heading_levels: Vec<u8>,
    /// Hyperlinks (TextNode/Surface with href). Distinct from gestures
    /// because a constrained medium (e.g. email) can carry links but
    /// not JS-driven gesture handlers — conflating them would hide
    /// legitimate degradation.
    pub link_count: usize,
    /// JS-driven gesture targets (GestureHandler).
    pub gesture_count: usize,
    /// Total form fields across all forms.
    pub form_field_count: usize,
    /// Images that carry an accessible name (alt text or a semantic ref).
    pub media_with_name_count: usize,
    /// Images deliberately marked decorative (no accessible name needed).
    pub media_decorative_count: usize,
    /// Distinct landmark roles present, canonicalized and sorted.
    pub landmark_roles: BTreeSet<String>,
}

impl SemanticSummary {
    /// Derive the contract summary from IR JSON (the source of truth).
    pub fn from_ir(json: &str) -> Result<Self, serde_json::Error> {
        let ir: VoceIr = serde_json::from_str(json)?;
        let mut s = SemanticSummary::default();

        if let Some(root) = &ir.root {
            // Landmark roles declared on SemanticNodes.
            if let Some(sems) = &root.semantic_nodes {
                for sem in sems {
                    if let Some(role) = &sem.role {
                        let r = role.to_ascii_lowercase();
                        if LANDMARK_ROLES.contains(&r.as_str()) {
                            s.landmark_roles.insert(r);
                        }
                    }
                }
            }
            if let Some(children) = &root.children {
                s.walk_ir(children);
            }
        }
        Ok(s)
    }

    fn walk_ir(&mut self, children: &[ChildNode]) {
        for child in children {
            match child.type_name() {
                "TextNode" => {
                    if let Some(t) = child.as_type::<crate::ir::TextNode>() {
                        if let Some(level) = t.heading_level {
                            if (1..=6).contains(&level) {
                                self.heading_levels.push(level as u8);
                            }
                        }
                        if t.href.as_ref().is_some_and(|h| !h.is_empty()) {
                            self.link_count += 1;
                        }
                    }
                }
                "Surface" => {
                    // The validator IR model omits Surface.href; read it
                    // from the raw node value.
                    let has_href = child
                        .value
                        .get("href")
                        .and_then(|v| v.as_str())
                        .is_some_and(|h| !h.is_empty());
                    if has_href {
                        self.link_count += 1;
                    }
                }
                "GestureHandler" => self.gesture_count += 1,
                "MediaNode" => {
                    if let Some(m) = child.as_type::<crate::ir::MediaNode>() {
                        if m.decorative.unwrap_or(false) {
                            self.media_decorative_count += 1;
                        } else if m.alt.as_ref().is_some_and(|a| !a.is_empty())
                            || child.semantic_node_id().is_some()
                        {
                            self.media_with_name_count += 1;
                        }
                    }
                }
                "FormNode" => {
                    if let Some(f) = child.as_type::<crate::ir::FormNode>() {
                        self.form_field_count += f.fields.as_ref().map(|v| v.len()).unwrap_or(0);
                        self.landmark_roles.insert("form".to_string());
                    }
                }
                _ => {}
            }
            if let Some(grandchildren) = child.children() {
                self.walk_ir(&grandchildren);
            }
        }
    }

    /// Derive the observed summary from an HTML-family compiled artifact.
    ///
    /// A deliberately lightweight tag scan — robust enough to detect
    /// dropped/gained semantic structure without a full HTML parser.
    pub fn from_html(html: &str) -> Self {
        let lower = html.to_ascii_lowercase();
        let mut s = SemanticSummary::default();

        // Heading levels in document order.
        let bytes = lower.as_bytes();
        let mut i = 0;
        while i + 2 < bytes.len() {
            if bytes[i] == b'<' && bytes[i + 1] == b'h' && bytes[i + 2].is_ascii_digit() {
                let level = bytes[i + 2] - b'0';
                // Require a tag boundary so we don't match <html> or <hr>.
                let after = bytes.get(i + 3).copied().unwrap_or(b' ');
                if (1..=6).contains(&level) && (after == b'>' || after == b' ' || after == b'\t') {
                    s.heading_levels.push(level);
                }
            }
            i += 1;
        }

        s.link_count = count_occurrences(&lower, "<a ");
        // The DOM/Hybrid compilers mark JS gesture targets with
        // data-voce-id; a JS-less medium (email) emits none.
        s.gesture_count = count_occurrences(&lower, "data-voce-id=");
        s.form_field_count = count_occurrences(&lower, "<input")
            + count_occurrences(&lower, "<textarea")
            + count_occurrences(&lower, "<select");

        // Images with a non-empty alt vs. explicitly empty alt (decorative).
        for (idx, _) in lower.match_indices("<img") {
            let tag_end = lower[idx..]
                .find('>')
                .map(|e| idx + e)
                .unwrap_or(lower.len());
            let tag = &lower[idx..tag_end];
            if tag.contains("alt=\"\"") || tag.contains("aria-hidden=\"true\"") {
                s.media_decorative_count += 1;
            } else if tag.contains("alt=\"") || tag.contains("aria-label=\"") {
                s.media_with_name_count += 1;
            }
        }

        for (tag, role) in [
            ("<nav", "navigation"),
            ("<main", "main"),
            ("<header", "banner"),
            ("<footer", "contentinfo"),
            ("<aside", "complementary"),
            ("<form", "form"),
        ] {
            if lower.contains(tag) {
                s.landmark_roles.insert(role.to_string());
            }
        }
        for role in LANDMARK_ROLES {
            if lower.contains(&format!("role=\"{role}\"")) {
                s.landmark_roles.insert((*role).to_string());
            }
        }
        s
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_ir_counts_headings_in_order() {
        let json = r#"{ "root": { "node_id": "r", "children": [
            { "value_type": "TextNode", "value": { "node_id": "a", "content": "Title", "heading_level": 1 } },
            { "value_type": "TextNode", "value": { "node_id": "b", "content": "Sub", "heading_level": 2 } },
            { "value_type": "TextNode", "value": { "node_id": "c", "content": "Body" } }
        ] } }"#;
        let s = SemanticSummary::from_ir(json).unwrap();
        assert_eq!(s.heading_levels, vec![1, 2]);
    }

    #[test]
    fn from_ir_counts_form_fields_and_landmark() {
        let json = r#"{ "root": { "node_id": "r", "children": [
            { "value_type": "FormNode", "value": { "node_id": "f", "semantic_node_id": "s",
              "fields": [ { "name": "email", "field_type": "email" }, { "name": "msg", "field_type": "textarea" } ] } }
        ] } }"#;
        let s = SemanticSummary::from_ir(json).unwrap();
        assert_eq!(s.form_field_count, 2);
        assert!(s.landmark_roles.contains("form"));
    }

    #[test]
    fn from_html_matches_equivalent_structure() {
        let html = r#"<!DOCTYPE html><html><body>
            <h1>Title</h1><h2>Sub</h2>
            <form><input type="email"><textarea></textarea></form>
        </body></html>"#;
        let s = SemanticSummary::from_html(html);
        assert_eq!(s.heading_levels, vec![1, 2]);
        assert_eq!(s.form_field_count, 2);
        assert!(s.landmark_roles.contains("form"));
    }

    #[test]
    fn html_scan_ignores_html_and_hr_tags() {
        let html = "<html><body><hr><h2 class=\"x\">Real</h2></body></html>";
        let s = SemanticSummary::from_html(html);
        assert_eq!(s.heading_levels, vec![2]);
    }
}
