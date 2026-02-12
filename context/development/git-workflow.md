# Git Workflow

Last updated: 2026-02-11 (America/Sao_Paulo)

## Purpose

Document the working git/beads workflow used by this repository.

## Standard Flow

1. Inspect local state (`git status`).
2. Implement and verify changes.
3. Stage relevant files.
4. Sync tracker metadata (`bd sync`).
5. Commit with conventional style.
6. Sync tracker again if metadata changed.
7. Push branch.

## Commit Guidance

- Keep commits focused and traceable.
- Follow conventional commit types/scopes.
- Include issue references when applicable.

## Beads Integration

- Track issue lifecycle with `bd` states (`open`, `in_progress`, `closed`).
- Use dependencies for blocked work.
- Use `bd ready --json` to pick next unblocked issue.

## Sources

- `docs/developer/CONTRIBUTING.md`
- `docs/BEADS_QUICK_REFERENCE.md`
- `docs/BEADS_DATA_MODEL.md`

## Update Notes

- 2026-02-11: Added unified git + beads flow for context operations.
