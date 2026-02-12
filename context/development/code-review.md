# Code Review Guide

Last updated: 2026-02-11

Review focus order:

1. Correctness and architecture fit
2. Type and error safety
3. Test coverage impact
4. Readability and maintainability

Repository-specific checks:

- Respect layer boundaries (`server -> infra -> app -> domain`).
- Validate provider/port contracts for integration changes.
- Confirm context docs remain aligned with behavior changes.

Context-specific checks:

- References point to existing files.
- Content is concise and not redundant.
- `Last updated` reflects real edits.
