# Sprint 19 — Testing & Reports

**Status:** Planned
**Goal:** Build automated testing infrastructure, report generation, and snapshot testing for the compiler. Add `voce test`, `voce report`, and `voce manifest` CLI commands. After this sprint, every compilation produces verifiable quality metrics and the compiler output is snapshot-tested for regression prevention.
**Depends on:** Sprint 18 (security, a11y, output quality — reports summarize these results)

---

## Deliverables

1. Auto-generated test report from IR analysis
2. `voce test` command: runs compiled output through quality checks
3. `voce report` command: full compilation report (perf, a11y, security)
4. `voce manifest` command: generates application manifest from IR
5. Snapshot testing with `insta` for compiler output
6. Integration tests: compile landing page IR → verify HTML structure
7. Benchmark: measure compilation time for reference landing page

---

## Tasks

### 1. Test Report Generation (`report/test_report.rs`)

Analyze the IR and compiled output to produce a test report:
