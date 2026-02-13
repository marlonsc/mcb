---
adr: 40
title: Unified Tool Execution Gate
status: PROPOSED
created: 2026-02-12
updated: 2026-02-12
related: [33, 34, 38]
supersedes: []
superseded_by: []
implementation_status: In Progress
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 040: Unified Tool Execution Gate

## Status

**Proposed** - 2026-02-12

## Context

Tool calls can enter through multiple interfaces (MCP, HTTP, admin web adapters). Direct repository access from non-unified entrypoints risks bypassing policy enforcement and provenance consistency.

## Decision

Adopt a single execution gate for tool calls:

1. Interface layers must delegate tool execution through unified routing.
2. Interface code must not call storage repositories directly.
3. Guard tests assert routing invariants in MCP, HTTP, and admin adapters.
4. Policy checks (project scope and conflict rejection) are enforced at unified tool handlers.

## Consequences

### Positive Consequences

- One consistent enforcement point for validation and policy.
- Smaller blast radius for behavioral changes.
- Better confidence from interface-level gate tests.

### Negative Consequences

- Slightly tighter coupling to unified routing interfaces.
- Additional test maintenance when routing APIs evolve.

## Alternatives Considered

### Alternative 1: Per-interface policy duplication

- **Description**: Keep separate enforcement logic in each transport/interface.
- **Pros**: Full local flexibility.
- **Cons**: High duplication and frequent drift.
- **Rejection Reason**: Incompatible with strict single-path execution requirements.
