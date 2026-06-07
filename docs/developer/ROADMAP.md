<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Development Roadmap

**Last updated:** 2026-06-07

Development roadmap for **Memory Context Browser (MCB)** — a high-performance MCP server for semantic code search, persistent memory, and agent-aware context management.

---

## Current State

| Field | Value |
| ------- | ------- |
| **Version** | v0.3.2 from `Cargo.toml` |
| **Branch** | `feat/v0.3.2-ci-gates` |
| **Build** | Use `bd show mcb-v5an --json` for current v0.3.2 release-lane state |
| **Tests** | Use `bd show mcb-v5an.11 --json` for current CI verification state |
| **Crates** | 7 first-party workspace crates |
| **ADRs** | 55 tracked ADRs |

### Project Metrics

| Metric | Value |
| -------- | ------- |
| Beads issues | Use `bd status --json` for current totals |
| Avg lead time | Use `bd status --json` for current lead-time metrics |
| TODO/FIXME | Use `make guard` and the relevant bead for current remediation state |
| Languages | 14 via tree-sitter |
| Embedding providers | 6 (FastEmbed, OpenAI, VoyageAI, Ollama, Gemini, Anthropic) |
| Vector stores | 5+ (EdgeVec, Milvus, Qdrant, Pinecone, Encrypted) |

### Technical Debt

Current work, blockers, and technical-debt ordering are tracked in beads.

- Use `bd ready --json` for actionable work.
- Use `bd list --status open,in_progress --json` for the full open graph.
- Use `bd show <id> --json` for an individual item's acceptance criteria and evidence.

---

### v0.3.2 — CI/CD Gates And Release Reliability

**Tracking:** `bd show mcb-v5an --json`
**Branch:** `feat/v0.3.2-ci-gates`
**Tracking bead:** `mcb-v5an`

Hardens the release pipeline and development gates after v0.3.1 shipped. The
scope is CI cache efficiency, nextest/test-gate reliability, hook enforcement,
typos/doc validation, and release workflow resilience.

| Area | Status |
| ------ | -------- |
| Release workflow resilience | Implemented; CI evidence tracked in beads |
| Rust cache and nextest CI tuning | Implemented; verification tracked in `mcb-v5an.11` |
| Typos and hook gates | Implemented |
| Docs/governance cleanup | Completed under `mcb-vy4k`; current release docs tracked in `mcb-v5an` |
| Final PR/check validation | Use `bd show mcb-v5an.11 --json` and `make pr WHAT=checks PR=<id>` |

---

### v0.3.1 — Released

Stabilizes the SeaQL + Loco baseline for release by closing handler response
format drift, Docker runtime configuration, test helper reuse, and agent
instruction canonicalization.

| Area | Status |
| ------ | -------- |
| MCP JSON response formatting cleanup | Released |
| Loco inline config for Docker profiles | Released |
| Docker app/stdio compose profiles | Released |
| Agent instruction canonicalization | Released |
| Release gates | Completed for v0.3.1 release publication |

---

### v0.2.1 — Historical Release (Admin UI + Modernization)

**Status:** Released
**Branch:** `release/v0.2.1`

Consolidates all pre-v0.3.0 work: admin UI, data model hardening, modernization cleanup. No intermediate 0.2.x release exists between this track and v0.3.0.

| Area | Status |
| ------ | -------- |
| Admin UI (Handlebars CRUD) | ✅ Complete |
| P0 modernization (org context, lock/cache, dead code) | ✅ Complete |
| P1/P2 modernization (provider consolidation, docs) | ✅ Complete |
| CI pipeline modernization | ✅ Complete |

---

### v0.3.0 — SeaQL + Loco.rs Platform Rebuild

**Status:** Released
**Released:** 2026-02-27
**Key ADRs:** 049 (Axum), 050 (Composition Root), 051 (SeaQL+Loco master plan), 052 (Schema Resolution)

Full platform rebuild on SeaQL (SeaORM, SeaQuery, SeaSchema, SeaStreamer) and Loco.rs. Replaces Figment/TOML config, dill DI, and adds native Axum + rmcp Tower support.

| Component | Description |
| ----------- | ------------- |
| **Loco.rs** | Framework for server lifecycle, YAML config, migrations, workers |
| **SeaORM** | Async ORM for all persistence (replaces raw SQLx) |
| **SeaORM Pro** | Admin UI with CRUD, GraphQL, and seaography |
| **Axum Native** | rmcp Tower compatibility via Axum (ADR-049) |
| **Manual Composition Root** | linkme + Handle pattern, dill removed (ADR-050) |
| **SeaStreamer** | Event architecture (replaces ADR-004 event bus) |

**Unblocks:** v0.4.0 Workflow System

---

### v0.4.0 — Workflow System
**Tracking:** `bd show mcb-6pjx --json`
**Key ADRs:** 034 (FSM), 035 (Scout), 036 (Policies), 037 (Orchestrator), 038 (Tiers)
Implements complete workflow system with FSM-based task orchestration, context scouting, and policy enforcement.

| Component | Description |
| ----------- | ------------- |
| **WorkflowFSM** | 12-state machine for task orchestration with transitions and compensation |
| **ContextScout** | Freshness tracking (Fresh/Acceptable/Stale/StaleWithRisk) |
| **PolicyEngine** | 11+ policies for workflow validation and enforcement |
| **TaskOrchestrator** | Multi-layer coordination with event broadcasting |
| **ExecutionTiers** | Hierarchical execution (immediate, scheduled, deferred) |

**Unblocks:** v0.5.0 Integrated Context System

---

### v0.5.0 — Integrated Context System

**Tracking:** future beads created from ADR-041 through ADR-046 when this milestone becomes active
**Key ADRs:** 041-046
Multi-source integrated context with knowledge graphs, hybrid search, and temporal queries.

| Component | Description |
| ----------- | ------------- |
| **Knowledge Graph** | petgraph-based relationships (calls, imports, extends) |
| **Hybrid Search** | RRF fusion of semantic embeddings + BM25 keyword ranking |
| **Freshness Tracking** | Temporal metadata, immutable snapshots, time-travel queries |
| **Context Versioning** | Immutable captures at commits/tags with temporal query API |
| **Policy Integration** | Workflow state gates freshness requirements |

---

### v1.0.0 — Production Enterprise

**Tracking:** conceptual milestone; create beads before implementation work starts

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
3. **Clean Architecture**: 7-crate first-party workspace with trait-based DI
4. **Documentation First**: Documentation updated with each code change

---

## Cross-References

- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
- **Changelog**: [CHANGELOG.md](../operations/CHANGELOG.md)
- **Contributing**: [CONTRIBUTING.md](./CONTRIBUTING.md)
- **ADR Index**: [docs/adr/README.md](../adr/README.md)
- **Knowledge Graph Spec**: [v040-KNOWLEDGE-GRAPH-SPEC.md](../v040-KNOWLEDGE-GRAPH-SPEC.md)
