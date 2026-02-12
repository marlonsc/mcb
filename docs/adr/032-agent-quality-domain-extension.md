---
adr: 32
title: Agent & Quality Domain Extension (MCB-Only)
status: SUPERSEDED
created: '2026-02-03'
updated: 2026-02-05
related: [9, 13, 29]
supersedes: []
superseded_by: [34]
implementation_status: Incomplete
---

## ADR-032: Agent & Quality Domain Extension (MCB-Only)

**Status:** Superseded by [ADR-034](034-workflow-core-fsm.md)
**Date:** 2026-02-03
**Deciders:** Architecture Team
**Supersedes:** None
**Related:** ADR-009 (Memory), ADR-013 (Clean Architecture), ADR-029 (Hexagonal/dill)

## Context

The oh-my-opencode workflow system currently uses:

- **6 shell hook scripts** - Stateless
- **9 markdown skill files** - Inject ~200 lines each
- **Beads CLI** - Separate SQLite database
- **GSD markdown files** - legacy-planning/ directory

Pain points:

1. No persistent state in hooks
2. Context pollution (~2000+ lines/session)
3. No execution history tracking
4. Duplicate data stores (Beads SQLite vs MCB SQLite)
5. No semantic search across workflow data

## Decision

**Extend MCB domain to be the SINGLE SOURCE OF TRUTH for workflow management.** No support for legacy file formats (legacy-planning/, .beads/).

### Key Decisions

#### 1. MCB-Only Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    MCB (Single Source of Truth)              │
│                                                              │
│  SQLite + Embeddings + FTS + Vector Store                   │
│                        │                                     │
│                     EXTENDS                                  │
│                        ▼                                     │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Agent      │ Quality    │ Project     │ Memory Ext    │ │
│  │ • sessions │ • configs  │ • phases    │ • executions  │ │
│  │ • deleg.   │ • checks   │ • issues    │ • error pat.  │ │
│  │ • tools    │            │ • decisions │ • context     │ │
│  │ • checkpt  │            │             │               │ │
│  └────────────────────────────────────────────────────────┘ │
│                                                              │
│  ❌ NO legacy-planning/ files                                     │
│  ❌ NO .beads/ files                                        │
│  ❌ NO bd CLI                                               │
│  ❌ NO bidirectional sync                                   │
└─────────────────────────────────────────────────────────────┘
```

#### 2. Tool Naming (per ADR-009)

| Prefix | Domain | Count |
| -------- | -------- | ------- |
| `agent_` | Session/delegation tracking | 7 |
| `quality_` | Quality gate enforcement | 3 |
| `memory_` | Executions, errors, context | 5 |
| `project_` | Project/phase/issue CRUD | 9 |

**Total: 24 MCP tools**

#### 3. Full CRUD for Project State

Replace GSD/Beads with complete CRUD operations:

```ascii
project_create          → Create project
project (action=create, resource=phase)    → Create phase
project (action=update, resource=phase)    → Update status/progress
project (action=create, resource=issue)    → Create issue
project (action=update, resource=issue)    → Update issue
project (action=add_dependency, resource=dependency)  → Add blocker
project_get_state       → Current state
project_get_ready_work  → Issues without blockers
project_log_decision    → Log decision
```

#### 4. No Legacy Support

| What | Decision |
| ------ | ---------- |
| legacy-planning/ import | NOT SUPPORTED |
| .beads/ import | NOT SUPPORTED |
| bd CLI compatibility | NOT SUPPORTED |
| Markdown export | NOT SUPPORTED |
| Bidirectional sync | NOT SUPPORTED |

**Rationale:** Simpler architecture, no sync conflicts, no parser code.

## Consequences

### Positive

1. **Single source of truth** - No sync conflicts
2. **Simpler architecture** - No import/export/sync code
3. **Semantic search** - All data has embeddings
4. **Unified queries** - One query language
5. **~100 LOC saved** - No parser code

### Negative

1. **Migration cost** - Existing projects need manual migration
2. **Learning curve** - New tools to learn
3. **MCB dependency** - Requires MCB server running

### Neutral

1. Legacy projects continue using bd CLI separately
2. No backward compatibility burden

## Implementation Plan

| Phase | Goal | LOC | Tools |
| ------- | | ------ | --- | ------- |-- |
| 1 | Agent Sessions | ~700 | 7 `agent_*` |
| 2 | Executions | ~400 | 2 `memory_*` |
| 3 | Quality Gates | ~500 | 3 `quality_*` |
| 4 | Error Patterns | ~400 | 2 `memory_*` |
| 5 | Project State | ~800 | 9 `project_*` |
| 6 | Context Assembly | ~400 | 1 `memory_*` |

**Total: ~3200 LOC | 14 plans | 24 tools | 9 tables**

## Alternatives Considered

### 1. Keep Bidirectional Sync

Maintain legacy-planning/ and .beads/ compatibility.

**Rejected because:**

- Complex sync logic
- Conflict resolution needed
- Parser code maintenance
- Two sources of truth

### 2. Import-Only (No Export)

Import legacy data but don't export.

**Rejected because:**

- Still need parser code
- One-way migration is cleaner
- Users can manually migrate

### 3. Extend Beads

Add MCB features to Beads CLI.

**Rejected because:**

- Beads is generic issue tracker
- Would need embedding pipeline
- Loses MCB integration

## Validation

### Architecture Rules

- CA007: Ports in mcb-domain only
- CA008: Infrastructure uses `Arc<dyn Trait>`
- Layer dependencies point inward

### Performance

| Tool | Target |
| ------ | -------- |
| `agent_start_session` | < 10ms |
| `quality_check_gate` | < 50ms |
| `project_get_ready_work` | < 50ms |
| `memory_get_context` | < 100ms |

## References

- [ADR-009: Persistent Session Memory](./009-persistent-session-memory-v0.2.0.md)
- [ADR-013: Clean Architecture](./013-clean-architecture-crate-separation.md)
- [ADR-029: Hexagonal Architecture](./029-hexagonal-architecture-dill.md)
- [Planning Documents](../plans/archive/LEGACY_PLANNING_PROJECT.md)
