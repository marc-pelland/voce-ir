//! Accessibility validation pass.
//!
//! Accessibility is a compile error in Voce IR. Missing semantic
//! information blocks compilation.

use crate::errors::{CodeMeta, Confidence, Diagnostic, Severity, ValidationResult};
use crate::index::NodeIndex;
use crate::ir::{ChildNode, VoceIr};
use crate::passes::ValidationPass;

pub struct AccessibilityPass;

const CODES: &[CodeMeta] = &[
    CodeMeta {
        code: "A11Y001",
        summary: "Interactive node has no SemanticNode for screen readers",
        hint: "Add a `SemanticNode` with the right `role` (`button`, `link`, `checkbox`, \
               etc.) and reference it via `semantic_node_id`. Voce treats accessibility \
               as a compile error — interactive nodes always need semantics.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y003",
        summary: "MediaNode is missing alt text and is not marked decorative",
        hint: "Set `alt` describing the image, OR add `semantic_node_id` referencing a \
               labeled SemanticNode, OR set `decorative: true` to mark it as purely \
               visual. Pick one — silence is not a valid option.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y004",
        summary: "Heading hierarchy skips a level (e.g. h1 → h3)",
        hint: "Heading levels jumped (e.g. h1 → h3, skipping h2). Demote the heading \
               to maintain a continuous hierarchy, or add the missing intermediate \
               heading. Screen readers depend on this for navigation.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y005",
        summary: "Form field is missing a label or aria-label",
        hint: "Set a `label` string on the FormField. If the design doesn't show a \
               visible label, set `aria_label` instead so screen readers can announce \
               the field.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "A11Y006",
        summary: "Link or button has no accessible text content",
        hint: "Add visible text content (a TextNode child) inside the interactive \
               element, OR set `semantic_node_id` referencing a SemanticNode with a \
               `label`. Icon-only buttons need explicit names.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y007",
        summary: "Text-on-background contrast ratio fails WCAG 2.2 AA",
        hint: "WCAG AA requires 4.5:1 for body text and 3:1 for large text \
               (≥18pt regular, or ≥14pt bold). Either lighten the text color, \
               darken the background, or — if this is decorative text on a \
               complex background — mark the surrounding Surface decorative \
               so the validator skips it. Partial-alpha colors are not \
               currently checked; document them as a known gap.",
        fix_confidence: None,
    },
    CodeMeta {
        code: "A11Y008",
        summary: "SemanticNode uses a positive tab_index, disrupting focus order",
        hint: "Positive tab_index values (1, 2, …) override DOM order and create \
               a custom focus path that screen-reader users have to memorize. \
               WCAG 2.4.3 expects focus order to follow meaning, which DOM \
               order already does. Use 0 (default — focusable, in DOM order) \
               or -1 (programmatic-only) instead. If you genuinely need a \
               custom order, restructure the IR so DOM order matches.",
        fix_confidence: Some(Confidence::Suggested),
    },
    CodeMeta {
        code: "A11Y009",
        summary: "Interactive element has a touch target smaller than 24×24 CSS px",
        hint: "WCAG 2.2 SC 2.5.8 requires interactive targets to be at least \
               24×24 CSS px. Heuristic: an interactive Surface whose padding-x \
               or padding-y totals less than 24 px without an explicit \
               min_width / min_height is likely below the threshold. Either \
               increase the padding, set explicit min dimensions, or — if the \
               element is purely decorative wrapping a real interactive child \
               — move the interactivity onto the larger ancestor.",
        fix_confidence: None,
    },
];

impl ValidationPass for AccessibilityPass {
    fn name(&self) -> &'static str {
        "accessibility"
    }

    fn codes(&self) -> &'static [CodeMeta] {
        CODES
    }

    fn run(&self, ir: &VoceIr, _index: &NodeIndex, result: &mut ValidationResult) {
        let root = match &ir.root {
            Some(r) => r,
            None => return,
        };

        // A11Y005: Check heading hierarchy
        let mut heading_levels: Vec<(String, i8)> = Vec::new();
        if let Some(ref children) = root.children {
            self.collect_headings(children, "/root/children", &mut heading_levels);
        }
        self.check_heading_hierarchy(&heading_levels, result);

        // Walk children for per-node checks
        if let Some(ref children) = root.children {
            self.check_children(children, "/root/children", result);
            // A11Y007: Color contrast. Only checks text where an ancestor
            // surface declares an explicit background — implicit defaults
            // are skipped so light-/dark-mode inversion doesn't false-fire.
            self.check_contrast(children, "/root/children", None, result);
            // A11Y009: touch target size on interactive surfaces.
            self.check_touch_targets(children, "/root/children", result);
        }

        // A11Y008: positive tab_index on SemanticNodes.
        if let Some(ref sems) = root.semantic_nodes {
            for (i, sem) in sems.iter().enumerate() {
                if let Some(ti) = sem.tab_index {
                    if ti > 0 {
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            code: "A11Y008".to_string(),
                            message: format!(
                                "SemanticNode tab_index = {ti} disrupts focus order — use 0 or -1"
                            ),
                            node_path: format!("/root/semantic_nodes/{i}"),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
            }
        }
    }
}

impl AccessibilityPass {
    fn check_children(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            match child.type_name() {
                // A11Y004: GestureHandler must have keyboard equivalent
                "GestureHandler" => {
                    if let Some(gh) = child.as_type::<crate::ir::GestureHandler>() {
                        if gh.keyboard_key.as_ref().is_none_or(|k| k.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y004".to_string(),
                                message:
                                    "GestureHandler must have a keyboard equivalent (keyboard_key)"
                                        .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // A11Y003: MediaNode images must have alt or be decorative
                "MediaNode" => {
                    if let Some(media) = child.as_type::<crate::ir::MediaNode>() {
                        let is_decorative = media.decorative.unwrap_or(false);
                        let has_alt = media.alt.as_ref().is_some_and(|a| !a.is_empty());
                        let has_semantic = child.semantic_node_id().is_some();

                        if !is_decorative && !has_alt && !has_semantic {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y003".to_string(),
                                message: "MediaNode must have alt text, a semantic_node_id, or be marked decorative".to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                    hint: None,
                            });
                        }
                    }
                }
                // A11Y001: FormNode must have semantic
                "FormNode" => {
                    if let Some(form) = child.as_type::<crate::ir::FormNode>() {
                        if form.semantic_node_id.as_ref().is_none_or(|s| s.is_empty()) {
                            result.diagnostics.push(Diagnostic {
                                severity: Severity::Error,
                                code: "A11Y001".to_string(),
                                message: "FormNode must have a semantic_node_id for accessibility"
                                    .to_string(),
                                node_path: path.clone(),
                                pass: self.name().to_string(),
                                hint: None,
                            });
                        }
                    }
                }
                // A11Y006: TextNode with href must have non-empty content for accessible link text
                "TextNode" => {
                    if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                        if text.href.as_ref().is_some_and(|h| !h.is_empty()) {
                            let has_content = text.content.as_ref().is_some_and(|c| !c.is_empty());
                            let has_semantic = child.semantic_node_id().is_some();
                            if !has_content && !has_semantic {
                                result.diagnostics.push(Diagnostic {
                                    severity: Severity::Error,
                                    code: "A11Y006".to_string(),
                                    message: "Link (TextNode with href) must have text content or a semantic_node_id with a label".to_string(),
                                    node_path: path.clone(),
                                    pass: self.name().to_string(),
                                    hint: Some("Add content text or a SemanticNode with an aria-label".to_string()),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }

            // Recurse
            if let Some(grandchildren) = child.children() {
                self.check_children(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }

    fn collect_headings(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        headings: &mut Vec<(String, i8)>,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            if child.type_name() == "TextNode" {
                if let Some(text) = child.as_type::<crate::ir::TextNode>() {
                    if let Some(level) = text.heading_level {
                        if level > 0 {
                            headings.push((path.clone(), level));
                        }
                    }
                }
            }

            if let Some(grandchildren) = child.children() {
                self.collect_headings(&grandchildren, &format!("{path}/children"), headings);
            }
        }
    }

    fn check_heading_hierarchy(&self, headings: &[(String, i8)], result: &mut ValidationResult) {
        if headings.is_empty() {
            return;
        }

        let mut prev_level: i8 = 0;
        for (path, level) in headings {
            if prev_level > 0 && *level > prev_level + 1 {
                result.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "A11Y005".to_string(),
                    message: format!(
                        "Heading level skipped: h{prev_level} -> h{level} (must not skip levels)"
                    ),
                    node_path: path.clone(),
                    pass: self.name().to_string(),
                    hint: None,
                });
            }
            prev_level = *level;
        }
    }

    /// A11Y007: walk the IR carrying the nearest-ancestor background color.
    /// At each TextNode that has an explicit `color`, compute contrast and
    /// emit a diagnostic if the WCAG AA threshold isn't met.
    ///
    /// Only emits when a real ancestor background was declared — implicit
    /// page defaults (white in light mode, dark in dark mode) would
    /// false-fire on near-white text the user wrote with dark mode in mind.
    /// Documented as a known gap; users with stricter requirements can
    /// declare an explicit Surface fill on the page wrapper.
    fn check_contrast(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        ambient_bg: Option<crate::contrast::Rgb>,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            // Update ambient bg if this node carries an explicit one.
            let next_bg = match child.type_name() {
                "Surface" => child
                    .value
                    .get("fill")
                    .and_then(crate::contrast::Rgb::from_json)
                    .or(ambient_bg),
                "Container" => child
                    .value
                    .get("background")
                    .and_then(crate::contrast::Rgb::from_json)
                    .or(ambient_bg),
                _ => ambient_bg,
            };

            // Check this TextNode's contrast against its ambient bg.
            if child.type_name() == "TextNode" {
                if let (Some(bg), Some(fg)) = (
                    ambient_bg,
                    child
                        .value
                        .get("color")
                        .and_then(crate::contrast::Rgb::from_json),
                ) {
                    let ratio = crate::contrast::contrast_ratio(fg, bg);
                    let font_size_px = child
                        .value
                        .get("font_size")
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(16.0);
                    let is_bold = child
                        .value
                        .get("font_weight")
                        .and_then(|v| v.as_str())
                        .is_some_and(|w| matches!(w, "Bold" | "ExtraBold" | "Black" | "SemiBold"));
                    let large = crate::contrast::is_large_text(font_size_px, is_bold);
                    let threshold = crate::contrast::aa_threshold(large);
                    if ratio < threshold {
                        let kind = if large { "large text" } else { "body text" };
                        result.diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            code: "A11Y007".to_string(),
                            message: format!(
                                "Text contrast {ratio:.2}:1 fails WCAG AA for {kind} (requires {threshold}:1)"
                            ),
                            node_path: path.clone(),
                            pass: self.name().to_string(),
                            hint: None,
                        });
                    }
                }
            }

            // Recurse into wrapper types that carry their own children list.
            if let Some(grandchildren) = child.children() {
                self.check_contrast(&grandchildren, &format!("{path}/children"), next_bg, result);
            }
        }
    }

    /// A11Y009: WCAG 2.2 SC 2.5.8 — interactive targets must be ≥ 24×24 CSS px.
    /// Heuristic since we can't render at validate time:
    ///
    /// - An interactive node is a Surface with `href`, a TextNode with
    ///   `href`, or a FormNode (the submit button is the interactive
    ///   surface here).
    /// - We sum the explicit padding on each axis. If padding-x < 24 OR
    ///   padding-y < 24, AND the node has no `min_width` / `min_height`
    ///   that would push past the threshold, we warn.
    /// - "≥ 24" includes 24 — exactly hitting the floor passes.
    ///
    /// Caveats: a small Surface with content that pushes its own size past
    /// 24 px (e.g. a 32 px-tall TextNode child) would clear the real WCAG
    /// rule even with no padding, but the heuristic warns anyway. The
    /// suggested fix in the hint covers that case (declare min_height).
    fn check_touch_targets(
        &self,
        children: &[ChildNode],
        parent_path: &str,
        result: &mut ValidationResult,
    ) {
        for (i, child) in children.iter().enumerate() {
            let path = format!("{parent_path}/{i}");

            // Only the *clickable surface itself* is a touch target. FormNode
            // is a container — its fields and submit button (rendered via
            // baseline form CSS with explicit padding) are the real targets,
            // and they get measured separately.
            let is_interactive = match child.type_name() {
                "Surface" => child.value.get("href").and_then(|v| v.as_str()).is_some(),
                "TextNode" => child.value.get("href").and_then(|v| v.as_str()).is_some(),
                _ => false,
            };

            if is_interactive {
                let pad_x = padding_axis(&child.value, "left", "right");
                let pad_y = padding_axis(&child.value, "top", "bottom");
                let min_w = length_px(child.value.get("min_width")).unwrap_or(0.0);
                let min_h = length_px(child.value.get("min_height")).unwrap_or(0.0);

                if (pad_x < 24.0 && min_w < 24.0) || (pad_y < 24.0 && min_h < 24.0) {
                    result.diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        code: "A11Y009".to_string(),
                        message: format!(
                            "Touch target may be < 24×24 px (padding {pad_x:.0}×{pad_y:.0}; min {min_w:.0}×{min_h:.0})"
                        ),
                        node_path: path.clone(),
                        pass: self.name().to_string(),
                        hint: None,
                    });
                }
            }

            if let Some(grandchildren) = child.children() {
                self.check_touch_targets(&grandchildren, &format!("{path}/children"), result);
            }
        }
    }
}

/// Sum the values of two padding sides (`top + bottom` for axis-y, etc.).
/// Reads `node.padding.<side>.value` from the raw JSON; missing or non-Px
/// units count as zero. Pixels are the only unit reliably comparable to
/// the WCAG threshold without rendering.
fn padding_axis(value: &serde_json::Value, side_a: &str, side_b: &str) -> f64 {
    let p = match value.get("padding") {
        Some(p) => p,
        None => return 0.0,
    };
    length_px(p.get(side_a)).unwrap_or(0.0) + length_px(p.get(side_b)).unwrap_or(0.0)
}

/// Read a Voce IR Length JSON: `{ "value": f64, "unit": "Px" }`. Returns
/// `Some(px)` only when unit is Px (or absent — Px is the default). Other
/// units (Rem, Percent, etc.) return None — they're not directly comparable
/// to 24 CSS pixels without rendering context.
fn length_px(value: Option<&serde_json::Value>) -> Option<f64> {
    let v = value?;
    let unit = v.get("unit").and_then(|u| u.as_str()).unwrap_or("Px");
    if unit != "Px" {
        return None;
    }
    v.get("value").and_then(|x| x.as_f64())
}
