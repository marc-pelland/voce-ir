# Sprint 51 — Real Image Processing Pipeline

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Add actual AVIF/WebP generation using Rust image processing crates. Process source images at compile time, generate responsive variants, and emit correct `<picture>` markup referencing real optimized files.

**Depends on:** Compiler-DOM asset pipeline (Phase 4), MediaNode schema

---

## Deliverables

- Add `image`, `fast_image_resize`, `ravif`, `webp`, and `blurhash` crates to compiler-dom
- `ImagePipeline` struct that accepts a source image path and produces:
  - AVIF variant (quality 60, effort 4)
  - WebP variant (quality 75)
  - Original format fallback
  - Responsive widths: 320, 640, 1024, 1440, 1920
  - BlurHash placeholder string (embedded inline as CSS background)
- `PictureEmitter` that generates correct `<picture>` with `<source>` elements, `srcset` with width descriptors, and `sizes` attribute derived from layout context
- Asset manifest (`assets.json`) listing all generated files with content hashes for cache busting
- Integration with existing `MediaNode` compilation — when `src` points to a local file, run through pipeline; when URL, pass through unchanged
- `--skip-images` flag on `voce compile` to bypass processing during development
- Unit tests for each format conversion, resize operation, and markup emission
- Integration test: compile reference landing page with real hero image, verify all variants exist and markup is correct

## Acceptance Criteria

- [ ] `cargo build -p compiler-dom` compiles with image processing crates
- [ ] Given a 2400x1600 JPEG source, pipeline emits 5 AVIF files, 5 WebP files, 5 JPEG files (15 total)
- [ ] Generated `<picture>` element contains correct `<source type="image/avif">` and `<source type="image/webp">` with proper `srcset`
- [ ] BlurHash placeholder renders as inline CSS gradient (no additional network request)
- [ ] Asset manifest includes content-hash filenames (e.g., `hero-a1b2c3.avif`)
- [ ] `--skip-images` flag produces markup with original `src` unchanged
- [ ] Output AVIF files are smaller than equivalent WebP files
- [ ] Processing a 5MB source image completes in under 3 seconds on M1
- [ ] All existing tests continue to pass
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
