# Tool Usage

Last updated: 2026-02-11

Default strategy:

- Use targeted file tools for reads/searches/edits.
- Use shell for execution workflows (tests, build, git, tracker commands).
- Run independent searches in parallel to reduce latency.

Context operations mapping:

- harvest: scan code/docs and produce compact context artifacts.
- learn: capture successful patterns and fixes into memory.
- recall: query memory before implementation.
- compact: reduce file size while preserving actionable content.
- validate: check references and freshness.
- sync: merge memory insights into context hierarchy.

Validation baseline:

- Broken link/reference checks must pass.
- Freshness checks should flag files older than 30 days.
- Context files should stay under 200 lines each.

Standard lifecycle:

- `recall -> harvest -> organize -> validate -> sync`
