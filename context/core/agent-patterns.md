# Agent Patterns

Last updated: 2026-02-11

Execution principles:

- Prefer explore-first behavior before asking questions.
- Match existing repository patterns before introducing new structures.
- Verify all modified files with diagnostics and project checks.

Delegation patterns:

- Use specialized agents for broad exploration and external research.
- Keep implementation edits local and verifiable.
- Use background research when parallelizable.

Memory behavior:

- Recall memory before major tasks.
- Store high-value learnings (patterns, errors, decisions).
- Sync stable learnings into context files for durable reuse.

Recent synced learnings (2026-02-11):

- Existing project context baseline already maintained in `docs/context/`.
- Phase 8 context assembly is tracked in issue `mcb-qt3`.
- Current high-value context includes architecture layering, provider matrix, and roadmap risk state.

Related:

- `context/core/tool-usage.md`
- `context/core/error-handling.md`
