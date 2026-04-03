# Voce IR — Animation, Asset Pipeline & Deployment

**Date:** 2026-04-02
**Status:** Living document
**Purpose:** Define how Voce IR handles animation compilation, image/font/media optimization, and deployment to hosting platforms.

---

## 1. Animation Compilation Strategy

### 1.1 Decision: Hybrid (CSS + WAAPI + Minimal JS)

The compiler uses a tiered approach — each animation uses the lightest technique that achieves the desired quality:

| Tier | Technology | JS Cost | Use Case |
| ---- | ---------- | ------- | -------- |
| **1 — CSS** | `transition`, `@keyframes`, `animation-timeline` | 0 bytes | Hover states, entrance animations, scroll-driven, spring via `linear()` |
| **2 — WAAPI** | `Element.animate()` + `.finished` promises | ~100-300B | Choreographed sequences, state machine transitions, programmatic scroll |
| **3 — rAF JS** | `requestAnimationFrame` | ~200-500B | Interruptible springs, gesture tracking, momentum/friction |
| **4 — View Transitions** | `document.startViewTransition()` | ~80-120B | Page/route transitions, shared element morphing |

**The compiler only emits code for features actually used.** A page with only hover states gets zero JS. A page with springs + gestures + choreography gets ~500-900B. No fixed runtime overhead.

### 1.2 The Key Innovation: Compile-Time Spring Curves

The CSS `linear()` function (Chrome 113+, Firefox 112+, Safari 17.2+ — ~95% support by 2026) accepts arbitrary easing curves as point arrays. The Voce IR compiler solves spring ODEs at compile time in Rust and emits pre-computed `linear()` curves:

```css
.voce-btn:active {
  transform: scale(0.95);
  transition: transform 280ms linear(0, 0.013, 0.049, 0.104, 0.175,
    0.259, 0.352, 0.45, 0.549, 0.645, 0.735, 0.816, 0.886, 0.942,
    0.984, 1.01, 1.025, 1.029, 1.025, 1.017, 1.008, 1.002, 1);
}
```

Spring physics with zero JS. 15-25 points per curve (~150-250 bytes CSS). Only when a spring must be interrupted mid-flight (velocity-aware) does the compiler emit a ~200B rAF stepper.

### 1.3 Compilation Patterns per IR Node

**Transition (hover/focus/active):** Pure CSS `transition` on pseudo-class. Zero JS, always.

**Transition (state machine change):** CSS classes + ~60B JS for class toggling. The transition itself is CSS.

**Sequence (staggered entrance):** CSS `@keyframes` with `animation-delay` per element (when count is known at compile time). For dynamic counts: WAAPI with `.finished` promise chaining (~150B).

**ScrollBinding:** CSS `animation-timeline: view()` / `scroll()` (~90%+ support). Optional IntersectionObserver fallback (~130B JS) for broad compatibility.

**PhysicsBody (spring):** Pre-computed `linear()` CSS easing by default (0B JS). rAF stepper (~200B) only when `interruptible: true`.

**GestureHandler (drag/continuous):** Must be JS — pointer events + transform updates (~250-350B).

**RouteTransition:** View Transitions API (~80B JS) with instant-navigation fallback.

**ReducedMotion:** Targeted `@media (prefers-reduced-motion: reduce)` overrides per animation. Each IR ReducedMotion node specifies the fallback; the compiler emits the corresponding CSS override.

### 1.4 Size Budget: A Real Landing Page

Hero entrance + hover states + scroll reveals + smooth scroll + spring button + page choreography + reduced motion:

| Component | Size |
| --------- | ---- |
| CSS for all animations | ~400-500B gzipped |
| JS (typical page) | ~150-250B min+gz |
| JS (complex page with gestures + springs) | ~500-900B min+gz |
| **Total animation overhead** | **550-1400B** |

For comparison: GSAP = ~28KB, Framer Motion = ~30-45KB, Motion One = ~3.8KB. The hybrid approach is 6-200x smaller.

### 1.5 What Makes Motion Feel "Quality"

Research across Apple, Stripe, Linear, Vercel, and award-winning sites:

- **Easing:** Spring for user-triggered motion (button press, menu open). Cubic-bezier for system-triggered (entrance, scroll). Never linear.
- **Spring parameters:** Stiffness 300-500, damping 25-35, mass 1. Quick with subtle overshoot — physical, not bouncy.
- **Stagger:** 30-80ms between elements (50ms most common). Top-to-bottom for vertical, left-to-right for horizontal.
- **Duration:** Micro-interactions 100-200ms, small transitions 200-350ms, medium 300-500ms, page transitions 400-700ms. Never >800ms.
- **Entrances are slower than exits** (300ms in, 200ms out). Entrances use ease-out, exits use ease-in.
- **Compositor-safe properties only:** `transform` and `opacity` for guaranteed 60fps. Never animate `width`, `height`, `margin`, `padding`.

---

## 2. Asset Pipeline

### 2.1 Image Optimization (Gatsby Model, Build-Time)

The compiler takes the best quality source image and generates optimized variants:

**For each MediaNode with type IMAGE:**

1. **Decode** source (PNG, JPEG, WebP, AVIF, TIFF, SVG, GIF)
2. **Generate responsive variants** at widths [320, 640, 768, 1024, 1280, 1920] in:
   - AVIF (quality 60-70 for photos)
   - WebP (quality 75-80)
   - Original format as fallback
3. **Generate placeholder:** BlurHash (~20-30 byte string) or inline base64 blur (~200-400B)
4. **Calculate** aspect ratio, dominant color for CLS prevention
5. **Determine loading strategy** from layout position:
   - Above fold: `loading="eager"`, `fetchpriority="high"`, preload in `<head>`
   - Below fold: `loading="lazy"`, blur-up reveal on load

**Compiled output for hero image:**
```html
<picture>
  <source type="image/avif" sizes="100vw"
    srcset="/assets/hero-320w.avif 320w, /assets/hero-640w.avif 640w,
           /assets/hero-1024w.avif 1024w, /assets/hero-1920w.avif 1920w">
  <source type="image/webp" sizes="100vw"
    srcset="/assets/hero-320w.webp 320w, ...">
  <img src="/assets/hero-1920w.jpg" alt="..." width="1920" height="1080"
       loading="eager" fetchpriority="high" decoding="async">
</picture>
```

Plus preload hint in `<head>` for the hero image.

**Rust crates for image processing:**

| Task | Crate | Notes |
| ---- | ----- | ----- |
| Decode/encode | `image` | Broad format support, mature |
| Resize | `fast_image_resize` | SIMD-accelerated, Lanczos3 |
| WebP | `webp` | FFI to libwebp |
| AVIF | `ravif` | Pure Rust (slower) or `libavif-sys` (faster, FFI) |
| BlurHash | `blurhash` | Generate placeholder strings |
| SVG | `resvg` | Rasterize SVGs at any resolution |
| Dominant color | `color_thief` | Palette extraction for placeholder backgrounds |

Start with pure Rust crates (simple, cross-platform). Optimize with libvips FFI if build times become a pain point for large sites.

### 2.2 Image CDN for Dynamic Content

ContentSlot images (from CMS, not known at build time) use CDN URL rewriting:

```toml
# voce.config.toml
[media.cdn]
provider = "cloudinary"  # or "imgix", "cloudflare"
base_url = "https://res.cloudinary.com/your-account/image/upload"
```

The compiler emits CDN URLs with transformation parameters:
```
https://res.cloudinary.com/demo/image/upload/f_auto,q_auto,w_640/{image_path}
```

`f_auto` and `q_auto` let the CDN choose optimal format and quality per browser.

### 2.3 Font Optimization

1. **Subset** to only glyphs used on the page (180KB → 24KB typical savings)
2. **Convert to WOFF2** (Brotli-compressed)
3. **`font-display: swap`** for body text, `font-display: optional` for decorative
4. **Preload** critical fonts in `<head>`
5. **Variable fonts** when multiple weights needed (one file, all weights)

Rust: `allsorts` for font parsing/subsetting, or shell out to `pyftsubset`.

### 2.4 Video/Audio

Don't transcode (too heavy for a compiler). Instead:
- Extract poster frame at ~2s (via `ffmpeg-next` Rust crate)
- Validate web-compatible format (warn if not MP4 H.264 or WebM)
- Emit `<video>` with `preload="metadata"`, lazy loading for below-fold
- Run poster frame through the image optimization pipeline

---

## 3. Deployment Strategy

### 3.1 IR-Driven Target Detection

The compiler analyzes the IR to determine the minimum deployment capability:

| IR Feature | Static | Edge | Server |
| ---------- | ------ | ---- | ------ |
| Static content, MediaNode, ThemeNode | Yes | Yes | Yes |
| ContentSlot (static cache) | Yes | Yes | Yes |
| ContentSlot (ISR) | No | **Required** | Yes |
| ActionNode (mutations) | Degraded* | **Required** | Yes |
| AuthContextNode | No | **Required** | Yes |
| Dynamic routes (/product/:id) | No | **Required** | Yes |
| SubscriptionNode (WebSocket) | No | No | **Required** |

*ActionNode on static: degrades to client-side fetch to user-provided external API.

```
voce deploy

Analyzing IR...
✓ 3 static routes
✓ 12 MediaNodes (build-time optimization)
⚡ 1 ActionNode (contact form) → requires edge functions
⚡ 2 ContentSlots with ISR → requires edge functions

Minimum target: Edge
Compatible: Vercel, Cloudflare, Netlify
```

### 3.2 Adapter Architecture (SvelteKit Model)

Rust trait-based adapter system:

```rust
pub trait DeployAdapter {
    fn name(&self) -> &str;
    fn validate(&self, analysis: &IrAnalysis) -> AdapterValidation;
    fn adapt(&self, build: &BuildOutput, dest: &Path) -> Result<AdaptedOutput>;
    fn generate_config(&self, build: &BuildOutput, dest: &Path) -> Result<()>;
    fn deploy(&self, output: &AdaptedOutput, opts: &DeployOptions) -> Result<DeployResult>;
}
```

**Built-in adapters:**

| Adapter | Output | Best For |
| ------- | ------ | -------- |
| `adapter-static` | HTML + assets in `dist/` | Landing pages, docs, portfolios |
| `adapter-cloudflare` | Pages + Workers | Best price/performance for edge |
| `adapter-vercel` | `.vercel/output/` + serverless functions | Best DX, built-in image CDN |
| `adapter-netlify` | `publish/` + functions | Good all-rounder |
| `adapter-node` | Rust binary + Dockerfile | Full server, WebSocket, self-hosted |

Edge functions are generated as JavaScript initially (widest compatibility). WASM compilation is a future optimization.

### 3.3 The `voce deploy` Experience

```bash
# First deploy — interactive setup
voce deploy
# → Analyzes IR, recommends platform, authenticates, builds, deploys
# → Returns preview URL

# Subsequent deploys
voce deploy          # Preview URL
voce deploy --prod   # Production

# Configuration
# voce.config.toml
[deploy]
default_target = "cloudflare"
```

Key principles:
- **Build locally, upload artifacts** — never send IR to the platform's build system
- **Preview by default** — `--prod` is explicit
- **IR analysis shown** — the user understands why a target was recommended
- **Warn on capability mismatch** — if user picks static but IR needs edge, explain clearly

### 3.4 Output Structure

**Static:**
```
dist/
├── index.html
├── about/index.html
├── assets/
│   ├── images/ (optimized AVIF/WebP at multiple sizes)
│   ├── fonts/ (subsetted WOFF2)
│   └── scripts/ (content-hashed, only if interactive nodes exist)
├── sitemap.xml
├── robots.txt
└── _headers (security headers for Cloudflare/Netlify)
```

**Edge:**
```
dist/
├── static/ (same as above)
├── functions/
│   ├── _middleware.js (auth, headers)
│   ├── api/actions/ (ActionNode handlers)
│   └── isr/ (ContentSlot revalidation)
└── platform.config (vercel.json / wrangler.toml / netlify.toml)
```

Content-hashed filenames for immutable caching on assets. HTML gets `must-revalidate`.

---

## 4. Phase Mapping

### Phase 1 (Schema + Validator)
- Define animation node schemas with easing/spring parameters sufficient for compile-time curve computation
- Define MediaNode with dimensions, alt, loading strategy, format hints

### Phase 2 (DOM Compiler)
- Implement tiered animation compilation (CSS → WAAPI → rAF)
- Implement spring ODE solver in Rust for `linear()` curve generation
- Implement image optimization pipeline (Rust crates)
- Implement font subsetting
- Implement `adapter-static` and `adapter-cloudflare`
- Implement `voce deploy` CLI

### Phase 3 (AI Bridge + Advanced)
- Image CDN URL rewriting for ContentSlot images
- `adapter-vercel` and `adapter-netlify`
- `adapter-node` for WebSocket/server deployments
- Edge function generation for ActionNode/ISR

---

*This document should be read alongside the other research docs in `docs/research/`.*
