//! Serde-deserializable IR model for validation.
//!
//! These structs mirror the FlatBuffers JSON canonical format.
//! They are used by the validator to traverse the IR tree, build
//! indexes, and run validation passes. Fields that validation
//! passes don't inspect are captured as `serde_json::Value` to
//! ensure round-trip fidelity without requiring exhaustive typing.

use serde::Deserialize;

/// The top-level IR document.
#[derive(Debug, Clone, Deserialize)]
pub struct VoceIr {
    #[serde(default)]
    pub schema_version_major: i32,
    #[serde(default)]
    pub schema_version_minor: i32,
    pub root: Option<ViewRoot>,
    pub routes: Option<RouteMap>,
    pub theme: Option<ThemeNode>,
    pub alternate_themes: Option<Vec<ThemeNode>>,
    pub auth: Option<AuthContextNode>,
    pub i18n: Option<I18nConfig>,
}

/// ViewRoot — document root, one per route.
#[derive(Debug, Clone, Deserialize)]
pub struct ViewRoot {
    pub node_id: Option<String>,
    pub children: Option<Vec<ChildNode>>,
    pub document_language: Option<String>,
    pub text_direction: Option<String>,
    pub semantic_nodes: Option<Vec<SemanticNode>>,
    pub metadata: Option<PageMetadata>,
}

/// ChildNode — wrapper for the union type.
/// FlatBuffers JSON format: `{ "value_type": "TypeName", "value": { ... } }`
#[derive(Debug, Clone, Deserialize)]
pub struct ChildNode {
    pub value_type: Option<String>,
    #[serde(default)]
    pub value: serde_json::Value,
}

// ─── Layout Nodes ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct Container {
    pub node_id: Option<String>,
    pub children: Option<Vec<ChildNode>>,
    pub layout: Option<String>,
    pub semantic_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Surface {
    pub node_id: Option<String>,
    pub children: Option<Vec<ChildNode>>,
    pub decorative: Option<bool>,
    pub semantic_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextNode {
    pub node_id: Option<String>,
    pub content: Option<String>,
    pub content_binding: Option<serde_json::Value>,
    pub localized_content: Option<LocalizedString>,
    pub heading_level: Option<i8>,
    pub href: Option<String>,
    pub semantic_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MediaNode {
    pub node_id: Option<String>,
    pub src: Option<String>,
    pub alt: Option<String>,
    pub decorative: Option<bool>,
    pub semantic_node_id: Option<String>,
}

// ─── State Nodes ────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct StateMachine {
    pub node_id: Option<String>,
    pub name: Option<String>,
    pub states: Option<Vec<State>>,
    pub transitions: Option<Vec<Transition>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct State {
    pub name: Option<String>,
    #[serde(default)]
    pub initial: bool,
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Transition {
    pub event: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub guard: Option<String>,
    pub effect: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataNode {
    pub node_id: Option<String>,
    pub name: Option<String>,
    pub source: Option<serde_json::Value>,
    pub auth_required: Option<bool>,
    pub loading_state_machine: Option<String>,
    pub cache_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeNode {
    pub node_id: Option<String>,
    pub inputs: Option<Vec<ComputeInput>>,
    pub expression: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeInput {
    pub name: Option<String>,
    pub source_node_id: Option<String>,
    pub field_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffectNode {
    pub node_id: Option<String>,
    pub effect_type: Option<String>,
    pub idempotent: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextNode {
    pub node_id: Option<String>,
    pub name: Option<String>,
}

// ─── Motion Nodes ───────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AnimationTransition {
    pub node_id: Option<String>,
    pub target_node_id: Option<String>,
    pub trigger_state_machine: Option<String>,
    pub trigger_event: Option<String>,
    pub reduced_motion: Option<ReducedMotion>,
    pub duration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Sequence {
    pub node_id: Option<String>,
    pub steps: Option<Vec<SequenceStep>>,
    pub reduced_motion: Option<ReducedMotion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SequenceStep {
    pub transition_id: Option<String>,
    #[serde(default)]
    pub parallel: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GestureHandler {
    pub node_id: Option<String>,
    pub target_node_id: Option<String>,
    pub gesture_type: Option<String>,
    pub trigger_event: Option<String>,
    pub trigger_state_machine: Option<String>,
    pub keyboard_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScrollBinding {
    pub node_id: Option<String>,
    pub target_node_id: Option<String>,
    pub reduced_motion: Option<ReducedMotion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PhysicsBody {
    pub node_id: Option<String>,
    pub target_node_id: Option<String>,
    #[serde(default)]
    pub interruptible: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReducedMotion {
    pub strategy: Option<String>,
    pub simplified_properties: Option<Vec<serde_json::Value>>,
    pub reduced_duration: Option<serde_json::Value>,
}

// ─── Navigation Nodes ───────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct RouteMap {
    pub node_id: Option<String>,
    pub routes: Option<Vec<RouteEntry>>,
    pub not_found_route: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteEntry {
    pub path: Option<String>,
    pub name: Option<String>,
    pub view_root_id: Option<String>,
    pub guard: Option<RouteGuard>,
    pub children: Option<Vec<RouteEntry>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RouteGuard {
    #[serde(default)]
    pub requires_auth: bool,
    pub required_roles: Option<Vec<String>>,
    pub redirect_on_fail: Option<String>,
}

// ─── A11y Nodes ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SemanticNode {
    pub node_id: Option<String>,
    pub role: Option<String>,
    pub label: Option<String>,
    pub labelled_by: Option<String>,
    pub described_by: Option<String>,
    pub controls: Option<String>,
    pub heading_level: Option<i8>,
    pub tab_index: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiveRegion {
    pub node_id: Option<String>,
    pub target_node_id: Option<String>,
    pub politeness: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FocusTrap {
    pub node_id: Option<String>,
    pub container_node_id: Option<String>,
    pub initial_focus_node_id: Option<String>,
    pub escape_behavior: Option<String>,
    pub escape_state_machine: Option<String>,
    pub escape_event: Option<String>,
}

// ─── Theming Nodes ──────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ThemeNode {
    pub node_id: Option<String>,
    pub name: Option<String>,
    pub colors: Option<serde_json::Value>,
    pub typography: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PersonalizationSlot {
    pub node_id: Option<String>,
    pub variants: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponsiveRule {
    pub node_id: Option<String>,
    pub breakpoints: Option<Vec<serde_json::Value>>,
}

// ─── Data Nodes ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ActionNode {
    pub node_id: Option<String>,
    pub source: Option<serde_json::Value>,
    pub method: Option<String>,
    pub auth_required: Option<bool>,
    pub csrf_protected: Option<bool>,
    pub invalidates: Option<Vec<String>>,
    pub invalidate_tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionNode {
    pub node_id: Option<String>,
    pub target_data_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthContextNode {
    pub node_id: Option<String>,
    pub provider: Option<String>,
    pub login_action_id: Option<String>,
    pub logout_action_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentSlot {
    pub node_id: Option<String>,
    pub content_key: Option<String>,
    pub cache_strategy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RichTextNode {
    pub node_id: Option<String>,
    pub blocks: Option<Vec<serde_json::Value>>,
}

// ─── Form Nodes ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct FormNode {
    pub node_id: Option<String>,
    pub fields: Option<Vec<FormField>>,
    pub submission: Option<FormSubmission>,
    pub semantic_node_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormField {
    pub name: Option<String>,
    pub field_type: Option<String>,
    pub label: Option<String>,
    pub validations: Option<Vec<ValidationRule>>,
    pub semantic_node_id: Option<String>,
    pub autocomplete: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidationRule {
    pub rule_type: Option<String>,
    pub value: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormSubmission {
    pub action_node_id: Option<String>,
    #[serde(default)]
    pub progressive: bool,
}

// ─── SEO ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct PageMetadata {
    pub title: Option<String>,
    pub title_template: Option<String>,
    pub description: Option<String>,
    pub canonical_url: Option<String>,
    pub open_graph: Option<OpenGraphData>,
    pub twitter_card: Option<serde_json::Value>,
    pub alternates: Option<Vec<AlternateLink>>,
    pub structured_data: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenGraphData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlternateLink {
    pub hreflang: Option<String>,
    pub href: Option<String>,
}

// ─── i18n ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct LocalizedString {
    pub message_key: Option<String>,
    pub default_value: Option<String>,
    pub parameters: Option<Vec<serde_json::Value>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct I18nConfig {
    pub default_locale: Option<String>,
    pub supported_locales: Option<Vec<String>>,
    pub mode: Option<String>,
}

// ─── Node Extraction Helper ─────────────────────────────────────

impl ChildNode {
    /// Try to extract the node_id from the inner value, regardless of type.
    pub fn node_id(&self) -> Option<String> {
        self.value
            .get("node_id")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Get the type discriminator.
    pub fn type_name(&self) -> &str {
        self.value_type.as_deref().unwrap_or("NONE")
    }

    /// Does this node type have children?
    pub fn children(&self) -> Option<Vec<ChildNode>> {
        self.value
            .get("children")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get the semantic_node_id if present.
    pub fn semantic_node_id(&self) -> Option<String> {
        self.value
            .get("semantic_node_id")
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    /// Deserialize the inner value as a specific type.
    pub fn as_type<T: serde::de::DeserializeOwned>(&self) -> Option<T> {
        serde_json::from_value(self.value.clone()).ok()
    }
}
