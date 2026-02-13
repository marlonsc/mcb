---
adr: 39
title: Context Persistence Boundary
status: PROPOSED
created: 2026-02-12
updated: 2026-02-12
related: [34, 35, 41]
supersedes: []
superseded_by: []
implementation_status: In Progress
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 039: Context Persistence Boundary

## Status

**Proposed** - 2026-02-12

## Context

The platform now persists context in multiple domains (memory, sessions, code indexing, and VCS entities). Without an explicit boundary, new write paths can diverge in scoping and provenance behavior.

## Decision

Define a single persistence boundary for origin context and project scoping:

1. Write operations are project-scoped by default.
2. Conflict between args and payload identifiers is rejected with invalid parameters.
3. Origin context is persisted across memory, session, code, and VCS write surfaces.
4. New handlers must route through shared resolver helpers for identifier precedence.

## Consequences

### Positive Consequences

- Consistent behavior across write surfaces.
- Lower risk of cross-project data leakage.
- Easier verification and regression testing.

### Negative Consequences

- Additional validation logic in handlers and adapters.
- Migration effort for legacy or partially-scoped write paths.

## Alternatives Considered

### Alternative 1: Local handler-specific validation only

- **Description**: Keep validation logic isolated in each handler.
- **Pros**: Simpler local ownership for each module.
- **Cons**: Drift and inconsistency over time.
- **Rejection Reason**: Conflicts with unified context and execution constraints.
