# Domain Concepts

Last updated: 2026-02-11 (America/Sao_Paulo)

## Product Model

- MCB is an MCP server for semantic code intelligence and project memory.
- Core domains: indexing, search, memory observations, sessions,
  validation, VCS context.

## Context Lifecycle Concepts

- Workflow state is explicit (FSM-driven, ADR-034).
- Project context is discovered and cached (Context Scout, ADR-035).
- Policy gates validate transitions/actions (ADR-036).
- Orchestration coordinates execution and compensation (ADR-037).

## Integrated Context Concepts (Planned/Active)

- Unified snapshot model combining VCS, memory, graph, and workflow state.
- Freshness levels and staleness signals guide safe reuse.
- Time-travel and versioned context snapshots are first-class design goals.

## Sources

- `README.md`
- `docs/context/domain-concepts.md`
- `docs/adr/034-workflow-core-fsm.md`
- `docs/adr/035-context-scout.md`
- `docs/adr/045-context-versioning-freshness.md`

## Update Notes

- 2026-02-11: Re-harvested from docs/context + Phase 8/9 ADR set.
