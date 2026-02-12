# Changelog

All notable changes to **Memory Context Browser** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

*No unreleased changes.*

---

## [0.2.1] - 2026-02-XX

### Summary

Pre-v0.3.0 consolidation release focused on v0.2.1 branch hardening: data-model-v2 merge, admin UI expansion, CI modernization, dead-code cleanup, MCP handler/provider consolidation, and documentation reconciliation.

### Fixed

- Session optionality and validation/doc alignment cleanup (`df781ce2`).
- Context stats and implicit-default behavior cleanup (`2e43b431`).
- Rustdoc/import hygiene and clippy/merge-fix follow-ups (`78699c98`, `be77523c`).
- Honesty and persistence integrity fixes with contextual error handling (`24cb7f83`).

### Added

- Admin UI capabilities: LOV endpoint system, filtered listing, entity dashboards, collapsible groups, advanced UI partials, and bulk actions (`ec2a952e`, `663a532e`, `807d43cc`, `4d5d1632`, `257f61c3`, `783f8370`).
- Data model v2 work merged into v0.2.1 (`9fe5a8f3`, `871f7d26`).
- Expanded provider/config support for SQLite/project context (`25348f3b`, `bed5a3f5`).

### Changed

- CI pipeline/workflow modernization and hardening (`2ba91d63`, `eb9e647d`, `f461c915`).
- Dead code and dependency reduction (`c3452805`, `9f36d616`, `787ec7a3`).
- Template engine and admin rendering consolidation toward Handlebars (`0c126590`, `b7b3c95d`, `4ca9cdd4`).
- Version stream consolidation: v0.2.2 track removed and folded into v0.2.1 (`e776cd86`).
- Documentation/model restructuring and ADR normalization (`f427d367`, `d76861af`, `1c8f309a`, `20eb9563`).

### Metrics

- Commits since v0.2.0: 63
- Rust Tests: 1,485 passing (`cargo test --workspace`)
- mcb-validate listed tests: 382 (`cargo test -p mcb-validate -- --list`)
- Crates: 9 workspace members
- Beads open issues triaged with scope tags: 50/50

---

## [0.2.0] - 2026-02-10

### Summary

Major stabilization release delivering critical bug fixes for Agent, Session, and Memory subsystems, strict MCP protocol compliance, SQLite DDL startup resilience, comprehensive test isolation, repository cleanup, filesystem reorganization, and a full documentation overhaul with rebranding to **Memory Context Browser**.

### Fixed

- **SQLite DDL Startup**: Replaced fragile FTS/trigger creation with `rebuild_fts_sqlite()` — drops and recreates FTS tables, triggers, and repopulates data. Eliminates `SQLITE_BUSY` failures on concurrent schema evolution.
- **SQLite Recovery**: Added backup-and-recreate recovery path when DDL operations fail on corrupted databases.
- **Agent SQL Storage**: Fixed "Failed to store tool call" error — correct repository dependency chain (Project -> Session -> Agent).
- **Session Schema**: Fallback logic for `agent_type` and `model` fields in session creation payloads.
- **Memory Enums**: Improved validation and error messages for `observation_type`.
- **Strict `tools/call` Validation**: Non-object `arguments` now return JSON-RPC `-32602` (Invalid Params) instead of being silently dropped.
- **Test Isolation**: All integration tests use `unique_test_config()` with unique temp DB paths per test, eliminating SQLite I/O contention during parallel execution.
- **Playwright E2E**: Fixed `test.describe()` not found error in CI by removing per-file spec iteration — Playwright now discovers all specs from `testDir`.
- **ADR Broken Links**: Fixed 3 cross-reference patterns across 4 ADRs (034/036/037/038).
- **Doctest Compilation**: Fixed registry module doctest return type.

### Added

- **Startup Smoke Tests**: New `startup_smoke_integration.rs` with process-spawning tests for DDL/init failure detection.
- **CI Startup Job**: `startup-smoke` job in GitHub Actions CI pipeline.
- **ADR Archive Structure**: `docs/adr/archive/` directory for superseded decisions.
- **YAML Frontmatter**: All 44+ ADRs now include standardized frontmatter.
- **Gap Regression Tests**: `gap_fixes_e2e.rs` for GAP-1, GAP-2, GAP-3 verification.
- **MCP Protocol Tests**: 3 integration tests for strict argument validation.
- **Systemd Hardening**: `StartLimitIntervalSec=90` + `StartLimitBurst=3` in service file.

### Changed

- **Rebranding**: "MCP Context Browser" renamed to "Memory Context Browser" across all 98+ occurrences (docs, templates, configs, README).
- **Filesystem Reorganization**: `docker-compose.yml`, `package.json`, `package-lock.json` moved to `tests/`. All Makefile, CI, and script references updated.
- **README Rewrite**: Complete professional rewrite (179 lines) with badges, feature overview, Quick Start, MCP Tools table, and ASCII architecture diagram.
- **Documentation Updates**: All docs updated to v0.2.0 (QUICKSTART, ARCHITECTURE, ENVIRONMENT_VARIABLES, docs/README).
- **ADR Status Standardization**: All ADRs use 5-value status set (IMPLEMENTED, ACCEPTED, PROPOSED, SUPERSEDED, ARCHIVED).
- **Legacy Removal**: Removed `/indexing` admin route (migrated to `/jobs`), deleted `ADMIN_SERVICE_DEFAULT_PORT` constant, removed tracked log artifacts.
- **Repository Cleanup**: 32 cruft files removed (reports, temp scripts, test logs, unused configs, screenshots).

### Metrics

- **Rust Tests**: 1,266 passing (0 failures, 11 ignored)
- **E2E Tests**: 44 passing (6 skipped for missing fixture data)
- **Total**: 3,143+ tests across unit, integration, golden, and E2E
- **Crates**: 9 workspace members
- **Integration Gaps**: 3/3 critical validation gaps closed
- **MCP Compliance**: Strict argument validation per JSON-RPC spec

---

## [0.1.5] - 2026-01-31

### Summary

New providers, health endpoints, and code quality improvements following DRY/SOLID principles.

### Added

- **Anthropic Embedding Provider**: Full Voyage AI model support with configurable dimensions.
- **Pinecone Vector Store Provider**: Production-ready cloud vector database integration.
- **Qdrant Vector Store Provider**: Self-hosted vector search with gRPC support.
- **Health Endpoints**: `/healthz` (liveness) and `/readyz` (readiness) for container orchestration.
- **Performance Metrics Decorator**: SOLID-compliant instrumented embedding provider.
- **Golden Test Framework**: Architecture boundary test scaffolding (ADR-027).

### Changed

- **DRY Refactoring**: Shared HTTP helpers across embedding/vector store providers (~200 lines deduplicated).
- **CI/CD**: Auto-merge Dependabot PRs (patch/minor), auto-tag on release branch merge.
- **Test Organization**: Inline tests moved to proper test directories.

### Fixed

- All architecture validation errors resolved (0 errors, 4 warnings).
- Validation service properly wired through DI system.

---

## [0.1.4] - 2026-01-28

### Summary

Rust-code-analysis integration, security fixes, and dependency updates.

### Added

- **RCA Integration**: Migrated `unwrap_detector.rs` to Rust-code-analysis Callback pattern.

### Changed

- **Dependencies**: uuid, clap, rust-rule-engine, jsonwebtoken, dirs, moka, chrono, thiserror, proc-macro2 updated.
- **Terminal Detection**: Replaced `atty` with `std::io::IsTerminal` (stable since Rust 1.70).

### Removed

- `atty` dependency (security advisory GHSA-g98v-hv3f-hcfr).
- Legacy AST executor code (240 lines).

### Security

- **GHSA-g98v-hv3f-hcfr**: Fixed by removing `atty` dependency.

---

## [0.1.3] - 2026-01-27

### Summary

Config consolidation and validation fixes. 16 config files reduced to 6, all 23 validation violations resolved.

---

## [0.1.2] - 2026-01-18

### Summary

Provider registration modernization (inventory -> linkme compile-time) and architecture validation crate (mcb-validate).

### Added

- **mcb-validate Crate**: Architecture validation with 12 migration rules, tree-sitter AST parsing, linter integration.
- **Linkme Registration**: All 15 providers migrated to compile-time distributed slices.
- **Admin UI Code Browser**: VectorStoreBrowser trait, 6 provider implementations, 3 UI pages with Prism.js highlighting.

### Changed

- Provider registration moved from runtime inventory to compile-time linkme (zero overhead).

---

## [0.1.0] - 2026-01-11

### Summary

First stable release — complete drop-in replacement for Claude-context with 14 languages processors, 6 embedding providers, 5 vector stores, systemd integration, and comprehensive documentation.

### Added

- 14 languages processors with AST parsing.
- HTTP transport foundation with session management.
- Binary auto-respawn, connection tracking, signal handling.
- Systemd user-level service integration.
- Migration guide from Claude-context.

---

## [0.0.3] - 2026-01-07

### Summary

Production foundation — circuit breaker, health checks, intelligent routing, Gemini/VoyageAI providers, encrypted vector storage.

---

## [0.0.2] - 2026-01-06

### Summary

Documentation architecture and development infrastructure (CI/CD, Makefile, testing foundation).

---

## [0.0.1] - 2026-01-06

### Summary

Architectural foundation — modular design, SOLID principles, provider framework, basic MCP protocol over stdio.

---

## Cross-References

- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
- **Roadmap**: [ROADMAP.md](../developer/ROADMAP.md)
- **Contributing**: [CONTRIBUTING.md](../developer/CONTRIBUTING.md)
