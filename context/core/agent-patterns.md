# Agent Patterns

Last updated: 2026-02-11 (America/Sao_Paulo)

## Purpose

Capture high-signal orchestration patterns for autonomous work in this repository.

## Execution Pattern

- Start with repo search (`glob`, `grep`, `ast-grep`) before edits.
- Prefer narrow, verifiable changes over broad refactors.
- Keep workflow state explicit: explore -> implement -> verify.
- Run verification locally before reporting completion.

## Delegation Pattern

- Use `explore` for internal pattern discovery.
- Use `librarian` for external docs/examples and best practices.
- Run independent searches in parallel to reduce blind spots.

## Safety Pattern

- Never suppress type/runtime issues with ignore directives.
- Never leave partial broken changes after failed attempts.
- Preserve existing user edits not related to current context task.

## Evidence Pattern

- Every context update should name source files.
- Keep context files small enough for fast scanning (MVI target: <200 lines).
- Include freshness metadata (`Last updated`, assumptions, update notes).

## Sources

- `README.md`
- `docs/architecture/ARCHITECTURE.md`
- `docs/adr/034-workflow-core-fsm.md`
- `docs/adr/035-context-scout.md`

## Update Notes

- 2026-02-11: Initial context/core baseline created from repository docs
  and ADR workflow series.
