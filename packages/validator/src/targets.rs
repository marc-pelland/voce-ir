//! Canonical compile-target registry.
//!
//! The single source of truth for "what targets exist," consumed by the
//! S79 agent capability manifest (`voce skills`). Today the `voce`
//! binary itself only drives the DOM target via `voce compile`, but the
//! project ships 7 compiler crates and the agent contract must describe
//! the full design space honestly — not just what this one binary
//! happens to wire up. Future `voce compile --target` will read this
//! same registry, so it stays a single list, not a hand-maintained
//! parallel to docs.

/// Stability of a target's output contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Stability {
    /// Output shape is committed; semver applies.
    Stable,
    /// Output is functional but may change between minor releases.
    Beta,
    /// Output exists but is exploratory.
    Experimental,
}

/// Metadata describing one compile target.
#[derive(Debug, Clone, serde::Serialize, schemars::JsonSchema)]
pub struct TargetInfo {
    /// Stable identifier used in CLI flags, JSON contracts, and docs.
    pub id: &'static str,
    /// Human-readable display name.
    pub name: &'static str,
    /// Output artifact kinds this target emits (e.g. `["html"]`,
    /// `["swift"]`, `["html", "wat"]`).
    pub outputs: &'static [&'static str],
    /// Conformance class — what the target is *required* to preserve.
    /// See `docs/compatibility-matrix.md` for the live verification.
    pub conformance_class: ConformanceClass,
    pub stability: Stability,
    /// One-line note — what this target is for / its constraints.
    pub notes: &'static str,
}

/// Maps onto the S68 capability-profile model: full preservation
/// (oracle/superset), required-contract-only (medium-constrained),
/// non-HTML-semantic (visual/GPU), or logic-only (no UI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ConformanceClass {
    OracleFull,
    OracleFullSuperset,
    RequiredContract,
    NonHtmlVisual,
    LogicOnly,
    Native,
}

/// All compile targets the Voce project ships.
pub const ALL: &[TargetInfo] = &[
    TargetInfo {
        id: "dom",
        name: "DOM (HTML)",
        outputs: &["html"],
        conformance_class: ConformanceClass::OracleFull,
        stability: Stability::Stable,
        notes: "Single-file HTML, zero runtime dependencies. The reference / oracle target.",
    },
    TargetInfo {
        id: "hybrid",
        name: "Hybrid (DOM + WASM)",
        outputs: &["html", "wasm"],
        conformance_class: ConformanceClass::OracleFullSuperset,
        stability: Stability::Stable,
        notes: "DOM plus a WASM payload for hot interaction paths.",
    },
    TargetInfo {
        id: "email",
        name: "Email HTML",
        outputs: &["html"],
        conformance_class: ConformanceClass::RequiredContract,
        stability: Stability::Stable,
        notes: "Table-layout email-safe HTML. Forms, gestures, landmarks degrade by medium.",
    },
    TargetInfo {
        id: "webgpu",
        name: "WebGPU",
        outputs: &["html", "wgsl"],
        conformance_class: ConformanceClass::NonHtmlVisual,
        stability: Stability::Beta,
        notes: "GPU-painted UI behind an HTML shell. Semantic parity needs a11y-tree extraction.",
    },
    TargetInfo {
        id: "wasm",
        name: "WASM (state machines)",
        outputs: &["wat"],
        conformance_class: ConformanceClass::LogicOnly,
        stability: Stability::Beta,
        notes: "Compiles IR state machines to WebAssembly text. Logic, not UI.",
    },
    TargetInfo {
        id: "ios-swiftui",
        name: "iOS (SwiftUI)",
        outputs: &["swift"],
        conformance_class: ConformanceClass::Native,
        stability: Stability::Beta,
        notes: "SwiftUI source for iOS / macOS Catalyst.",
    },
    TargetInfo {
        id: "android-compose",
        name: "Android (Jetpack Compose)",
        outputs: &["kotlin"],
        conformance_class: ConformanceClass::Native,
        stability: Stability::Beta,
        notes: "Kotlin/Jetpack Compose source for Android.",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_all_seven_targets() {
        assert_eq!(ALL.len(), 7);
    }

    #[test]
    fn target_ids_are_unique() {
        let mut ids: Vec<&str> = ALL.iter().map(|t| t.id).collect();
        ids.sort_unstable();
        let len = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), len, "duplicate target id");
    }

    #[test]
    fn dom_is_the_oracle() {
        let dom = ALL.iter().find(|t| t.id == "dom").expect("dom present");
        assert_eq!(dom.conformance_class, ConformanceClass::OracleFull);
        assert_eq!(dom.stability, Stability::Stable);
    }
}
