//! Content Security Policy assembly for the DOM compiler.
//!
//! S70 Day 1 hardening:
//!
//! - `'unsafe-inline'` is dropped from `script-src`. Every inline script the
//!   compiler emits (interactive JS, JSON-LD blocks) gets a SHA-256 hash that
//!   becomes a `'sha256-...'` source. External scripts are still forbidden
//!   (no `'self'` origin in `script-src` would matter — there are none).
//! - `'unsafe-inline'` is **kept** for `style-src`. The compiler emits a
//!   per-element inline `style` attribute on most nodes; switching to hashes
//!   would multiply the CSP size by O(node count) and require a CSP per page.
//!   Worth revisiting (S70 follow-up) if/when the compiler moves to a class-
//!   based emit. Documented here so the rationale isn't lost.
//! - Three new directives: `frame-ancestors 'none'`, `base-uri 'self'`,
//!   `form-action 'self'`. These are zero-cost defenses that should always
//!   be on for static output.
//! - A per-IR override (`PageMetadata.content_security_policy`) replaces the
//!   default policy entirely — for projects with stricter requirements (or
//!   experiments) without needing to fork the compiler.
//
// Hashes are over the *exact bytes* between `<script>` and `</script>`, NOT
// including the surrounding tags. The compiler's emit path is the canonical
// source: any change to whitespace around a script body changes its hash.

use base64::Engine;
use sha2::{Digest, Sha256};

/// Default CSP when no override is supplied. `script_hashes` are base64-encoded
/// SHA-256 hashes of each inline script body, in the form `'sha256-...'`.
pub fn build_default(script_hashes: &[String]) -> String {
    let script_src = if script_hashes.is_empty() {
        "script-src 'self'".to_string()
    } else {
        let hashes = script_hashes
            .iter()
            .map(|h| format!("'sha256-{h}'"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("script-src 'self' {hashes}")
    };

    [
        "default-src 'self'",
        script_src.as_str(),
        "style-src 'self' 'unsafe-inline'",
        "img-src 'self' https: data:",
        "frame-ancestors 'none'",
        "base-uri 'self'",
        "form-action 'self'",
    ]
    .join("; ")
}

/// Choose the policy that lands in the `<meta http-equiv="Content-Security-Policy">`
/// tag. An explicit override always wins; otherwise `build_default` is used.
pub fn resolve(override_csp: Option<&str>, script_hashes: &[String]) -> String {
    match override_csp {
        Some(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => build_default(script_hashes),
    }
}

/// SHA-256 hash of `script`, base64-encoded. Suitable for inclusion in a
/// `'sha256-...'` CSP source. Hashes are over the exact bytes the compiler
/// will emit between `<script>` and `</script>`.
pub fn hash_script(script: &str) -> String {
    let mut h = Sha256::new();
    h.update(script.as_bytes());
    let digest = h.finalize();
    base64::engine::general_purpose::STANDARD.encode(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_csp_includes_new_directives() {
        let csp = build_default(&[]);
        assert!(csp.contains("frame-ancestors 'none'"));
        assert!(csp.contains("base-uri 'self'"));
        assert!(csp.contains("form-action 'self'"));
    }

    #[test]
    fn default_csp_drops_unsafe_inline_for_scripts() {
        let csp = build_default(&[]);
        // script-src is the segment from "script-src" up to the next "; ".
        let segment = csp
            .split("; ")
            .find(|s| s.starts_with("script-src"))
            .expect("script-src present");
        assert!(!segment.contains("'unsafe-inline'"), "got: {segment}");
    }

    #[test]
    fn style_src_keeps_unsafe_inline_with_documented_rationale() {
        // Per-element inline styles are pervasive in compiler output; keeping
        // 'unsafe-inline' here is intentional. If the compiler ever moves to
        // class-based styling this should become a hash list.
        let csp = build_default(&[]);
        let segment = csp
            .split("; ")
            .find(|s| s.starts_with("style-src"))
            .expect("style-src present");
        assert!(segment.contains("'unsafe-inline'"));
    }

    #[test]
    fn each_script_hash_appears_once_with_sha256_prefix() {
        let h1 = hash_script("alert(1)");
        let h2 = hash_script("alert(2)");
        let csp = build_default(&[h1.clone(), h2.clone()]);
        assert!(csp.contains(&format!("'sha256-{h1}'")));
        assert!(csp.contains(&format!("'sha256-{h2}'")));
    }

    #[test]
    fn hash_is_stable_for_identical_input() {
        assert_eq!(hash_script("hello"), hash_script("hello"));
        assert_ne!(hash_script("hello"), hash_script("HELLO"));
    }

    #[test]
    fn override_replaces_default_entirely() {
        let custom = "default-src 'none'; script-src 'self'";
        let resolved = resolve(Some(custom), &["x".to_string()]);
        assert_eq!(resolved, custom);
    }

    #[test]
    fn override_falls_through_when_empty_or_whitespace() {
        let csp = resolve(Some("   "), &[]);
        assert!(csp.contains("frame-ancestors 'none'"));
    }
}
