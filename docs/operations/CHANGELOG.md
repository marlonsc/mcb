<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Changelog
<!-- markdownlint-disable MD024 -->

All notable changes to**Memory Context Browser** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]
### No unreleased changes

---

## [0.3.0-dev] - 2026-02-XX

### Summary

Complete platform rebuild on the SeaQL ecosystem (SeaORM 2.x, SeaQuery, Seaography)
and Loco.rs framework. Replaces custom sqlx persistence layer and admin UI with
battle-tested libraries. All 9 MCP tools fully operational on the new stack.

### Added

- **SeaORM 2.x persistence layer** — 35 entities, 30 domain↔entity conversions, 7 repository implementations, 31-table migration via SchemaManager API (SQLite + PostgreSQL)
- **Loco.rs framework integration** — `McbApp` with Hooks trait, admin controllers, GraphQL controller, MCP stdio coexistence via Tokio task
- **Seaography GraphQL API** — Schema auto-generated from SeaORM entities, mounted at `/api/graphql` with JWT auth
- **SeaORM Pro admin panel** — Dashboard with multi-database queries (MySQL, PostgreSQL, SQLite), config serving at `/admin/config`
- **Contract snapshot tests** — 35 MCP tool contract tests via `insta` crate
- **ADR-051** — SeaQL + Loco.rs platform rebuild decision record
- **ADR-052** — Schema resolution for domain vs SeaORM entity naming
- **10 third-party forks** — SeaQL ecosystem libraries forked as git submodules in `third-party/` with `[patch.crates-io]` overrides

### Changed

- **Persistence**: sqlx → SeaORM 2.0.0-rc.34 (all repositories migrated)
- **Config format**: TOML (Figment) → YAML (Loco-native) for development and test configs
- **DI bootstrap**: Provider factories updated to resolve SeaORM repositories
- **Version roadmap**: Old v0.3.0 (Workflow FSM) → v0.4.0; Old v0.4.0 (Knowledge Graph) → v0.5.0

### Removed

- Custom sqlx persistence layer (`crates/mcb-providers/src/database/sqlite/`, ~3,827 LOC)
- Custom admin UI module (`crates/mcb-server/src/admin/`, ~5,062 LOC)
- Legacy TOML config files (replaced by YAML)
- 5 superseded ADRs archived (004, 007, 025, 026, 029)

### Fixed

- **memory list** — SQL bug in observation listing now resolved via SeaORM SeaQuery
- **session create** — Schema validation errors fixed with proper field handling
- **agent log_tool** — SQL storage failure fixed in agent repository
- **vcs list_repositories** — Repository discovery bug fixed
- **project tool** — Expanded from stubs to partial implementation (Get/List operations)

### Metrics

- Rust tests: 1,619 passing (`cargo test --workspace`)
- Contract snapshot tests: 35 passing (`cargo insta test`)
- Crates: 9 workspace members
- MCP tools: 9/9 operational
- SeaORM entities: 35
- Database tables: 31 (via single migration)
- ADRs: 52 total

## [0.2.1] - 2026-02-23

### Summary

Pre-v0.3.0 consolidation release focused on v0.2.1 branch hardening:
data-model-v2 merge, admin UI expansion, CI modernization,
dead-code cleanup, MCP handler/provider consolidation, and documentation
reconciliation.

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
- Rust Tests: 1,705 passing (`cargo test --workspace`)
- mcb-validate listed tests: 349 (`cargo test -p mcb-validate -- --list`)
- Crates: 9 workspace members
- Beads open issues triaged with scope tags: 50/50

---

## [0.2.0] - 2026-02-10

### Summary

Major stabilization release delivering critical bug fixes for Agent,
Session, and Memory subsystems, strict MCP protocol compliance, SQLite
DDL startup resilience, comprehensive test isolation, repository cleanup,
filesystem reorganization, and a full documentation overhaul with rebranding
to **Memory Context Browser**.

### Fixed

- **SQLite DDL Startup**: Replaced fragile FTS/trigger creation with
  `rebuild_fts_sqlite()` — drops and recreates FTS tables, triggers, and
  repopulates data. Eliminates `SQLITE_BUSY` failures on concurrent schema
  evolution.
- **SQLite Recovery**: Added backup-and-recreate recovery path when DDL operations fail on corrupted databases.
- **Agent SQL Storage**: Fixed "Failed to store tool call" error — correct repository dependency chain (Project -> Session -> Agent).
- **Session Schema**: Fallback logic for `agent_type` and `model` fields in session creation payloads.
- **Memory Enums**: Improved validation and error messages for `observation_type`.
- **Strict `tools/call` Validation**: Non-object `arguments` now return JSON-RPC `-32602` (Invalid Params) instead of being silently dropped.
- **Test Isolation**: All integration tests use `unique_test_config()` with
  unique temp DB paths per test, eliminating SQLite I/O contention during
  parallel execution.
- **Playwright E2E**: Fixed `test.describe()` not found error in CI by
  removing per-file spec iteration — Playwright now discovers all specs from
  `testDir`.
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
- **Filesystem Reorganization**: `docker-compose.yml`, `package.json`,
  `package-lock.json` moved to `tests/`. All Makefile, CI, and script
  references updated.
- **README Rewrite**: Complete professional rewrite (179 lines) with badges,
  feature overview, Quick Start, MCP Tools table, and ASCII architecture
  diagram.
- **Documentation Updates**: All docs updated to v0.2.0 (QUICKSTART,
  ARCHITECTURE, ENVIRONMENT_VARIABLES, docs/README).
- **ADR Status Standardization**: All ADRs use 5-value status set
  (IMPLEMENTED, ACCEPTED, PROPOSED, SUPERSEDED, ARCHIVED).
- **Legacy Removal**: Removed `/indexing` admin route (migrated to `/jobs`),
  deleted `ADMIN_SERVICE_DEFAULT_PORT` constant, removed tracked log
  artifacts.
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

First stable release — complete drop-in replacement for Claude-context with
14 languages processors, 6 embedding providers, 5 vector stores, systemd
integration, and comprehensive documentation.

### Added

- 14 languages processors with AST parsing.
- HTTP transport foundation with session management.
- Binary auto-respawn, connection tracking, signal handling.
- Systemd user-level service integration.
- Migration guide from Claude-context.

---

## [0.0.3] - 2026-01-07

### Summary

Production foundation — circuit breaker, health checks, intelligent routing,
Gemini/VoyageAI providers, encrypted vector storage.

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
