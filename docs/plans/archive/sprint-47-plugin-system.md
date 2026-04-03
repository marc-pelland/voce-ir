# Sprint 47 — Plugin System

**Status:** Planned
**Phase:** 6 (Ecosystem & Community)
**Depends on:** S44 (v0.5.0 inspector), S38 (v0.4.0 compilation)

---

## Goal

Build a plugin system that allows the community to create custom validator passes, custom compile targets, and content adapters without forking the core.

---

## Deliverables

- Plugin API specification: trait-based interface for Rust plugins
- Custom validator pass plugins: implement ValidationPass trait, register via config
- Custom compile target plugins: implement CompileTarget trait, register via config
- Content adapter plugins: custom data source integrations (CMS, API, database)
- Plugin discovery: voce.config.toml plugin section with local path or registry URL
- Plugin registry: central index of community plugins (initially a JSON file in repo)
- Plugin scaffold: `voce plugin init --type validator|compiler|adapter` CLI command
- Plugin testing harness: run plugins against standard test IR
- Documentation: plugin development guide, API reference, example plugins
- 3 example plugins: custom lint rule, Markdown compile target, RSS adapter

---

## Acceptance Criteria

- [ ] Custom validator pass loads and runs from plugin configuration
- [ ] Custom compile target produces output from plugin
- [ ] `voce plugin init` scaffolds a working plugin project
- [ ] Plugin testing harness validates plugin against standard IR
- [ ] 3 example plugins compile, install, and run correctly
- [ ] Plugin API is versioned and documented
- [ ] Breaking plugin API changes are detected at compile time
