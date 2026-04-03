# Sprint 17 — Asset Pipeline

**Status:** Planned
**Goal:** Build the image optimization and font subsetting pipeline using Rust crates. Generate responsive image variants (AVIF/WebP), blur placeholders, preload hints, and lazy loading. After this sprint, MediaNode compilation produces optimized, responsive images with appropriate loading strategies.
**Depends on:** Sprint 16 (forms/data — asset pipeline is independent but sequenced after interactive features)

---

## Deliverables

1. Image decoding and format detection (PNG, JPEG, WebP, AVIF, GIF, SVG)
2. Responsive variant generation at [320, 640, 768, 1024, 1280, 1920] widths
3. AVIF and WebP encoding via Rust crates
4. BlurHash placeholder generation
5. Font subsetting for used glyphs only
6. Above-fold detection → preload hints in `<head>`
7. Lazy loading for below-fold images

---

## Tasks

### 1. Image Processor (`assets/image.rs`)

Core image processing pipeline using Rust crates:
