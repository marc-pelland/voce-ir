# Voce Cross-Target Compatibility Matrix

**Scope:** semantic parity (meaning preserved), not pixel parity.
**Source of truth:** `SemanticSummary::from_ir` (validator lib).
**Verifier:** `packages/validator/tests/cross_target_parity.rs`,
run over the 13-fixture corpus in `tests/fixtures/`.
**Status:** S68 Slice 1–2 (HTML-family). SwiftUI / Compose / WASM rows
land in Slice 3.

## Legend

| Mark | Meaning |
| --- | --- |
| ✓ | Full parity — dimension preserved for every corpus fixture |
| ◐ | Degraded — the medium legitimately cannot preserve this; documented, not a bug |
| ✗ | Not applicable via this lens — needs a different extractor |
| ⚠ | Silent gap — a real divergence treated as a bug (Deliverable 5 ticket) |

## Matrix (semantic dimensions × target)

| Dimension | DOM | Hybrid | Email | WebGPU | SwiftUI | Compose | WASM |
| --- | :-: | :-: | :-: | :-: | :-: | :-: | :-: |
| Heading order & levels | ✓ | ✓ | ✓ | ✗¹ | — | — | — |
| Named media (alt) | ✓ | ✓ | ✓ | ✗¹ | — | — | — |
| Form fields | ✓ | ✓ | ◐² | ✗¹ | — | — | — |
| Links (href) | ✓ | ✓ | ✓³ | ✗¹ | — | — | — |
| Gestures (JS) | ✓ | ✓ | ◐⁵ | ✗¹ | — | — | — |
| Landmark roles | ✓ | ✓ | ◐⁴ | ✗¹ | — | — | — |

`—` = not yet measured (Slice 3). DOM and Hybrid are asserted at the
full-preservation contract; Email is asserted only on the dimensions it
is *required* to preserve (headings, named media).

## Notes

1. **WebGPU — wrong lens, not a failure.** The WebGPU target paints the
   UI on the GPU behind a fixed HTML shell (the scraper sees a constant
   `h=[2]` regardless of input). HTML-scraped parity is meaningless for
   it; semantic parity requires extracting from an accessibility-tree
   fallback. Tracked for a dedicated Slice-3 extractor. WebGPU compile
   success is smoke-covered in `cross_target_tests.rs`.

2. **Email forms — medium limitation (◐).** Most email clients strip or
   block `<form>` and inputs; the Email target does not emit them. This
   is the medium's nature, documented and expected.

3. **Email interactive links — fixed (✓, was ⚠).** The Email compiler
   silently dropped `href` on both TextNode and Surface, flattening all
   links to plain text (`links-and-nav`: 4 links → 0 anchors).
   **Deliverable 5 resolved:** both paths now emit email-safe anchors
   (`<a href>` with `target`/`rel` matching the DOM compiler; Surface
   links wrap the block in a `display:block` anchor). `links-and-nav`
   now emits 4 anchors, matching the IR contract, and the Email profile
   asserts interactive parity so it cannot regress.

4. **Email landmarks — medium limitation (◐).** Email layout is
   table-based with no semantic landmark elements/roles; their absence
   is expected for the medium.

5. **Email gestures — medium limitation (◐).** Gesture handlers are
   JS-driven; email has no scripting, so `gesture-tap`'s handler does
   not survive. This is the medium's nature, not a bug — and is why
   `SemanticSummary` tracks links and gestures separately: conflating
   them would have hidden the legitimate gesture degradation behind
   the (now correct) link parity.

## How to regenerate

```
cargo test -p voce-validator --test cross_target_parity
# divergence dump:
cargo test -p voce-validator --test cross_target_parity \
  diagnostic_html_family_dump -- --ignored --nocapture
```

A regression in a **✓** or required-**◐**-contract cell fails CI. The
**⚠** cell is a tracked bug, not a gate, until Deliverable 5 resolves it.
