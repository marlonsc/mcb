# Development Roadmap

## Overview

Development roadmap for **Memory Context Browser (MCB)** — a high-performance MCP server for semantic code search, persistent memory, and agent-aware context management.

--

### v0.3.0 — Workflow System

**Status:** Planning
**Target:** Q1 2026
**Key ADRs:** 034 (FSM), 035 (Scout), 036 (Policies), 037 (Orchestrator), 038 (Tiers)

Implements complete workflow system with FSM-based task orchestration, context scouting, and policy enforcement.

| Component | Description |
|-----------|-------------|
| **WorkflowFSM** | 12-state machine for task orchestration with transitions and compensation |
| **ContextScout** | Freshness tracking (Fresh/Acceptable/Stale/StaleWithRisk) |
| **PolicyEngine** | 11+ policies for workflow validation and enforcement |
| **TaskOrchestrator** | Multi-layer coordination with event broadcasting |
| **ExecutionTiers** | Hierarchical execution (immediate, scheduled, deferred) |

**Quality Gates:**

-   `make quality` passes (0 errors)
-   `make validate` passes (0 violations)
-   Performance benchmarks established
-   Migration guide from v0.2.0

**Unblocks:** v0.4.0 Integrated Context System

---

### v0.4.0 — Integrated Context System

**Status:** Design phase (parallel to v0.3.0)
**Target:** Q2 2026 (after v0.3.0)
**Key ADRs:** 041-046

Multi-source integrated context with knowledge graphs, hybrid search, and temporal queries.

| Component | Description |
|-----------|-------------|
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
|---------|-------------|
| SLA Guarantees | 99.9% uptime with monitoring |
| High Availability | Multi-region with automatic failover |
| Disaster Recovery | Backup/restore with point-in-time recovery |
| Compliance | SOC 2 Type II, ISO 27001, GDPR |

---

## Version History

| Version | Date | Key Changes |
|---------|------|-------------|
| v0.0.1 | 2026-01-06 | Architectural foundation, provider framework |
| v0.0.2 | 2026-01-06 | Documentation, CI/CD infrastructure |
| v0.0.3 | 2026-01-07 | Circuit breaker, health checks, Gemini/VoyageAI providers |
| v0.1.0 | 2026-01-11 | First stable release, 14 languages, systemd integration |
| v0.1.2 | 2026-01-18 | Linkme provider registration, mcb-validate crate, Admin UI |
| v0.1.3 | 2026-01-27 | Config consolidation, validation fixes |
| v0.1.4 | 2026-01-28 | RCA integration, security fixes |
| v0.1.5 | 2026-01-31 | Anthropic/Pinecone/Qdrant providers, health endpoints |
| **v0.2.0** | **2026-02-10** | **Stabilization, rebranding, DDL fix, test isolation, docs overhaul** |

---

## Development Principles

1. **ADR-Driven**: Architectural decisions documented before implementation
2. **Test-First**: Core functionality developed with comprehensive tests
3. **Clean Architecture**: 9-crate workspace with trait-based DI
4. **Documentation First**: Documentation updated with each code change

---

## Cross-References

-   **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
-   **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
-   **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
-   **ADR Index**: [docs/adr/README.md](../adr/README.md)
