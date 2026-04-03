//! Font optimization pipeline — glyph collection, @font-face generation,
//! fallback stacks with metric overrides, preload hints.
//!
//! Collects text content from IR, determines required Unicode codepoints,
//! generates optimized CSS font declarations with content-hashed filenames.

use std::collections::{BTreeSet, HashMap};

/// Collected font usage data from the IR tree.
#[derive(Debug, Default)]
pub struct FontUsage {
    /// Font families used, with their codepoints and weights.
    pub families: HashMap<String, FontFamilyUsage>,
}

/// Usage data for a single font family.
#[derive(Debug, Default)]
pub struct FontFamilyUsage {
    /// All Unicode codepoints used with this font.
    pub codepoints: BTreeSet<u32>,
    /// Font weights used (e.g., 400, 700).
    pub weights: BTreeSet<u16>,
    /// Whether this font appears above the fold (should be preloaded).
    pub above_fold: bool,
}

/// A processed font ready for emission.
#[derive(Debug)]
pub struct ProcessedFont {
    /// Font family name.
    pub family: String,
    /// Font weight.
    pub weight: u16,
    /// Content-hashed WOFF2 filename (if source bytes provided).
    pub woff2_filename: Option<String>,
    /// Source font bytes in WOFF2 format (if processed).
    pub woff2_data: Option<Vec<u8>>,
    /// Unicode range string for @font-face.
    pub unicode_range: String,
    /// Whether to emit a preload hint.
    pub preload: bool,
}

/// System font fallback stacks with metric adjustments.
pub struct FallbackStack {
    /// CSS font-family value with fallbacks.
    pub family_stack: String,
    /// CSS size-adjust for the fallback (prevents layout shift).
    pub size_adjust: Option<f64>,
    /// CSS ascent-override for the fallback.
    pub ascent_override: Option<f64>,
    /// CSS descent-override for the fallback.
    pub descent_override: Option<f64>,
}

/// Safety codepoints always included in subsets (digits, punctuation, common symbols).
const SAFETY_CODEPOINTS: &[u32] = &[
    // Space, basic punctuation
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E,
    0x2F,
    // Digits
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39,
    // More punctuation
    0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x40,
    // Common symbols
    0x5B, 0x5C, 0x5D, 0x5E, 0x5F, 0x60, 0x7B, 0x7C, 0x7D, 0x7E,
    // Non-breaking space, en-dash, em-dash, ellipsis
    0xA0, 0x2013, 0x2014, 0x2026,
    // Smart quotes
    0x2018, 0x2019, 0x201C, 0x201D,
];

/// Collect font usage from an IR JSON document.
pub fn collect_font_usage(doc: &serde_json::Value) -> FontUsage {
    let mut usage = FontUsage::default();

    if let Some(root) = doc.get("root") {
        if let Some(children) = root.get("children").and_then(|v| v.as_array()) {
            for child in children {
                collect_from_node(child, &mut usage, true);
            }
        }
    }

    // Add safety codepoints to all families
    for family_usage in usage.families.values_mut() {
        for &cp in SAFETY_CODEPOINTS {
            family_usage.codepoints.insert(cp);
        }
    }

    usage
}

fn collect_from_node(node: &serde_json::Value, usage: &mut FontUsage, above_fold: bool) {
    let type_name = node
        .get("value_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let value = node.get("value").cloned().unwrap_or(serde_json::Value::Null);

    if type_name == "TextNode" {
        let content = value.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let family = value
            .get("font_family")
            .and_then(|v| v.as_str())
            .unwrap_or("system-ui");
        let weight = value
            .get("font_weight")
            .and_then(|v| v.as_str())
            .map(weight_name_to_number)
            .unwrap_or(400);

        let entry = usage.families.entry(family.to_string()).or_default();
        for ch in content.chars() {
            entry.codepoints.insert(ch as u32);
        }
        entry.weights.insert(weight);
        if above_fold {
            entry.above_fold = true;
        }
    }

    // Recurse into children
    if let Some(children) = value.get("children").and_then(|v| v.as_array()) {
        for child in children {
            collect_from_node(child, usage, above_fold);
        }
    }
}

/// Convert a CSS font-weight name to numeric value.
fn weight_name_to_number(name: &str) -> u16 {
    match name {
        "Thin" => 100,
        "ExtraLight" | "UltraLight" => 200,
        "Light" => 300,
        "Regular" | "Normal" => 400,
        "Medium" => 500,
        "SemiBold" | "DemiBold" => 600,
        "Bold" => 700,
        "ExtraBold" | "UltraBold" => 800,
        "Black" | "Heavy" => 900,
        _ => 400,
    }
}

/// Generate a `unicode-range` CSS descriptor from a set of codepoints.
pub fn unicode_range(codepoints: &BTreeSet<u32>) -> String {
    if codepoints.is_empty() {
        return String::new();
    }

    // Merge contiguous ranges
    let mut ranges: Vec<(u32, u32)> = Vec::new();
    for &cp in codepoints {
        if let Some(last) = ranges.last_mut() {
            if cp == last.1 + 1 {
                last.1 = cp;
                continue;
            }
        }
        ranges.push((cp, cp));
    }

    ranges
        .iter()
        .map(|&(start, end)| {
            if start == end {
                format!("U+{start:04X}")
            } else {
                format!("U+{start:04X}-{end:04X}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate `@font-face` CSS block for a font.
pub fn font_face_css(
    family: &str,
    weight: u16,
    woff2_url: Option<&str>,
    unicode_range_str: &str,
) -> String {
    let src = if let Some(url) = woff2_url {
        format!("local('{family}'), url('{url}') format('woff2')")
    } else {
        format!("local('{family}')")
    };

    format!(
        "@font-face {{\n\
         \x20 font-family: '{family}';\n\
         \x20 font-style: normal;\n\
         \x20 font-weight: {weight};\n\
         \x20 font-display: swap;\n\
         \x20 src: {src};\n\
         \x20 unicode-range: {unicode_range_str};\n\
         }}"
    )
}

/// Generate a preload `<link>` tag for a font file.
pub fn preload_link(url: &str) -> String {
    format!(
        "<link rel=\"preload\" as=\"font\" type=\"font/woff2\" href=\"{url}\" crossorigin>"
    )
}

/// Get a system font fallback stack with metric adjustments for a given font family.
///
/// Returns the fallback CSS and optional metric overrides to prevent layout shift.
pub fn fallback_stack(family: &str) -> FallbackStack {
    // Known font metrics for popular fonts (size-adjust, ascent-override, descent-override)
    match family {
        "Inter" => FallbackStack {
            family_stack: format!(
                "'{family}', 'Inter fallback', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
            ),
            size_adjust: Some(107.0),
            ascent_override: Some(90.0),
            descent_override: Some(22.0),
        },
        "Roboto" => FallbackStack {
            family_stack: format!(
                "'{family}', 'Roboto fallback', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
            ),
            size_adjust: Some(100.3),
            ascent_override: Some(92.0),
            descent_override: Some(24.0),
        },
        _ => FallbackStack {
            family_stack: format!(
                "'{family}', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif"
            ),
            size_adjust: None,
            ascent_override: None,
            descent_override: None,
        },
    }
}

/// Generate fallback @font-face with metric overrides to prevent CLS.
pub fn fallback_font_face_css(family: &str, stack: &FallbackStack) -> Option<String> {
    let size_adjust = stack.size_adjust?;
    let ascent = stack.ascent_override?;
    let descent = stack.descent_override?;

    Some(format!(
        "@font-face {{\n\
         \x20 font-family: '{family} fallback';\n\
         \x20 src: local('-apple-system'), local('BlinkMacSystemFont'), local('Segoe UI');\n\
         \x20 size-adjust: {size_adjust:.1}%;\n\
         \x20 ascent-override: {ascent:.1}%;\n\
         \x20 descent-override: {descent:.1}%;\n\
         }}"
    ))
}

/// Generate a content hash for font data (for cache-busting filenames).
pub fn font_content_hash(data: &[u8]) -> String {
    let hash: u32 = data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
    format!("{:06x}", hash & 0xFFFFFF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_name_conversion() {
        assert_eq!(weight_name_to_number("Bold"), 700);
        assert_eq!(weight_name_to_number("Regular"), 400);
        assert_eq!(weight_name_to_number("SemiBold"), 600);
        assert_eq!(weight_name_to_number("Unknown"), 400);
    }

    #[test]
    fn unicode_range_single_codepoints() {
        let mut cps = BTreeSet::new();
        cps.insert(0x41); // A
        cps.insert(0x42); // B
        cps.insert(0x43); // C
        cps.insert(0x50); // P (gap)
        let range = unicode_range(&cps);
        assert_eq!(range, "U+0041-0043, U+0050");
    }

    #[test]
    fn unicode_range_empty() {
        let range = unicode_range(&BTreeSet::new());
        assert!(range.is_empty());
    }

    #[test]
    fn font_face_has_display_swap() {
        let css = font_face_css("Inter", 400, Some("/fonts/inter.woff2"), "U+0020-007E");
        assert!(css.contains("font-display: swap"));
        assert!(css.contains("font-family: 'Inter'"));
        assert!(css.contains("font-weight: 400"));
        assert!(css.contains("url('/fonts/inter.woff2')"));
        assert!(css.contains("local('Inter')"));
    }

    #[test]
    fn font_face_local_only() {
        let css = font_face_css("Inter", 700, None, "U+0041");
        assert!(css.contains("font-display: swap"));
        assert!(css.contains("local('Inter')"));
        assert!(!css.contains("url("));
    }

    #[test]
    fn preload_link_format() {
        let link = preload_link("/fonts/inter-subset-abc123.woff2");
        assert!(link.contains("rel=\"preload\""));
        assert!(link.contains("as=\"font\""));
        assert!(link.contains("crossorigin"));
    }

    #[test]
    fn fallback_stack_known_font() {
        let stack = fallback_stack("Inter");
        assert!(stack.family_stack.contains("Inter fallback"));
        assert!(stack.size_adjust.is_some());
    }

    #[test]
    fn fallback_stack_unknown_font() {
        let stack = fallback_stack("CustomFont");
        assert!(stack.family_stack.contains("CustomFont"));
        assert!(stack.size_adjust.is_none());
    }

    #[test]
    fn fallback_font_face_has_metric_overrides() {
        let stack = fallback_stack("Inter");
        let css = fallback_font_face_css("Inter", &stack).unwrap();
        assert!(css.contains("size-adjust"));
        assert!(css.contains("ascent-override"));
        assert!(css.contains("descent-override"));
    }

    #[test]
    fn collect_font_usage_from_ir() {
        let ir: serde_json::Value = serde_json::from_str(r#"{
            "root": {
                "node_id": "root",
                "children": [
                    {
                        "value_type": "TextNode",
                        "value": {
                            "node_id": "t1",
                            "content": "Hello",
                            "font_family": "Inter",
                            "font_weight": "Bold"
                        }
                    }
                ]
            }
        }"#).unwrap();

        let usage = collect_font_usage(&ir);
        let inter = usage.families.get("Inter").unwrap();
        assert!(inter.codepoints.contains(&('H' as u32)));
        assert!(inter.codepoints.contains(&('e' as u32)));
        assert!(inter.weights.contains(&700));
        assert!(inter.above_fold);
    }

    #[test]
    fn content_hash_deterministic() {
        let data = b"font data here";
        let h1 = font_content_hash(data);
        let h2 = font_content_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 6);
    }
}
