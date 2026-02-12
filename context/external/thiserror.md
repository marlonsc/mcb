# thiserror
Last updated: 2026-02-12

Scope: project-specific `thiserror` strategy, `anyhow` boundary usage, and error-mapping discipline.  
Cross-reference: `context/external/sqlx.md`, `context/external/rocket.md`, `context/external/rmcp.md`, `context/external/tracing.md`.

## Executive Summary

MCB uses a dual strategy:

- `thiserror` for typed, stable library/domain contracts
- `anyhow` at orchestration/bootstrap boundaries where multiple heterogeneous sources are aggregated

This split is deliberate and architecture-aligned. It enables precise contract handling while keeping top-level flows ergonomic.

## Context7 + External Research

Context7 IDs:
- `/dtolnay/thiserror`
- `/dtolnay/anyhow`

Official documentation:
- https://docs.rs/thiserror
- https://docs.rs/anyhow

## Actual MCB Usage (Current Source of Truth)

### Primary Internal Evidence

Key anchors for typed errors:

- `crates/mcb-domain/src/error/mod.rs`
- `crates/mcb-domain/src/ports/browse.rs`
- `crates/mcb-domain/src/ports/providers/metrics.rs`
- `crates/mcb-language-support/src/error.rs`
- `crates/mcb-ast-utils/src/error.rs`
- `crates/mcb-validate/src/lib.rs`

Key anchors for conversion/mapping:

- `crates/mcb-server/src/error_mapping.rs`
- handlers under `crates/mcb-server/src/handlers/`

Boundary-oriented `anyhow` usage is visible in bootstrap/operational setup, such as:

- `crates/mcb-providers/src/database/sqlite/provider.rs`
- `crates/mcb-infrastructure/src/config/loader.rs`

### Error Taxonomy Patterns in MCB

#### Core enum-based domain model

MCB relies on explicit enums with structured variants rather than stringly-typed errors.

Benefits:

- predictable matching and mapping behavior
- maintainable transport/protocol conversion
- strong compile-time guarantees on failure classes

#### Source preservation

Patterns with `#[from]` and/or explicit source fields are used to keep causal chains available.

Operational impact:

- better triage in logs
- safer conversion at boundaries

#### Helper constructor pattern

`crates/mcb-domain/src/error/mod.rs` includes helper constructors for common variants, which improves consistency and reduces repetitive ad hoc text.

### Boundary Rules

#### Domain contract rule

Domain-facing interfaces should not leak provider/framework error types (`sqlx`, transport-specific, etc.).

Rationale:

- keeps domain stable when infrastructure changes
- enforces clean architecture separation

#### Validation enforcement

The repo contains fixture-based validation checks that model forbidden leak patterns (for example, domain signatures exposing external errors).

Relevant fixture path:

- `crates/mcb-validate/tests/fixtures/rust/domain_wrong_error.rs`

### Mapping to Transport and Protocol Surfaces

The typed error model eventually maps into:

- HTTP/admin responses
- RMCP tool call responses
- structured logs and telemetry

Primary mapping layer:

- `crates/mcb-server/src/error_mapping.rs`

This mapping layer is a reliability boundary: changes in variants or messages must preserve response coherence.

## ADR Alignment (Critical)

Aligned ADRs:

- `docs/adr/019-error-handling-strategy.md`
- `docs/adr/013-clean-architecture-crate-separation.md`
- `docs/adr/021-dependency-management.md`

Practical interpretation:

- typed contracts where semantics matter
- aggregated context where orchestration dominates

## GitHub Evidence (Upstream + In-Repo)

### Upstream Evidence

- https://github.com/dtolnay/thiserror (official thiserror repository)
- https://github.com/dtolnay/anyhow (official anyhow repository)

### In-Repo Evidence

Key anchors for typed errors:

- `crates/mcb-domain/src/error/mod.rs`
- `crates/mcb-domain/src/ports/browse.rs`
- `crates/mcb-domain/src/ports/providers/metrics.rs`
- `crates/mcb-language-support/src/error.rs`
- `crates/mcb-ast-utils/src/error.rs`
- `crates/mcb-validate/src/lib.rs`

Key anchors for conversion/mapping:

- `crates/mcb-server/src/error_mapping.rs`
- handlers under `crates/mcb-server/src/handlers/`

Boundary-oriented `anyhow` usage:

- `crates/mcb-providers/src/database/sqlite/provider.rs`
- `crates/mcb-infrastructure/src/config/loader.rs`

External examples:

- https://github.com/paradigmxyz/reth/blob/main/crates/engine/tree/src/tree/error.rs
- https://github.com/huggingface/candle/blob/main/candle-core/src/error.rs

## Common Pitfalls

### Overuse of `anyhow` in deep library code

Risk:

- semantic contract loss and weaker architecture boundaries.

Mitigation:

- keep typed errors in library and domain crates; confine `anyhow` to orchestration/bootstrap edges.

### Generic variant text

Risk:

- low-actionability diagnostics during incidents.

Mitigation:

- include domain operation context in variant messages.

### Variant growth without mapping updates

Risk:

- new variants may bypass consistent transport/protocol mapping.

Mitigation:

- require mapping tests/coverage update when variant set changes.

### Lost source chains

Risk:

- root cause obscured in logs/incident analysis.

Mitigation:

- preserve source references through conversion path and log structured context.

### Contributor Guidance

Do:

- model meaningful failure classes as typed enums with `thiserror`
- convert external errors at adapter boundaries
- preserve source chains where useful
- keep messages concrete and actionable
- update mapping tests when variants evolve

Do not:

- expose external crate error types from domain interfaces
- bury semantic errors inside generic text-only wrappers
- panic for expected operational failures

## References

### Verification Checklist

When editing error code:

1. Verify boundary hygiene (no domain leak of infrastructure errors).
2. Verify all new variants are mapped at server/protocol boundaries.
3. Verify source-chain preservation for important failure paths.
4. Verify tests cover representative conversions and response shapes.
5. Verify tracing logs still carry operation-specific context.

Suggested commands:

```bash
rg -n "derive\(Error|thiserror::Error|\#\[error\(" crates
rg -n "sqlx::Error|tokio::.*Error" crates/mcb-domain
cargo test
```

### Cross-Document Map

- Persistence boundary and SQL failure classes: `context/external/sqlx.md`
- HTTP mapping and responder behavior: `context/external/rocket.md`
- MCP tool/protocol error behavior: `context/external/rmcp.md`
- Diagnostic instrumentation around failures: `context/external/tracing.md`

### Official Documentation

- https://docs.rs/thiserror
- https://docs.rs/anyhow
- https://rust-lang.github.io/api-guidelines/interoperability.html#error-handling

### Repository Anchors

- `crates/mcb-domain/src/error/mod.rs`
- `crates/mcb-server/src/error_mapping.rs`
- `crates/mcb-validate/tests/fixtures/rust/domain_wrong_error.rs`
