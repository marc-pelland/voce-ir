//! Compiler IR — an arena-based graph representation optimized for code generation.
//!
//! This is distinct from the validator's serde IR model. The compiler IR uses:
//! - Concrete types (no Option for required fields — validation already passed)
//! - Arena-based storage with handle indices for O(1) traversal
//! - Typed node variants for exhaustive pattern matching

use std::collections::HashMap;

/// Handle into the node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// The complete compiler IR document.
#[derive(Debug)]
pub struct CompilerIr {
    /// Arena of all nodes.
    pub nodes: Vec<CNode>,
    /// Root node handle.
    pub root: NodeId,
    /// Node ID string → arena index mapping.
    pub id_map: HashMap<String, NodeId>,
    /// Document-level metadata.
    pub meta: DocumentMeta,
    /// State machines to compile to JS.
    pub state_machines: Vec<CompiledStateMachine>,
    /// Gesture handlers to compile to event listeners.
    pub gesture_handlers: Vec<CompiledGestureHandler>,
    /// Animations to compile to CSS/JS.
    pub animations: Vec<CompiledAnimation>,
    /// Forms to compile to HTML + validation JS.
    pub forms: Vec<CompiledForm>,
    /// Semantic nodes indexed by their node_id for ARIA emission.
    pub semantic_map: HashMap<String, SemanticInfo>,
    /// Responsive rules — media queries with property overrides.
    pub responsive_rules: Vec<CompiledResponsiveRule>,
}

/// A responsive media query rule.
#[derive(Debug, Clone)]
pub struct CompiledResponsiveRule {
    /// Minimum viewport width (in px) for this rule.
    pub min_width_px: f64,
    /// Property overrides: (target_node_id, css_property, css_value).
    pub overrides: Vec<(String, String, String)>,
}

/// Semantic information for ARIA attribute emission.
#[derive(Debug, Clone, Default)]
pub struct SemanticInfo {
    pub role: Option<String>,
    pub label: Option<String>,
    pub labelled_by: Option<String>,
    pub described_by: Option<String>,
    pub tab_index: Option<i32>,
}

/// A form ready for HTML + JS compilation.
#[derive(Debug, Clone)]
pub struct CompiledForm {
    pub id: String,
    pub fields: Vec<CompiledFormField>,
    pub action_endpoint: Option<String>,
    pub action_method: String,
    pub progressive: bool,
}

#[derive(Debug, Clone)]
pub struct CompiledFormField {
    pub name: String,
    pub field_type: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub autocomplete: Option<String>,
    pub validations: Vec<CompiledValidationRule>,
    pub description: Option<String>,
    /// Options for Select, Radio fields.
    pub options: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompiledValidationRule {
    pub rule_type: String,
    pub value: Option<String>,
    pub message: String,
}

/// A state machine ready for JS compilation.
#[derive(Debug, Clone)]
pub struct CompiledStateMachine {
    pub id: String,
    pub name: String,
    pub initial_state: String,
    pub states: Vec<String>,
    pub transitions: Vec<CompiledTransition>,
}

#[derive(Debug, Clone)]
pub struct CompiledTransition {
    pub event: String,
    pub from: String,
    pub to: String,
    pub guard: Option<String>,
    pub effect: Option<String>,
}

/// An animation transition ready for CSS/JS compilation.
#[derive(Debug, Clone)]
pub struct CompiledAnimation {
    pub id: String,
    pub target_node_id: String,
    pub properties: Vec<(String, String, String)>, // (property, from, to)
    pub duration_ms: f64,
    pub easing_css: String,
    pub has_reduced_motion: bool,
    pub reduced_motion_strategy: String,
}

/// A gesture handler ready for JS compilation.
#[derive(Debug, Clone)]
pub struct CompiledGestureHandler {
    pub id: String,
    pub target_node_id: String,
    pub gesture_type: String,
    pub trigger_event: Option<String>,
    pub trigger_state_machine: Option<String>,
    pub keyboard_key: Option<String>,
}

/// Document-level metadata.
#[derive(Debug, Default)]
pub struct DocumentMeta {
    pub schema_version: String,
    pub language: Option<String>,
    pub text_direction: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub canonical_url: Option<String>,
    pub og_title: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Option<String>,
    /// Theme CSS custom properties (name → value pairs).
    pub theme_vars: Vec<(String, String)>,
    /// JSON-LD structured data blocks.
    pub structured_data: Vec<String>,
}

/// A compiler node — wraps the type-specific data with common fields.
#[derive(Debug)]
pub struct CNode {
    /// String node_id from the IR.
    pub id: String,
    /// Node type and type-specific data.
    pub kind: NodeKind,
    /// Children handles.
    pub children: Vec<NodeId>,
    /// Semantic node reference (for ARIA emission).
    pub semantic_node_id: Option<String>,
    /// Inline styles to emit (populated during lowering).
    pub styles: HashMap<String, String>,
}

/// Type-specific node data for the compiler.
#[derive(Debug)]
pub enum NodeKind {
    ViewRoot {
        language: Option<String>,
    },
    Container {
        layout: String,
        direction: String,
        main_align: String,
        cross_align: String,
        gap: Option<String>,
        wrap: bool,
    },
    Surface {
        decorative: bool,
        href: Option<String>,
        target: Option<String>,
    },
    Text {
        content: String,
        heading_level: i8,
        tag: String, // "h1"-"h6", "p", "span", "a"
        href: Option<String>,
        target: Option<String>,
    },
    Media {
        src: String,
        alt: String,
        media_type: String,
        decorative: bool,
        above_fold: bool,
    },
    /// Rich text content — paragraphs, headings, lists, tables, code blocks.
    RichText {
        blocks: Vec<RichTextBlock>,
    },
    /// Catch-all for nodes the compiler doesn't emit HTML for directly
    /// (StateMachine, GestureHandler, AnimationTransition, etc.)
    /// These contribute JS or metadata, not HTML structure.
    NonVisual {
        type_name: String,
        data: serde_json::Value,
    },
}

/// A block in a RichTextNode.
#[derive(Debug)]
pub struct RichTextBlock {
    pub block_type: String,
    pub level: i8,
    pub children: Vec<RichTextSpan>,
    pub media_src: Option<String>,
    pub media_alt: Option<String>,
    pub code_language: Option<String>,
    pub rows: Vec<RichTextBlock>,
}

/// An inline span within a rich text block.
#[derive(Debug)]
pub struct RichTextSpan {
    pub text: String,
    pub marks: Vec<String>,
    pub link_url: Option<String>,
}
