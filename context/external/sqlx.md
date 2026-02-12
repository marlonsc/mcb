# sqlx

Last updated: 2026-02-12  
Scope: SQLx usage in MCB, repository execution patterns, operational risks, and validation guidance.  
Cross-reference: `context/external/tokio.md`, `context/external/thiserror.md`, `context/external/rocket.md`, `context/external/mcb-main-libraries-reference.md`.

---

## Executive Summary

SQLx is the persistence engine for core MCB data paths, especially SQLite-backed workflows:

- memory and observations
- sessions and metadata
- VCS entities and project models
- issue/plan/org/agent entity storage

In this repository, SQLx is not just a data access library. It is part of the reliability surface for MCP tools that read/write persistent state.

---

## Context7 + External Research

Context7 ID: `/websites/rs_sqlx_sqlx`

Official documentation:
- https://docs.rs/sqlx
- https://docs.rs/sqlx/latest/sqlx/sqlite/

Upstream repository:
- https://github.com/launchbadge/sqlx

---

## Actual MCB Usage (Current Source of Truth)

### Architecture Placement and Boundaries

#### Where SQLx is allowed

SQLx usage is intentionally concentrated in infrastructure/provider layers:

- `crates/mcb-providers/src/database/sqlite/*`
- selected infrastructure test or bootstrap paths

#### Where SQLx is not supposed to leak

Domain and application contract layers should not expose SQLx-specific error or row types.

Related evidence:

- `crates/mcb-domain/src/ports/infrastructure/database.rs` (abstraction intent)
- validation fixtures in `crates/mcb-validate/tests/fixtures/rust/domain_wrong_error.rs` explicitly model leak anti-patterns

This boundary is a clean-architecture requirement, not stylistic preference.

### Connection pool lifecycle

Primary anchor:

- `crates/mcb-providers/src/database/sqlite/provider.rs`

Observed behaviors:

- `sqlx::SqlitePool::connect(...)` to bootstrap connection pools
- schema application during initialization
- reconnect/error-path handling around initialization failures

### Executor wrapper pattern

Primary anchor:

- `crates/mcb-providers/src/database/sqlite/executor.rs`

Observed behaviors:

- centralized query execution abstraction over `SqlitePool`
- parameter binding through internal `SqlParam` mapping
- row extraction helpers via trait abstraction (`SqlRow`)

This wrapper keeps repositories consistent and reduces direct SQLx API sprawl.

### Repository implementation pattern

Representative repositories:

- `crates/mcb-providers/src/database/sqlite/memory_repository.rs`
- `crates/mcb-providers/src/database/sqlite/agent_repository.rs`
- `crates/mcb-providers/src/database/sqlite/project_repository.rs`
- `crates/mcb-providers/src/database/sqlite/issue_entity_repository.rs`
- `crates/mcb-providers/src/database/sqlite/org_entity_repository.rs`
- `crates/mcb-providers/src/database/sqlite/vcs_entity_repository.rs`

Common shape:

1. receive domain/request data
2. prepare parameterized SQL through executor abstraction
3. map rows into domain entities/value objects
4. convert lower-level failures into domain/infrastructure error taxonomy

### SQLite-specific operational behavior

Observed operational touchpoints:

- schema DDL application at startup
- potential file backup/recovery paths in provider setup
- FTS/hybrid-search interactions for memory retrieval flows

These are high-impact areas for release reliability.

### Query and Transaction Discipline

#### Parameterization is mandatory

All dynamic values should flow through bind/param APIs or executor param objects.

Rationale:

- prevent SQL injection vectors
- improve consistency and readability
- make query assembly easier to audit

#### Keep transactions explicit and short

Even in SQLite, long transactions can serialize write behavior and degrade concurrency.

Policy:

- use transaction scopes only where atomicity is required
- avoid embedding non-DB async work inside transaction window
- fail fast and rollback quickly on error paths

#### Avoid query-shape drift across repositories

Repository methods with similar semantics should preserve shape conventions:

- filtering style
- pagination handling
- sorting defaults
- null/missing-field behavior

Inconsistency here creates subtle API divergence at higher layers.

### Error Mapping and Boundary Hygiene

SQLx errors should be transformed near repository boundaries.

Preferred behavior:

- do not return `sqlx::Error` through domain-level APIs
- map to project error taxonomy with enough diagnostic context
- preserve source chains for debugging/logging

Cross-reference:

- `context/external/thiserror.md`

---

## ADR Alignment (Critical)

This document aligns with the following Architecture Decision Records:

- **ADR-009**: Database abstraction and repository patterns
- **ADR-021**: Error handling and boundary hygiene
- **ADR-034**: Query discipline and transaction management

---

## Common Pitfalls

### Runtime-only query validation risk

Current style uses runtime query strings heavily (`query`/`query_as` style through wrappers).

Risk:

- query shape mistakes discovered at runtime/test time instead of compile time.

Mitigation:

- strong integration test coverage per repository path
- optional targeted adoption of compile-time checked macros where practical

### SQLite write contention under load

Risk:

- increased tail latency for write-heavy operations.

Mitigation:

- tune pool and journaling pragmas according to deployment profile
- keep transaction scopes short
- separate read/write heavy workflows when possible

### Schema drift between code and runtime DB

Risk:

- startup/runtime failures after schema mismatch.

Mitigation:

- deterministic schema application order
- migration verification in CI/integration tests

### Row mapping fragility

Risk:

- missing/renamed columns trigger conversion failures.

Mitigation:

- centralize row conversion helpers
- add coverage for nullability and optional fields

### Contributor Guidelines

Do:

- keep SQLx usage in provider/infrastructure persistence modules
- bind every dynamic value
- map SQLx errors into project error types at repository edge
- add integration tests when query logic changes
- keep repository contracts domain-oriented

Do not:

- expose SQLx-specific types in domain contracts
- construct ad-hoc SQL with interpolated dynamic strings
- add long-running non-DB async operations inside transactions
- duplicate near-identical query logic across multiple repositories without shared conventions

---

## GitHub Evidence (Upstream + In-Repo)

### Upstream Repository

- https://github.com/launchbadge/sqlx
- https://github.com/launchbadge/sqlx/blob/main/tests/sqlite/sqlite.rs

### In-Repository Anchors

- `crates/mcb-providers/src/database/sqlite/provider.rs`
- `crates/mcb-providers/src/database/sqlite/executor.rs`
- `crates/mcb-providers/src/database/sqlite/memory_repository.rs`
- `crates/mcb-providers/src/database/sqlite/row_convert.rs`

### External Examples

- https://github.com/edo1z/rust-rocket-sqlx-sample

---

## References

### Verification Checklist (SQLx Changes)

When modifying SQLx-related code:

1. Confirm boundaries: no SQLx leakage into domain interfaces.
2. Confirm parameterization on every dynamic value.
3. Validate transaction scope minimality.
4. Run repository/integration tests touching changed queries.
5. Validate startup path still applies schema successfully.
6. Confirm error mapping preserves useful context.

Suggested commands:

```bash
rg -n "sqlx::|SqlitePool|query\(|query_as\(" crates/mcb-providers crates/mcb-infrastructure
cargo test
```

### Cross-Document Map

- Async runtime and concurrency around DB work: `context/external/tokio.md`
- Typed error conversion and propagation: `context/external/thiserror.md`
- Transport handlers that call persistence paths: `context/external/rocket.md`
- Central index of library responsibilities: `context/external/mcb-main-libraries-reference.md`
