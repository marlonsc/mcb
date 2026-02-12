# Technical Patterns

Last updated: 2026-02-11 (America/Sao_Paulo)

## Architecture Pattern

- Clean Architecture with strict inward dependencies.
- Main flow: `mcb-server -> mcb-infrastructure -> mcb-application -> mcb-domain`.
- Supporting crates: `mcb-providers`, `mcb-validate`, `mcb-language-support`, `mcb-ast-utils`.

## Registration Pattern

- Provider discovery uses `linkme` distributed slices (compile-time registration).
- New providers should integrate via trait + distributed slice entry.

## Async/Error Pattern

- Traits are async-oriented (`#[async_trait]`) where required.
- Prefer explicit domain errors and propagation over panics.

## Context System Pattern (ADR Series)

- ADR-034..037 define workflow FSM, context scout, policies, orchestrator.
- ADR-041..046 define integrated context architecture
  (versioning, graph, hybrid search, freshness).

## Sources

- `README.md`
- `docs/architecture/ARCHITECTURE.md`
- `docs/adr/034-workflow-core-fsm.md`
- `docs/adr/035-context-scout.md`
- `docs/adr/041-integrated-context-system-architecture.md`

## Update Notes

- 2026-02-11: Harvested and condensed from existing
  `docs/context/technical-patterns.md` plus ADR updates.
