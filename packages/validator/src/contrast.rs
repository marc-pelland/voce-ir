//! WCAG 2.2 contrast computation (S82 Day 1).
//!
//! Pure functions — no I/O, no IR types, no allocation beyond the return.
//! Easy to unit-test and to call from the A11Y007 validator pass.
//!
//! References:
//!   - WCAG 2.1 SC 1.4.3 (AA) and 1.4.6 (AAA) define contrast thresholds.
//!   - https://www.w3.org/WAI/WCAG22/Techniques/general/G18 has the
//!     canonical formula.
//!
//! The math:
//!   - sRGB channel → linear:  if c ≤ 0.03928, c/12.92, else ((c+0.055)/1.055)^2.4
//!   - relative luminance:     L = 0.2126*R + 0.7152*G + 0.0722*B
//!   - contrast ratio:         (L_lighter + 0.05) / (L_darker + 0.05)
//!
//! Inputs are 0-255 sRGB. We don't currently account for transparency
//! (alpha) — when a text color has a < 255 the validator skips the check
//! rather than guess at the composited result. Same for backgrounds with
//! partial transparency. Documented as a known gap.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const WHITE: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };

    /// Parse a Voce IR color JSON object: `{"r": u8, "g": u8, "b": u8, "a"?: u8}`.
    /// Returns `None` if any required field is missing or the alpha is
    /// less than 255 (we don't approximate compositing — see module docs).
    pub fn from_json(value: &serde_json::Value) -> Option<Rgb> {
        let r = value.get("r")?.as_u64()?;
        let g = value.get("g")?.as_u64()?;
        let b = value.get("b")?.as_u64()?;
        let a = value.get("a").and_then(|v| v.as_u64()).unwrap_or(255);
        if a < 255 {
            return None; // transparent — we don't composite
        }
        Some(Rgb {
            r: r.min(255) as u8,
            g: g.min(255) as u8,
            b: b.min(255) as u8,
        })
    }
}

/// Convert an sRGB channel byte (0-255) to a linear-light value (0.0-1.0).
fn srgb_to_linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance per WCAG: dot product of linear-light RGB with
/// the standard weights.
pub fn relative_luminance(c: Rgb) -> f64 {
    let r = srgb_to_linear(c.r);
    let g = srgb_to_linear(c.g);
    let b = srgb_to_linear(c.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Contrast ratio between two colors. Range is 1.0 (identical) to 21.0
/// (black on white). Symmetric: `contrast(a, b) == contrast(b, a)`.
pub fn contrast_ratio(a: Rgb, b: Rgb) -> f64 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (lighter, darker) = if la > lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// Whether text counts as "large" per WCAG. Large text is 18pt+ regular
/// OR 14pt+ bold. `font_size_px` is the CSS pixel size (1pt ≈ 1.333px).
pub fn is_large_text(font_size_px: f64, is_bold: bool) -> bool {
    let pt = font_size_px / 1.333;
    pt >= 18.0 || (is_bold && pt >= 14.0)
}

/// Required contrast ratio for AA conformance.
pub fn aa_threshold(large_text: bool) -> f64 {
    if large_text { 3.0 } else { 4.5 }
}

/// Required contrast ratio for AAA conformance. Stricter; not the default.
pub fn aaa_threshold(large_text: bool) -> f64 {
    if large_text { 4.5 } else { 7.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_on_white_is_21_to_1() {
        let r = contrast_ratio(Rgb::BLACK, Rgb::WHITE);
        // Floating-point — assert close, not equal.
        assert!((r - 21.0).abs() < 0.01, "got {r}");
    }

    #[test]
    fn identical_colors_are_1_to_1() {
        let pink = Rgb {
            r: 255,
            g: 192,
            b: 203,
        };
        let r = contrast_ratio(pink, pink);
        assert!((r - 1.0).abs() < 0.001);
    }

    #[test]
    fn contrast_is_symmetric() {
        let a = Rgb {
            r: 99,
            g: 102,
            b: 241,
        };
        let b = Rgb {
            r: 24,
            g: 24,
            b: 27,
        };
        assert!((contrast_ratio(a, b) - contrast_ratio(b, a)).abs() < 1e-9);
    }

    #[test]
    fn medium_grey_on_white_fails_aa_body() {
        // #999 on white is 2.85:1 — fails AA body (4.5) and AA large (3.0).
        let r = contrast_ratio(
            Rgb {
                r: 153,
                g: 153,
                b: 153,
            },
            Rgb::WHITE,
        );
        assert!(r < 4.5, "{r} should be below body AA threshold");
        assert!(r < 3.0, "{r} should be below large-text AA threshold");
    }

    #[test]
    fn voce_primary_on_dark_passes_aa_body() {
        // #818cf8 (primary-hover) on #0a0a0c (dark bg) — used in compiled output.
        let primary = Rgb {
            r: 0x81,
            g: 0x8c,
            b: 0xf8,
        };
        let dark_bg = Rgb {
            r: 0x0a,
            g: 0x0a,
            b: 0x0c,
        };
        let r = contrast_ratio(primary, dark_bg);
        assert!(r >= 4.5, "{r} must clear AA body threshold");
    }

    #[test]
    fn large_text_uses_relaxed_threshold() {
        // #767676 on white is 4.54:1 — clears AA body for large but not body normal.
        // Matches WCAG's published example numbers for borderline cases.
        let r = contrast_ratio(
            Rgb {
                r: 0x76,
                g: 0x76,
                b: 0x76,
            },
            Rgb::WHITE,
        );
        assert!(r >= aa_threshold(true), "{r} should clear large-text AA");
    }

    #[test]
    fn is_large_text_thresholds() {
        // 18pt = 24px regular: large.
        assert!(is_large_text(24.0, false));
        // 14pt = 18.67px bold: large.
        assert!(is_large_text(18.67, true));
        // 14pt regular: NOT large.
        assert!(!is_large_text(18.67, false));
        // 17.99pt regular (just under 18): NOT large.
        assert!(!is_large_text(23.99, false));
    }

    #[test]
    fn rgb_from_json_parses_full_alpha() {
        let v = serde_json::json!({"r": 99, "g": 102, "b": 241, "a": 255});
        assert_eq!(
            Rgb::from_json(&v),
            Some(Rgb {
                r: 99,
                g: 102,
                b: 241
            })
        );
    }

    #[test]
    fn rgb_from_json_parses_missing_alpha_as_opaque() {
        let v = serde_json::json!({"r": 99, "g": 102, "b": 241});
        assert!(Rgb::from_json(&v).is_some());
    }

    #[test]
    fn rgb_from_json_skips_partial_alpha() {
        let v = serde_json::json!({"r": 99, "g": 102, "b": 241, "a": 128});
        assert!(Rgb::from_json(&v).is_none());
    }

    #[test]
    fn aa_thresholds() {
        assert_eq!(aa_threshold(false), 4.5);
        assert_eq!(aa_threshold(true), 3.0);
    }

    #[test]
    fn aaa_thresholds() {
        assert_eq!(aaa_threshold(false), 7.0);
        assert_eq!(aaa_threshold(true), 4.5);
    }
}
