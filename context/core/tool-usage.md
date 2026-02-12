# Tool Usage

Last updated: 2026-02-11 (America/Sao_Paulo)

## Purpose

Define practical tool selection rules for efficient and verifiable repository work.

## Primary Rules

- Use dedicated file tools for file operations:
  - `read` for inspection
  - `glob` for file discovery
  - `grep` for content search
  - `apply_patch` for surgical edits
- Use `bash` for execution tasks (build, tests, git, bd CLI).
- Prefer parallel calls when commands are independent.

## Search Strategy

- First pass: broad `grep` terms for architecture and workflow keywords.
- Second pass: targeted `ast-grep` for structural code patterns.
- Third pass: deep file reads for authoritative docs/ADRs.

## Validation Strategy

- Validate context references and freshness after edits.
- Record stale files and unresolved references explicitly.
- Keep output concise, path-based, and reproducible.

## Beads Notes

- Use `bd` via `bash` in this environment.
- Prefer JSON output (`--json`) for structured parsing.

## Sources

- `README.md`
- `docs/BEADS_QUICK_REFERENCE.md`
- `docs/BEADS_DATA_MODEL.md`
- `docs/adr/035-context-scout.md`

## Update Notes

- 2026-02-11: Added repository-specific tooling defaults for context operations.
