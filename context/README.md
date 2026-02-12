# Context Knowledge Base

Last updated: 2026-02-11

Purpose: keep high-signal project context for fast agent recall and long-term memory sync.

Hierarchy:

- `core/` - cross-cutting agent and tool behavior
- `development/` - delivery workflow, testing, review rules
- `project-intelligence/` - project-specific architecture and domain context
- `external/` - external library notes and links
- `core/sync-log.md` - timestamped context synchronization history

Operating rules:

- Keep each file under 200 lines (MVI: Minimum Viable Information).
- Prefer bullets and short tables over long prose.
- Update `Last updated` when content changes.
- Use explicit file paths for references.
- Run dependency checks after updates.

Validation checklist:

- All internal references resolve to existing files.
- No stale content older than 30 days.
- Cross-file duplication minimized.

Validation command:

- `scripts/context/validate-context.sh`
