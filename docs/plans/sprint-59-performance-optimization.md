# Sprint 59 — Performance Optimization

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Optimize compilation speed, output size, and AI generation latency across the full pipeline. Profile the Rust compiler, add caching layers, and minimize HTML emission. Targets: <1s compilation, <5KB for simple pages, <3s AI generation.

**Depends on:** Sprint 58 (production build provides real-world baseline measurements)

---

## Deliverables

- **Rust compiler profiling:**
  - Profile `compiler-dom` with `cargo flamegraph` on reference landing page
  - Identify and fix top 5 hotspots
  - Add `#[inline]` hints for hot-path functions in codegen
  - Evaluate `rayon` for parallel node compilation (Container children can compile independently)
- **Compilation caching:**
  - Content-addressed cache in `.voce/cache/` — hash IR subtrees, skip recompilation of unchanged nodes
  - Cache invalidation on schema version change or compiler version change
  - `--no-cache` flag to force full recompilation
  - Cache hit/miss reporting in `--verbose` output
- **Output size optimization:**
  - CSS deduplication: collect identical style declarations, emit shared classes
  - HTML minification: remove unnecessary whitespace, collapse boolean attributes
  - Critical CSS extraction: inline above-the-fold styles, defer remainder
  - Dead code elimination: only emit CSS custom properties that are actually referenced
  - Shared component detection: identical subtrees compiled once, referenced by class
- **AI generation optimization** (TypeScript):
  - Prompt caching: hash prompt text, return cached IR for identical prompts
  - Streaming IR: begin validation as soon as partial IR is available
  - Token budget optimization: measure token usage per node type, identify verbose patterns
  - Parallel generation for multi-page sites (generate pages concurrently)
- **Benchmark suite:** `benches/` directory using `criterion` crate
  - `bench_compile_landing_page` — full compilation of reference landing page
  - `bench_compile_minimal` — single TextNode compilation
  - `bench_validate` — validation of reference landing page
  - `bench_serialize_deserialize` — FlatBuffers round-trip
- **Performance regression CI:** benchmark results tracked, PR fails if compilation regresses by > 10%

## Acceptance Criteria

- [ ] Reference landing page compiles in under 1 second (measured by `criterion` benchmark, median of 100 runs)
- [ ] Single TextNode "Hello World" IR compiles to under 5KB HTML output
- [ ] Reference landing page HTML output is under 50KB (before gzip)
- [ ] Reference landing page HTML output is under 15KB (gzipped)
- [ ] Cache hit skips compilation and returns result in under 50ms
- [ ] AI generation for a simple card component completes in under 3 seconds (measured end-to-end including API call)
- [ ] CSS deduplication reduces stylesheet size by at least 30% on reference landing page
- [ ] `criterion` benchmark suite runs as part of `cargo bench` with no failures
- [ ] Flamegraph generated and saved as `docs/perf/compiler-flamegraph.svg`
- [ ] No performance regression greater than 10% compared to Sprint 58 baseline
- [ ] All existing tests continue to pass
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
