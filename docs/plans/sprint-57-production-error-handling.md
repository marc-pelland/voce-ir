# Sprint 57 — Production Error Handling

**Phase:** 7 — Production Readiness
**Status:** Planned
**Goal:** Harden error handling across the full Voce IR pipeline for production use. Implement graceful failures with helpful messages, recovery from partial AI generation, timeout handling, rate limiting for API calls, and structured error reporting.

**Depends on:** Sprint 54 (integration tests provide regression safety net)

---

## Deliverables

- **Unified error taxonomy** in `packages/schema/src/errors.rs`:
  - `VoceError` enum covering: `SchemaError`, `ValidationError`, `CompilationError`, `DeploymentError`, `PipelineError`, `AiBridgeError`
  - Every variant includes: error code (e.g., `V001`), human-readable message, source location (node path in IR), suggestion for fix
  - `thiserror` derives for all library errors, `anyhow` context at CLI boundaries
- **Validator error improvements:**
  - Replace all string error messages with structured `ValidationDiagnostic { code, severity, node_path, message, suggestion }`
  - Severity levels: Error (blocks compilation), Warning (emits but flags), Info (informational)
  - Collect all errors in a single pass instead of failing on first (report up to 50 diagnostics)
- **Compiler resilience:**
  - Graceful degradation: if a node fails to compile, emit a visible error placeholder in output (`<div class="voce-error">`) instead of crashing
  - Partial output: compile what succeeds, report what failed
  - Timeout per node: 5 second max per node compilation (guards against infinite loops in computed styles)
- **AI bridge hardening** (TypeScript):
  - Retry with exponential backoff (3 attempts, 1s/2s/4s)
  - Rate limiting: token bucket (60 requests/minute default, configurable)
  - API key rotation: accept multiple keys, rotate on 429 responses
  - Partial generation recovery: if AI returns incomplete IR, validate what exists, report truncation
  - Timeout: 30 second default for generation calls, configurable
  - Streaming progress callback for long generations
- **CLI error presentation:**
  - Colored, formatted error output (using `miette` crate for rich diagnostics)
  - `--json-errors` flag for machine-readable error output
  - Exit codes: 0 success, 1 validation error, 2 compilation error, 3 deployment error, 4 AI bridge error
- **Logging:** structured logging via `tracing` crate, `--verbose` flag for debug output, `VOCE_LOG` env var

## Acceptance Criteria

- [ ] Every error produced by the CLI includes an error code, message, and actionable suggestion
- [ ] Validator reports all diagnostics in a single pass (tested: IR with 5 errors reports all 5)
- [ ] Compiler emits partial output with error placeholders for failed nodes (tested: 1 bad node in 10 does not crash)
- [ ] AI bridge retries on transient failures (tested: mock 503 response triggers retry, succeeds on attempt 2)
- [ ] AI bridge respects rate limits (tested: 61st request in a minute is delayed, not rejected)
- [ ] API key rotation works (tested: 429 on key A switches to key B)
- [ ] `--json-errors` flag produces valid JSON array of error objects
- [ ] Exit codes are correct for each error category
- [ ] `--verbose` flag produces structured trace output
- [ ] No `unwrap()` calls remain in any library crate (verified by grep)
- [ ] All existing tests continue to pass
- [ ] `cargo clippy --workspace -- -D warnings` passes clean
