# Project State Context

**Last updated:** 2026-02-03
**Source:** `.planning/STATE.md`, `.planning/PROJECT.md`, `docs/developer/ROADMAP.md`, `docs/developer/ROADMAP.md`, `docs/operations/CHANGELOG.md`

## Overview
Phase 6 "Memory Search" is currently underway with `release/v0.1.5` hosting the latest validated artifacts; the project has completed architecture cleanup, Git foundation/analysis, and memory foundations (SQLite/FTS triggers + deduplication) and is preparing for the 06-02 Hybrid Search plan.

## Current Phase
- **Phase:** 6 of 10 — Memory Search (in progress)
- **Plan:** 1 of 3 (FTS5 Infrastructure) complete; next is 06-02 Hybrid Search according to `.planning/STATE.md` and the roadmap's v0.2.0 vision.
- **Progress:** 53% progress bar on `.planning/STATE.md`.
- **Next action:** Finish `06-02-PLAN.md`, validate Hybrid Search integration tests, and merge into `release/v0.1.5` before advancing the tracker.

## Requirements & Debt
- **Validated requirements:** Full MCP protocol, 14 languages supported, 7 embedding providers, 8 vector stores, clean architecture with linkme/dill DI, architecture validation (mcb-validate), health endpoints, instrumentation, 1670+ tests.
- **Active work:** Git-aware semantic indexing (ADR-008), persistent session memory (ADR-009), advanced browser UI (ADR-028), provider health checks, break `mcb-validate` dependency cycle, consolidate language support.
- **Technical debt to track:** mcb-validate currently coupled to runtime, duplicate Tree-sitter logic, missing centralized provider health/config validation.

## Roadmap Signals
- `docs/developer/ROADMAP.md` marks v0.1.4 (RCA integration + security) as the latest release and outlines v0.2.0 as the next major effort spanning 25 phases (10 git + 10 memory + 5 UI).
- Key upcoming capabilities include git-aware indexing (multi-branch, submodule support, change impact), session memory workflows (progressive disclosure, context injection), and advanced code browser features (tree navigation, SSE updates).

## Metrics Snapshot
- **Tests:** 1818+ (per `docs/operations/CHANGELOG.md` updates that run alongside the docs generator).
- **Providers:** 7 embedding providers (update docs badges and metrics via pre-commit when counts change).
- **Vector stores:** 8 vector stores, ensure `docs/CONFIGURATION.md` and `docs/operations/CHANGELOG.md` list mirrors the live set.

## Related Context
- `docs/context/technical-patterns.md`
- `docs/context/domain-concepts.md`
- `docs/developer/ROADMAP.md`
- `docs/user-guide/README.md` (contextual user-facing summary)
