---
name: flext-context-routing
description: Route FLEXT repositories through global execution skills and the branch-matched local flext-law domain delta after flext-core dependency detection.
---

# FLEXT Context Routing

This is the sole always-loaded local FLEXT surface. It selects the exact
branch-matched FLEXT law without duplicating universal execution governance.

## Required composition

1. Read `${config.AiHub.paths.agents_home}/skills/inviolable-rules/SKILL.md`.
2. Before build, generation, docs, checks, tests, or diagnosis, read
   `${config.AiHub.paths.agents_home}/skills/make-check/SKILL.md`.
3. For every FLEXT task, read the exact local
   `.agents/skills/flext-law/SKILL.md`.
4. At every completion boundary, read
   `${config.AiHub.paths.agents_home}/skills/verification-loop/SKILL.md`.

Fail closed if a required file is absent. Never resolve `flext-law` by an
unqualified catalog name, from `main`, or from another checkout.

## Detection and scope

- Activate when the workspace provider marker or dependency graph contains
  `flext-core`.
- In workspace mode, use the active workspace root and its checked-out law.
- In standalone mode, use the FLEXT root law pinned to the same branch or
  release; never fall back to `main`.
- Load only local surfaces declared in `.agents/provider.toml`. Global skills
  remain owned by `config.AiHub.paths.agents_home` and are not copied into the local provider.

## Memory and MCP

- `AGENTS.md` Learned sections: continual-learning only (≤12 bullets/section, high-signal; never promote law).
- Index: `.cursor/hooks/state/continual-learning-index.json`.
- MCP via ai-hub gateway: see `${config.AiHub.paths.ai_hub}/docs/MCP_AGENT_GUIDE.md` (beads/memory/CRG/ast-grep; Make for Done).

## ADR boundary

Architecture decisions remain in `docs/architecture/adr/`. This skill routes to law; it does not replace ADR ownership.
