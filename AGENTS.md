<!-- AIHUB-INVIOLABLE-LAW-PRELUDE v1 -->
# AIHUB Inviolable Law — Strict Prelude

1. Truth: never claim done/green/resolved without command, exit code, decisive output.
2. Root cause: exterminate bypass, fallback, shim, suppression, stub, hardcode, catch-based normalization, retry, compatibility, partial execution, keyring, or old+new coexistence.
3. Tracker first: use the canonical tracker only when selected and available. If its runtime is explicitly suspended, create no substitute tracker or ledger; preserve evidence in separately authorized Git/PR/CI and do not declare the phase DONE.
4. Research first: inspect code, docs, canonical sources before acting; never invent APIs, flags, facts, or behavior.
5. Owner first: use the project's declared facades/primitives; do not reimplement them locally.
6. Gate persistence: a failure stops only that invocation. Correct its owner,
   republish, and rerun until green; never switch phase or repository because a
   check, review, approval, or merge is pending. Escalate only after every
   authorized technical action is exhausted and the remaining condition is
   genuinely external or requires new authority.
7. Landing: native gates, commit, fast-forward push, bead evidence.
8. Divergence: FF push rejected → integrate by cooperation: `git merge --no-ff` the integration base into your lane, resolve conflicts, revalidate, land. Never rebase or force-push an authorized change or integration branch; adopt all current worktree state and fix it forward.
9. Escalation: impossible rule → exact error. Rule conflict → present both with numbers. Unclear → one targeted question. Never guess.
10. Precedence: NEWEST > OLDEST. USER REQUEST > BEADS > ADRs > SKILLs > DOCS > default. Adjust lower/older to higher/newer. Doubt → ASK USER FIRST.
11. Workspaces: follow `rules/coordination/gascity.md`. Every manual task uses a dedicated Git worktree, branch, and physical `.venv`, never the primary checkout. Gas City suspension keeps orchestration inactive. Worktrees and staging stay on the destination filesystem, never `/tmp`; no borrowed environment, backup, or archive. Retire worktrees after verified integration.
12. Phase closure: keep the phase active through check repair, review resolution,
    independent approval, merge into the configured integration branch, and
    post-merge proof. Only then, with its Bead closed with evidence, is it DONE.
    When the operator states that no independent reviewer exists and authorizes
    an administrative merge, that authorization replaces the approval row alone;
    every other row stays mandatory and closure records the approval as
    operator-authorized, never as satisfied.
13. Root Make only: diagnostics, validation, generation, tests, Waza,
    publication, and deployment run only through selector-free verbs in the
    repository root Makefile; bare verbs perform their declared operation. A full
    suite has its own verb, first runs the incremental verb, and uses the same
    persistent external testmon database.
14. Red means red: a warning, skip, empty output, missing tool, missing report,
    zero collection, caught exception, retry, or normalized failure is RED. The
    only acceptable zero-execution test result is a typed incremental testmon
    cache hit with an integrity-checked database and complete deselection
    accounting; it is never reported as tests passed. The first exception and
    raw traceback escape unchanged.
<!-- /AIHUB-INVIOLABLE-LAW-PRELUDE -->

## AGENTS.md — mcb

> Packaged governance `agents-governance` `0.5.0` owns the capability indexes: 66 agents, 96 rules, 137 skills. Consume them through `GovernanceBundle`; do not copy their bodies here.

<!-- AIHUB-AGENTS-SCOPE-LOCAL-BEGIN -->
## AGENTS.md — mcb

> Packaged governance `agents-governance` `0.3.0` owns the capability indexes: 62
> agents, 50 rules, 102 skills. Consume them through `GovernanceBundle`; do not copy
> their bodies here.

## AGENTS.md — mcb

> **Parent workspace law** lives in
> [`AGENTS.md`](https://raw.githubusercontent.com/marlonsc/agents/0.12.0-dev/AGENTS.md)
> — read it first. Universal engineering core: `~/.agents/UNIVERSAL_CORE.md`.
> Composition: global skills + parent/root `AGENTS.md` + this scope delta. Do not
> re-embed universal law.
>
> **Standalone / independent mode:** when `../AGENTS.md` does not resolve, pin the
> parent raw `AGENTS.md` URL to the same branch/release as this package (never `main`).

<!-- migrated from CLAUDE.md -->

## CLAUDE.md

Canonical governance lives in this repo's `AGENTS.md`, whose first bytes are the strict
prelude selected by `config.AiHub.governance.law_surface`, followed by the project
overlay. Generated universal-core bodies are retired. **Do not duplicate rules here** —
keep only project-specific notes below.

<!-- AIHUB-POINTER-AUTHORITY-BEGIN -->
<!-- AIHUB-POINTER-AUTHORITY-END -->

- **Task tracking:** `bd` (beads). Run `bd prime`.
- **Validation:** prefer `make` targets (`make lint` / `make typecheck` / `make test`).
- **Tools:** `ast-grep` (`sg`) for structural search; never `rm` / `sed -i` (use the
  Edit tool or `trash-put`).

<!-- project-specific notes below -->

<!-- migrated from .windsurfrules -->
<!-- generated by `make gen-agent-pointers`; source: AGENTS.md -->
## MCB — Windsurf Rules

`AGENTS.md` is the project single source of truth for all agent rules,
architecture, commands, beads workflow, validation, and Git policy.

Do not duplicate those rules here. Update `AGENTS.md`, then run
`make gen-agent-pointers`.
<!-- AIHUB-AGENTS-SCOPE-LOCAL-END -->
