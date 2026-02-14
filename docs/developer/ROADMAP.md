<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Development Roadmap

**Last updated:** 2026-02-14

Development roadmap for**Memory Context Browser (MCB)** — a high-performance MCP server for semantic code search, persistent memory, and agent-aware context management.

---

## Current State

| Field | Value |
| ------- | ------- |
| **Version** | v0.2.1 |
| **Branch** | `release/v0.2.1` (only active release branch) |
| **Build** | ✅ `cargo check --workspace` passes |
| **Tests** | 1,705 passing (`cargo test --workspace`) |
| **Crates** | 9 (Clean Architecture workspace) |
| **ADRs** | 48 (including Phase 8-9) |

### Project Metrics

| Metric | Value |
| -------- | ------- | |
| Beads issues | 312 total (75 open, 237 closed, 39 ready, 36 blocked) |
| Avg lead time | 9.5 hours |
| TODO/FIXME | 164 markers in `crates/` |
| Languages | 13 via tree-sitter |
| Embedding providers | 7 (OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Anthropic, Null) |
| Vector stores | 5+ (EdgeVec, Milvus, Qdrant, Pinecone, Encrypted) |

### Technical Debt

1. **mcb-validate** coupled to runtime — should be decoupled
2. **Duplicate tree-sitter** logic across crates — need centralization
3. **Missing provider health** checks — no centralized validation
4. **241 TODO/FIXME** markers — accumulated code/docs debt
5. **225 missing_docs warnings** in mcb-domain struct fields

---

### v0.2.1 — Current Release (Admin UI + Modernization)

**Status:** In progress
**Branch:** `release/v0.2.1`

Consolidates all pre-v0.3.0 work: admin UI, data model hardening, modernization cleanup. No intermediate 0.2.x release exists between this track and v0.3.0.

| Area | Status |
| ------ | -------- |
| Admin UI (Handlebars CRUD) | ✅ Complete |
| P0 modernization (org context, lock/cache, dead code) | ✅ Complete |
| P1/P2 modernization (provider consolidation, docs) | ✅ Complete |
| CI pipeline modernization | ✅ Complete |

---

### v0.3.0 — Workflow System

**Status:** Planning
**Target:** Q1 2026
**Key ADRs:** 034 (FSM), 035 (Scout), 036 (Policies), 037 (Orchestrator), 038 (Tiers)

Implements complete workflow system with FSM-based task orchestration, context scouting, and policy enforcement.

| Component | Description |
| ----------- | ------------- |
| **WorkflowFSM** | 12-state machine for task orchestration with transitions and compensation |
| **ContextScout** | Freshness tracking (Fresh/Acceptable/Stale/StaleWithRisk) |
| **PolicyEngine** | 11+ policies for workflow validation and enforcement |
| **TaskOrchestrator** | Multi-layer coordination with event broadcasting |
| **ExecutionTiers** | Hierarchical execution (immediate, scheduled, deferred) |

### Quality Gates

- `make check` passes (0 errors)
- `make validate` passes (0 violations)
- Performance benchmarks established
- Migration guide from v0.2.0

**Unblocks:** v0.4.0 Integrated Context System

---

### v0.4.0 — Integrated Context System

**Status:** Design phase (parallel to v0.3.0)
**Target:** Q2 2026 (after v0.3.0)
**Key ADRs:** 041-046

Multi-source integrated context with knowledge graphs, hybrid search, and temporal queries.

| Component | Description |
| ----------- | ------------- |
| **Knowledge Graph** | petgraph-based relationships (calls, imports, extends) |
| **Hybrid Search** | RRF fusion of semantic embeddings + BM25 keyword ranking |
| **Freshness Tracking** | Temporal metadata, immutable snapshots, time-travel queries |
| **Context Versioning** | Immutable captures at commits/tags with temporal query API |
| **Policy Integration** | Workflow state gates freshness requirements |

**Architecture:** 5-layer system (VCS -> Indexing -> Memory -> Graph -> Search -> Policies), embedded-first (petgraph, tantivy, vecstore).

---

### v1.0.0 — Production Enterprise

**Status:** Conceptual
**Target:** After v0.4.0

Enterprise-grade platform with SLA guarantees, compliance certifications, and high-availability deployment.

| Feature | Description |
| --------- | ------------- |
| SLA Guarantees | 99.9% uptime with monitoring |
| High Availability | Multi-region with automatic failover |
| Disaster Recovery | Backup/restore with point-in-time recovery |
| Compliance | SOC 2 Type II, ISO 27001, GDPR |

---

## Development Principles

1. **ADR-Driven**: Architectural decisions documented before implementation
2. **Test-First**: Core functionality developed with comprehensive tests
3. **Clean Architecture**: 9-crate workspace with trait-based DI
4. **Documentation First**: Documentation updated with each code change

---

## Cross-References

- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
- **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
- **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
- **ADR Index**: [docs/adr/README.md](../adr/README.md)
- **Knowledge Graph Spec**: [v040-KNOWLEDGE-GRAPH-SPEC.md](../v040-KNOWLEDGE-GRAPH-SPEC.md)
