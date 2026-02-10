# Domain Concepts Context

**Last updated:** 2026-02-03
**Source:** `README.md` (project narrative), `.planning/STATE.md` (phase tracking), `docs/developer/ROADMAP.md`, and `.planning/PROJECT.md`

## Overview

The product delivers semantic code search by combining vector embeddings, git context, and MCP tooling inside a Clean Architecture stack so teams can ask natural-language queries and get code recommendations enriched with project memory.

## Key Concepts

### MCP search tools

**Used in:** `README.md` "MCP Tools"

-   `index (action=start)`: ingest a repository and store embeddings in the selected vector store.
-   `search (resource=code)`: answer natural-language prompts by matching vectors in the index.
-   `index (action=status)` / `index (action=clear)`: observe and reset collections, keeping search data predictable.
**When to use:** Build API surfaces and CLI workflows around these MCP tools so every search operation stays traceable and testable.

### Phase-driven memory search

**Used in:** `.planning/STATE.md` (Phase 6 in progress) and `docs/developer/ROADMAP.md` (v0.2.0 vision)

-   Phase 6 "Memory Search" is active; plan 06-01 concluded and 06-02 (Hybrid Search Implementation) is the next checkpoint.
-   The release branch `release/v0.2.0` holds these artifacts, so phase milestones should update this branch before advancing to the next plan.
**Guidance:** Always align new work with the numbered plan file (e.g., `06-02-PLAN.md`) and update `.planning/STATE.md` progress metrics to reflect completion percentages.

## Project-State Signals

**Used in:** `.planning/PROJECT.md` and `docs/developer/ROADMAP.md`

-   `docs/developer/ROADMAP.md` tracks release v0.2.0 (stabilization, rebranding, DDL fix) as current and plots v0.3.0 (workflow system) as the next target.
-   `.planning/PROJECT.md` lists validated requirements (MCP tools, 14 languages, 7 embedding providers, 8 vector stores, clean architecture) and active debt items (mcb-validate coupling, duplicate Tree-sitter, missing provider health checks).
-   `docs/context/project-state.md` consolidates these signals so contributors can see the validated capabilities, project constraints, and planned objectives together.

## Constraints

-   Approximately 1805 tests span the 9 crates, so major changes should keep test coverage in mind.
-   7 embedding providers and 8 vector stores (per `.planning/STATE.md` metrics) mean configuration must stay flexible.

## Related Context

-   `docs/context/technical-patterns.md`
-   `docs/developer/ROADMAP.md` (long-term planning)
-   `docs/context/project-state.md`
