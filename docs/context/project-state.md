# Project State Context

**Last updated:** 2026-02-11
**Source:** Cargo.toml, git log, Beads stats, ROADMAP.md, PHASE-8-9-DEPENDENCY-MAP.md

## Current State

- **Version:** v0.2.1-dev (on branch `release/v0.2.1`)
- **Active branches:** `release/v0.2.1` (current), `release/v0.2.1`, `feat/data-model-v2`
- **Build:** ⚠️ Check `cargo check` — storage module issue reported in `mcb-providers/src/lib.rs`
- **Phase:** 8-9 planning — Phase 8 not started, Phase 9 ADRs complete

## Recent Activity (Feb 5-11)

Heavy admin feature development (412 commits in 6 days):
- **Admin UI**: Handlebars templates, entity CRUD, LOV endpoints, dashboards, navigation
- **Refactoring**: Tera → Handlebars migration, agent type definitions, tracing integration
- **Data model**: project_context module, ensure_parent for SQLite, repository resolver
- **Fixes**: Contextual errors, FK auto-create, DDL test counts, CI pipeline optimization

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
| Beads issues | 291 total | 75 open, 37 blocked, 38 ready, 216 closed |
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
