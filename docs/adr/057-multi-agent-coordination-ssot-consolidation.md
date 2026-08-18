<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 57
title: Multi-Agent Coordination and SSOT Consolidation
status: ACCEPTED
created: 2026-06-27
updated: 2026-06-27
related: [34, 35, 36, 37, 47, 56]
supersedes: []
superseded_by: []
implementation_status: "Implemented by generated agent pointers and ADR cleanup"
---

# ADR 057: Multi-Agent Coordination and SSOT Consolidation

## Status

**Accepted** - 2026-06-27

## Context

Agent coordination rules had drifted across tool-specific pointer files, IDE
guides, workflow ADRs, and project instructions. That made changes expensive and
increased the risk that one agent would follow stale process text while another
used the current rule set.

MCB now has a canonical project instruction file, generated pointer files, and
executable checks that can verify those pointers. The remaining architecture
documents should describe product decisions, not restate live operating rules.

## Decision

1. `AGENTS.md` is the source of truth for agent/operator rules, Beads
   coordination, local workflow discipline, and command-routing expectations.
2. Tool-specific files remain thin generated pointers produced by
   `make gen-agent-pointers`. They may add tool-local review ordering, but must
   not duplicate the rule corpus.
3. `.gemini/styleguide.md` is a generated Gemini review-order pointer only. It
   must not carry local copies of architecture violation tables, forbidden
   pattern lists, or command recipes.
4. ADRs may describe architecture, product behavior, and historical decisions.
   They must not restate live agent process rules, hook scripts, CI fragments,
   remediation recipes, or duplicated gate catalogs.
5. Executable sources own executable behavior: `Makefile`, `makefiles/*.mk`,
   `scripts/lib/mcb.sh`, `.github/workflows/ci.yml`, and
   `config/mcb-validate*.toml`.
6. `bd` remains the coordination ledger. ADRs can cite that dependency, but live
   task state, claims, blockers, and evidence belong in Beads.

## Consolidation Audit

- ADR-034, ADR-035, ADR-037, and ADR-056 remain product/architecture records.
  They reference workflow, Beads, sessions, or tenant identity as system
  concepts, not as duplicated agent-law text.
- ADR-036 now keeps the policy guard architecture and points operational policy
  details back to the executable SSOTs.
- ADR-047 now points multi-agent operational coordination to `AGENTS.md` and
  this ADR while keeping the Project entity decision.

## Consequences

### Positive

- Rule changes happen in one canonical place and are checked through generated
  pointers.
- Architecture documents stay smaller and less likely to diverge from current
  hooks, CI, or Make targets.
- Multi-agent sessions keep a single ledger and a single rule source.

### Negative

- Readers must follow links from ADRs to `AGENTS.md` or executable sources for
  operational detail.
- Historical ADR examples may be less self-contained after duplicate process
  snippets are removed.

## References

- [`AGENTS.md`](../../AGENTS.md)
- [ADR-036: Enforcement Policies](036-enforcement-policies.md)
- [ADR-047: Project Architecture](047-project-architecture.md)
- `scripts/lib/agent_pointers.py`
