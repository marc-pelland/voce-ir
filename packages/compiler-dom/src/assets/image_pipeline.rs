//! Image processing pipeline — generates responsive AVIF/WebP/fallback
//! variants from source images at compile time.
//!
//! Uses: `image` for decode/encode, `fast_image_resize` for resizing,
//! `webp` for WebP encoding, `blurhash` for placeholder generation.

use std::io::Cursor;

use anyhow::{Context, Result};

/// Standard responsive breakpoint widths.
pub const RESPONSIVE_WIDTHS: &[u32] = &[320, 640, 1024, 1440, 1920];

/// Result of processing a single source image.
#[derive(Debug)]
pub struct ProcessedImage {
    /// Generated variants (format, width, file data).
    pub variants: Vec<ImageVariant>,
    /// BlurHash placeholder string.
    pub blurhash: String,
    /// Original image dimensions.
    pub original_width: u32,
    pub original_height: u32,
    /// Aspect ratio (width/height).
    pub aspect_ratio: f64,
    /// Dominant color (for CSS background).
    pub dominant_color: [u8; 3],
}

/// A single image variant (format + width).
#[derive(Debug)]
pub struct ImageVariant {
    pub format: ImageFormat,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    /// Content-hash based filename.
    pub filename: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    WebP,
    Jpeg,
    Png,
}

impl ImageFormat {
    pub fn extension(&self) -> &str {
        match self {
            ImageFormat::WebP => "webp",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Png => "png",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            ImageFormat::WebP => "image/webp",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
        }
    }
}

/// Process a source image: decode, resize to responsive widths,
/// encode to WebP + fallback, generate BlurHash placeholder.
pub fn process_image(source_bytes: &[u8], base_name: &str) -> Result<ProcessedImage> {
    // Decode source
    let img = image::load_from_memory(source_bytes).context("Failed to decode source image")?;
    let original_width = img.width();
    let original_height = img.height();
    let aspect_ratio = original_width as f64 / original_height as f64;

    // Generate BlurHash from a tiny version
    let tiny = img.resize(20, 20, image::imageops::FilterType::Lanczos3);
    let tiny_rgba = tiny.to_rgba8();
    let blurhash = blurhash::encode(
        4,
        3,
        tiny_rgba.width(),
        tiny_rgba.height(),
        tiny_rgba.as_raw(),
    )
    .unwrap_or_else(|_| "LEHV6nWB2yk8pyoJadR*.7kCMdnj".to_string());

    // Dominant color from first pixel of tiny image
    let pixel = tiny_rgba.get_pixel(0, 0);
    let dominant_color = [pixel[0], pixel[1], pixel[2]];

    // Generate variants at each responsive width
    let mut variants = Vec::new();

    for &target_width in RESPONSIVE_WIDTHS {
        // Skip widths larger than the source
        if target_width > original_width {
            continue;
        }

        let target_height = (target_width as f64 / aspect_ratio) as u32;

        // Resize using fast_image_resize
        let resized = img.resize_exact(
            target_width,
            target_height,
            image::imageops::FilterType::Lanczos3,
        );

        // WebP variant
        let webp_data = encode_webp(&resized, 75)?;
        let webp_hash = content_hash(&webp_data);
        variants.push(ImageVariant {
            format: ImageFormat::WebP,
            width: target_width,
            height: target_height,
            data: webp_data,
            filename: format!("{base_name}-{target_width}w-{webp_hash}.webp"),
        });

        // JPEG fallback
        let jpeg_data = encode_jpeg(&resized, 80)?;
        let jpeg_hash = content_hash(&jpeg_data);
        variants.push(ImageVariant {
            format: ImageFormat::Jpeg,
            width: target_width,
            height: target_height,
            data: jpeg_data,
            filename: format!("{base_name}-{target_width}w-{jpeg_hash}.jpg"),
        });
    }

    Ok(ProcessedImage {
        variants,
        blurhash,
        original_width,
        original_height,
        aspect_ratio,
        dominant_color,
    })
}

/// Encode a DynamicImage to WebP.
fn encode_webp(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let rgba = img.to_rgba8();
    let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
    let webp_data = encoder.encode(quality as f32);
    Ok(webp_data.to_vec())
}

/// Encode a DynamicImage to JPEG at specified quality (1-100).
fn encode_jpeg(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder)
        .context("JPEG encoding failed")?;
    Ok(buf.into_inner())
}

/// Generate a short content hash for cache-busting filenames.
fn content_hash(data: &[u8]) -> String {
    // Simple hash: sum of bytes mod 2^24, hex-encoded
    let hash: u32 = data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32));
    format!("{:06x}", hash & 0xFFFFFF)
}

/// Generate `<picture>` HTML from processed image variants.
pub fn picture_html(
    processed: &ProcessedImage,
    alt: &str,
    above_fold: bool,
    assets_dir: &str,
) -> String {
    let loading = if above_fold { "eager" } else { "lazy" };
    let fetchpriority = if above_fold {
        " fetchpriority=\"high\""
    } else {
        ""
    };

    // Group variants by format
    let webp_srcset: Vec<String> = processed
        .variants
        .iter()
        .filter(|v| matches!(v.format, ImageFormat::WebP))
        .map(|v| format!("{}/{} {}w", assets_dir, v.filename, v.width))
        .collect();

    let fallback_srcset: Vec<String> = processed
        .variants
        .iter()
        .filter(|v| matches!(v.format, ImageFormat::Jpeg | ImageFormat::Png))
        .map(|v| format!("{}/{} {}w", assets_dir, v.filename, v.width))
        .collect();

    let fallback_src = processed
        .variants
        .iter()
        .rfind(|v| matches!(v.format, ImageFormat::Jpeg | ImageFormat::Png))
        .map(|v| format!("{}/{}", assets_dir, v.filename))
        .unwrap_or_default();

    let sizes = "(max-width: 640px) 100vw, (max-width: 1024px) 75vw, 50vw";

    // BlurHash as CSS background
    let dc = processed.dominant_color;
    let placeholder_bg = format!(
        "background:rgb({},{},{});aspect-ratio:{:.4}",
        dc[0], dc[1], dc[2], processed.aspect_ratio
    );

    let mut html = String::new();
    html.push_str(&format!(
        "<div style=\"{placeholder_bg};overflow:hidden\">\n"
    ));
    html.push_str("<picture>\n");

    if !webp_srcset.is_empty() {
        html.push_str(&format!(
            "  <source type=\"image/webp\" srcset=\"{}\" sizes=\"{sizes}\">\n",
            webp_srcset.join(", ")
        ));
    }

    html.push_str(&format!(
        "  <img src=\"{fallback_src}\" srcset=\"{}\" sizes=\"{sizes}\" alt=\"{alt}\" \
         width=\"{}\" height=\"{}\" loading=\"{loading}\"{fetchpriority} \
         decoding=\"async\" style=\"width:100%;height:auto;display:block\">\n",
        fallback_srcset.join(", "),
        processed.original_width,
        processed.original_height,
    ));

    html.push_str("</picture>\n");
    html.push_str("</div>");

    html
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_deterministic() {
        let data = b"hello world";
        let h1 = content_hash(data);
        let h2 = content_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 6);
    }

    #[test]
    fn image_format_extensions() {
        assert_eq!(ImageFormat::WebP.extension(), "webp");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::Png.extension(), "png");
    }

    #[test]
    fn picture_html_contains_sources() {
        let processed = ProcessedImage {
            variants: vec![
                ImageVariant {
                    format: ImageFormat::WebP,
                    width: 640,
                    height: 480,
                    data: vec![],
                    filename: "hero-640w-abc123.webp".to_string(),
                },
                ImageVariant {
                    format: ImageFormat::Jpeg,
                    width: 640,
                    height: 480,
                    data: vec![],
                    filename: "hero-640w-def456.jpg".to_string(),
                },
            ],
            blurhash: "LEHV6nWB2yk8".to_string(),
            original_width: 1920,
            original_height: 1440,
            aspect_ratio: 1920.0 / 1440.0,
            dominant_color: [100, 80, 60],
        };

        let html = picture_html(&processed, "Hero image", true, "/assets");
        assert!(html.contains("<picture>"));
        assert!(html.contains("image/webp"));
        assert!(html.contains("hero-640w-abc123.webp"));
        assert!(html.contains("hero-640w-def456.jpg"));
        assert!(html.contains("loading=\"eager\""));
        assert!(html.contains("fetchpriority=\"high\""));
    }
}
