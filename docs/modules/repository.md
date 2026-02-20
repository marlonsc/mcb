<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# repository Module

**Port Traits**: [`crates/mcb-domain/src/repositories/`](../../crates/mcb-domain/src/repositories/)
**Implementations**: See [`providers.md#database-sqlite`](./providers.md#database-sqlite)

## ↔ Code ↔ Docs cross-reference

| Direction | Link |
| --------- | ---- |
| Code → Docs | [`crates/mcb-domain/src/repositories/mod.rs`](../../crates/mcb-domain/src/repositories/mod.rs) links here |
| Port Traits | [`mcb-domain::repositories`](../../crates/mcb-domain/src/repositories/) |
| Implementations | [`mcb-providers::database::sqlite`](./providers.md#database-sqlite) |

## Overview

The repository module defines the data access contracts (ports) used by the domain and application layers. Following Clean Architecture, the domain defines the *what* (traits), and the infrastructure/providers implement the *how* (SQLite/sqlx).

## Repository Traits (`mcb-domain`)

| Repository | Purpose | Source |
| ----------- | ------- | ------ |
| `MemoryRepository` | Observation storage + FTS search | [`memory_repository.rs`](../../crates/mcb-domain/src/repositories/memory_repository.rs) |
| `AgentRepository` | Agent session persistence | [`agent_repository.rs`](../../crates/mcb-domain/src/repositories/agent_repository.rs) |
| `OrgEntityRepository` | Organization tenant isolation | [`org_registry.rs`](../../crates/mcb-domain/src/repositories/org_registry.rs) |
| `VcsEntityRepository` | Repository & branch metadata | [`repository_registry.rs`](../../crates/mcb-domain/src/repositories/repository_registry.rs) |
| `PlanEntityRepository` | Task plans & versioning | [`plan_registry.rs`](../../crates/mcb-domain/src/repositories/plan_registry.rs) |
| `ProjectRepository` | Project & worktree coordination | [`project_repository.rs`](../../crates/mcb-domain/src/repositories/project_repository.rs) |

## Implementation Strategy

All production implementations use **SQLite 3** via the `sqlx` crate, providing ACID transactions and row-level isolation for multi-tenancy.

---
*Updated 2026-02-20 - Consolidated repository documentation to point to implementations in providers.md.*
