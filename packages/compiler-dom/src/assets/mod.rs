//! Asset pipeline — image optimization, font subsetting, media handling.
//!
//! Phase 1: Emit correct `<picture>` markup with responsive srcset.
//! Phase 2: Real image processing (WebP generation, resize, BlurHash).

pub mod font_pipeline;
#[cfg(feature = "image-pipeline")]
pub mod image_pipeline;

/// Standard responsive breakpoint widths for srcset generation.
pub const RESPONSIVE_WIDTHS: &[u32] = &[320, 640, 768, 1024, 1280, 1920];

/// Generate srcset attribute value from a source image path.
///
/// If the source has an extension, generates width-based variants:
/// `/images/hero.jpg` → `/images/hero-320w.jpg 320w, /images/hero-640w.jpg 640w, ...`
///
/// In Phase 2+, the actual image files will be generated during compilation.
/// For now, this generates the markup assuming the files exist.
pub fn generate_srcset(src: &str, widths: &[u32]) -> String {
    let (base, ext) = match src.rsplit_once('.') {
        Some((b, e)) => (b, e),
        None => return String::new(),
    };

    widths
        .iter()
        .map(|w| format!("{base}-{w}w.{ext} {w}w"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Generate a default sizes attribute for responsive images.
pub fn default_sizes() -> &'static str {
    "(max-width: 640px) 100vw, (max-width: 1024px) 75vw, 50vw"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srcset_generation() {
        let srcset = generate_srcset("/images/hero.jpg", &[320, 640, 1024]);
        assert_eq!(
            srcset,
            "/images/hero-320w.jpg 320w, /images/hero-640w.jpg 640w, /images/hero-1024w.jpg 1024w"
        );
    }

    #[test]
    fn srcset_no_extension() {
        let srcset = generate_srcset("noext", &[320]);
        assert!(srcset.is_empty());
    }
}
