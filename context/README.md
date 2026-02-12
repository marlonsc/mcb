# Context Directory

Last updated: 2026-02-11 (America/Sao_Paulo)

## Structure

```text
context/
├── core/
│   ├── agent-patterns.md
│   ├── tool-usage.md
│   └── error-handling.md
├── development/
│   ├── git-workflow.md
│   ├── testing-patterns.md
│   └── code-review.md
├── project-intelligence/
│   ├── technical-patterns.md
│   ├── domain-concepts.md
│   ├── integrations.md
│   ├── conventions.md
│   ├── project-state.md
│   └── v0.2.2-history-and-pending.md
└── external/
    └── mcp-and-agents.md
```

## MVI Rule

- Keep each file under ~200 lines.
- Keep sections scan-friendly (target: <=30 seconds to parse).
- Prefer links to authoritative docs over repeated long text.

## Maintenance

- Run reference and freshness validation after updates.
- Add an update note with date and intent for each modified context file.
