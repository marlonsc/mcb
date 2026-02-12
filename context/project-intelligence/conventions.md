# Conventions

Last updated: 2026-02-11 (America/Sao_Paulo)

## Development Conventions

- Prefer make targets for build/test/lint/validate workflows.
- Keep changes aligned with crate boundaries and existing import patterns.
- Update docs/context when architecture or process behavior changes.

## Safety Conventions

- No suppressed type/runtime errors.
- No silent failure paths.
- No destructive git operations unless explicitly requested.

## Context File Conventions

- Keep each file concise (MVI target: <200 lines).
- Include `Last updated`, source paths, and update notes.
- Prefer explicit, searchable headings over long prose.

## Sources

- `docs/context/conventions.md`
- `docs/developer/CONTRIBUTING.md`
- `README.md`

## Update Notes

- 2026-02-11: Added context-management conventions for long-term assistant recall.
