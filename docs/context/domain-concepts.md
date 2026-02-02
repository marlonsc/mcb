# Domain Concepts Context

**Last updated:** 2026-02-02
**Source:** `README.md` (project narrative) and `.planning/STATE.md` (phase tracking)

## Overview
The product delivers semantic code search by combining vector embeddings, git context, and MCP tooling inside a Clean Architecture stack so teams can ask natural-language queries and get code recommendations enriched with project memory.

## Key Concepts

### MCP search tools
**Used in:** `README.md` "MCP Tools"
- `index_codebase`: ingest a repository and store embeddings in the selected vector store.
- `search_code`: answer natural-language prompts by matching vectors in the index.
- `get_indexing_status` / `clear_index`: observe and reset collections, keeping search data predictable.
**When to use:** Build API surfaces and CLI workflows around these MCP tools so every search operation stays traceable and testable.

### Phase-driven memory search
**Used in:** `.planning/STATE.md` (Phase 6 in progress)
- Phase 6 "Memory Search" currently executing; Phase 5 established SQLite + FTS + dedup workflows.
- Next action is 06-02 PLAN (Hybrid Search Implementation) which layers vector + memory search together.
**Guidance:** Sync new work with the documented phase progression and commit to `release/v0.1.5` so the branch state matches `.planning/STATE.md`.

## Constraints
- Approximately 1805 tests span the 9 crates, so major changes should keep test coverage in mind.
- 7 embedding providers and 8 vector stores (per `.planning/STATE.md` metrics) mean configuration must stay flexible.

## Related Context
- `docs/context/technical-patterns.md`
- `docs/developer/ROADMAP.md` (long-term planning)
