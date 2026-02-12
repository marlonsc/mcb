# Project State Context

**Last updated:** 2026-02-12
**Source:** Cargo.toml, git log, Beads stats, ROADMAP.md, CI pipeline results

## Current State

- **Version:** v0.2.1-dev (on branch `release/v0.2.1`)
- **Active branches:** `release/v0.2.1` (current)
- **Build:** ✅ All green — `cargo check`, `cargo fmt`, `make lint`, `make test` (28 suites)
- **CI:** ✅ Modernized — SHA-pinned Actions, `save-if` cache poisoning mitigation, `paths-filter`
- **Phase:** v0.2.1 modernization epic (`mcb-b9qd`) COMPLETED. Phase 8-9 planning next.

## Recent Activity (Feb 5-12)

Heavy admin feature development and modernization:
- **Admin UI**: Handlebars templates, entity CRUD, LOV endpoints, dashboards, navigation
- **Refactoring**: Tera → Handlebars migration, agent type definitions, tracing integration
- **Data model**: project_context module, ensure_parent for SQLite, repository resolver
- **CI modernization** (2026-02-12): SHA-pinned all GitHub Actions, `save-if` cache protection,
  `dorny/paths-filter` change detection, unified test matrix, Windows `shell: pwsh`
- **v0.2.1 closure** (2026-02-12): Epic `mcb-b9qd` completed — 8/8 items closed (P0: org-context,
  lock/cache, dead-code; P1: entity dispatch, provider boilerplate, config-driven DB; P2: validation
  engine rationalization, documentation drift reconciliation)

## Phase 8 (v0.3.0) — NOT STARTED

- **Scope**: Workflow FSM, Freshness Tracking, Policies, Compensation
- **ADRs needed**: ADR-034 (FSM), ADR-035 (Scout), ADR-036 (Policies), ADR-037 (Orchestrator)
- **Blocker**: `mcb-bphe` epic (P0) — ADRs must be written before implementation
- **Dependent issues**: 7 features blocked on this epic
- **Timeline risk**: Was planned for Feb 17 start — currently at risk

## Phase 9 (v0.4.0) — READY TO EXECUTE

- **Scope**: Knowledge Graph, Hybrid Search, Context Versioning, Time-Travel
- **ADRs**: ADR-041-046 (all complete and locked ✅)
- **Beads issues**: 35 created + dependency-linked
- **Blocker**: v0.3.0 must release first (`mcb-6pjx` epic, P0)
- **Timeline**: Feb 17 - Mar 16 (4 weeks) — **AT RISK** due to Phase 8 delay

## Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| Tests | 10,028 functions | Across all crates |
| Crates | 9 | Clean Architecture workspace |
| ADRs | 46 | Including Phase 8-9 |
| Embedding providers | 7 | OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic, Null |
| Vector stores | 5+ | EdgeVec, Milvus, Qdrant, Pinecone, Encrypted |
| Languages | 13 | Via tree-sitter |
| Beads issues | 306 total | 76 open, 38 blocked, 38 ready, 229 closed |
| TODO/FIXME | 241 | Code + docs debt markers |
| Docs files | 145 | Well-documented project |

## Technical Debt

1. **mcb-validate** coupled to runtime — should be decoupled
2. **Duplicate tree-sitter** logic across crates — need centralization
3. **Missing provider health** checks — no centralized validation
4. **241 TODO/FIXME** markers — accumulated code/docs debt
5. **225 missing_docs warnings** in mcb-domain struct fields

## Timeline Risk Assessment

| Scenario | v0.3.0 Ready | Phase 9 Start | Impact |
|----------|-------------|---------------|--------|
| Best case | Feb 10 | Feb 17 | On track ✅ |
| Nominal | Feb 24 | Mar 3 | 1-week slip |
| Risk case | Mar 10 | Mar 17 | 3-week slip, no recovery ❌ |

**Current trajectory**: Phase 8 not started → nominal/risk scenario likely.

## Roadmap

- **v0.2.1** (current): Admin UI, data model v2, Handlebars migration
- **v0.2.1** (in progress): Advanced admin features, filtering, bulk operations
- **v0.3.0** (Phase 8): Workflow FSM, freshness tracking, policies
- **v0.4.0** (Phase 9): Knowledge graph, hybrid search, versioning, time-travel

## Related Context

- `docs/context/technical-patterns.md` — architecture patterns
- `docs/context/domain-concepts.md` — entity model
- `docs/context/integrations.md` — external dependencies
- `docs/developer/ROADMAP.md` — detailed roadmap
- `.planning/PHASE-8-9-DEPENDENCY-MAP.md` — phase dependencies

## Mirror Context

- `context/README.md` — operational context hierarchy

## Change Notes

- 2026-02-11T23:26:00-03:00 - Reconciled with `context/` hierarchy and added mirror reference.
- 2026-02-12 - Updated for v0.2.1 closure: build status green, CI modernized, epic `mcb-b9qd` complete, Beads stats refreshed.
