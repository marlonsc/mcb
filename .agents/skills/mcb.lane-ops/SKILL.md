---
name: mcb-lane-ops
description: "mcb lane operations: startup checklist, validation battery, landing recipe, triage routing, handoff protocol"
allowed-tools: Bash(make *), Bash(bd *), Bash(gc mail *), Bash(gh api *), Bash(git *)
metadata:
  aihub.tags: '["effective:2026-09-27", "route:project", "subject:beads", "subject:worktree"]'
---

# mcb lane ops

Workstation router for marlonsc/mcb operations. `.agents/` is not tracked in the
repo (workstation-local by policy, commit 7afaccaeb), so this skill only routes to
the durable sources — it never restates them.

## Durable sources

| Subject | Source |
|---|---|
| Advisory triage rules D1–D5 | `docs/adr/059-mimosa-advisory-triage-policy.md` (repo, tracked) |
| Full operations runbook | `docs/developer/AGENT-OPERATIONS.md` (repo, tracked) |
| Fleet acting rules | `~/agents/rules/coordination/mcb-session-operating-rules.md` |
| Fleet triage rule | `~/agents/rules/coordination/mimosa-advisory-triage.md` |
| Cycle command | `~/agents/commands/implementation/mcb-cycle.md` |
| Handoff command | `~/agents/commands/inspection/mcb-handoff.md` |
| Pre-merge auditor | `~/agents/agents/project-wide/mcb-lane-auditor.md` |
| Bead playbooks | `bd show <bead>` (description = playbook, comments = trail) |
| Cycle plan records | `~/.claude/plans/<date>-<topic>-handoff/` |

## Startup checklist

1. Restore drift, `git pull --ff-only` (never piped), read `bd list`.
2. Pick a bead; search PRs + remote branches + upstream WIP for pre-existing work.
3. Claim with lane-note comment; mail the coordinator bead/branch/PR.
4. Lane: `git worktree add /home/marlonsc/mcb-wt-<slug> -b <kind>/<slug> origin/develop`
   → `make setup` (physical `.venv`) → `direnv allow`.

## Validation battery (each isolated, rc captured, logs read)

```
make gen check
CI=Y make check
CI=N make check
make test
make rust WHAT=test   # ORT_DYLIB_PATH + CARGO_HOME exported; MATCH=<filter> to narrow
```

Zero-execution = RED. rc without its log line = unproven.

## Landing recipe

Battery green → commit with evidence body → push → PR (`--base develop`, proof
table) → merge commit → post-merge proof → close bead with evidence → retire lane →
mail the coordinator.

## Handoff protocol

Bead description becomes the playbook; the trail goes to comments; the cycle record
(with self-critique) goes to `~/.claude/plans/`; the local pointer file in
`.zcode/` only routes. See the `mcb-handoff` command.
