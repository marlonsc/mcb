<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# Agent Operations Runbook

<!-- TOC START -->

- [Purpose](#purpose)
- [Session startup checklist](#session-startup-checklist)
- [Lane lifecycle](#lane-lifecycle)
- [Validation battery](#validation-battery)
- [Environment quirks that break silently](#environment-quirks-that-break-silently)
- [Landing recipe](#landing-recipe)
- [Security advisory triage](#security-advisory-triage)
- [Handoff protocol](#handoff-protocol)
- [Coordination](#coordination)

<!-- TOC END -->

## Purpose

The operating manual for agent sessions working this repository: how a session starts,
how work is validated and landed, and where every durable source of truth lives. Written
for the next session, not for this one — if you are reading this mid-incident, jump to
[Validation battery](#validation-battery).

Durable sources this runbook routes to (never restate them here):

| Subject                              | Source                                                       |
| ------------------------------------ | ------------------------------------------------------------ |
| Mimosa advisory triage rules (D1–D5) | [ADR 059](../adr/059-mimosa-advisory-triage-policy.md)       |
| Fleet acting rules                   | `~/agents/rules/coordination/mcb-session-operating-rules.md` |
| Fleet workflow commands              | `~/agents/commands/` (`mcb-cycle`, `mcb-handoff`)            |
| Workstation skill router             | `.agents/skills/mcb.lane-ops/SKILL.md` (untracked by policy) |
| Cycle/handoff plan records           | `~/.claude/plans/<date>-<topic>-handoff/`                    |
| Bead playbooks                       | `bd show <bead>` description + comments                      |

## Session startup checklist

1. `git -C /home/marlonsc/mcb pull --ff-only` after `git restore .` (runtime drift is
   expected; never chain pull behind a pipe).
2. `bd ready` / `bd list` — pick a bead; search open PRs and remote branches for
   pre-existing work before writing any code. Redoing finished work is a defect;
   adopting abandoned work is the job.
3. Claim the bead (`bd update ... --status in_progress --assignee <you>`) and comment
   the lane path + branch. A bead untouched for over one hour is abandoned — claimed,
   deferred, and blocked included.
4. `gc mail send human` with bead + branch (+ PR when it exists).
5. Verify the toolchain pins before the first make call (see
   [quirks](#environment-quirks-that-break-silently)).

## Lane lifecycle

- Lane = dedicated `git worktree` at `/home/marlonsc/mcb-wt-<slug>`, branch
  `<kind>/<slug>` off `origin/develop`, **physical** `.venv` via `make setup`, then
  `direnv allow`. Never a symlinked venv, never the primary checkout.
- `make setup` after worktree creation; upg/bootstrap chicken-eggs resolve by running
  the failing verb a second time before diagnosing.
- Retire the lane after verified integration: `git worktree remove --force`,
  `git branch -D`, and prune the remote branch if GitHub did not.

## Validation battery

Run each gate isolated with its exit code captured; nothing merges without all green.
GitHub CI is disabled by operator decision — these gates are the only CI.

```
make gen check          # generator fixed point (never `gen WHAT=apply` by hand)
CI=Y make check         # all gates, CI posture
CI=N make check         # all gates, local posture
make test               # python suite (testmon incremental engine)
make rust WHAT=test     # workspace suite via cargo nextest (junit in .reports/nextest)
```

Zero-execution results are RED, never "passed". An exit code is not evidence: read the
log (`test result:`, the assertion text) before declaring RED or GREEN — a compile
failure is not a test failure.

`MATCH=<nextest -E filter>` narrows the rust run; `SCOPE=warmup` populates the HF model
cache in-job.

## Environment quirks that break silently

- mise lock/install only with the aube-capable receipt
  `~/.local/share/mise/bootstrap/mise-2026.9.15`; the host mise corrupts npm tool
  installs (shims that execute ELF binaries via node).
- `make test WHAT=rust` is a retired selector: it runs pytest and collects zero tests
  with exit 0. The rust gate is `make rust WHAT=test`.
- Rust runtime needs
  `ORT_DYLIB_PATH=$HOME/.local/lib/onnxruntime-linux-x64-1.24.4/lib/libonnxruntime.so`
  and `CARGO_HOME=$HOME/.cargo`.
- `bd` must be 1.3.0-fd.1 (schema v67) everywhere; old binaries fail with schema skew.
  `--force` is operator-authorized for unlocking beads.
- The Mimosa pre-tool hook blocks writing source files via Bash (heredocs, sed, python
  writers) and commands whose text cites shell-script paths — author with Write/Edit,
  stage with `git add -A`.
- Clippy denies `expect_used` even in `#[test]` functions (return `Result` and use `?`)
  and the workspace denies unused variables (prefix throwaway probe parameters with
  `_`).

## Landing recipe

1. Battery green (captured rc), changes committed with evidence in the body.
2. `git push -u origin <branch>`; `gh pr create --base develop` with the proof table
   (gates, rc, counts) in the body.
3. Merge with a merge commit (no-ff):
   `gh api -X PUT repos/marlonsc/mcb/pulls/N/merge -f merge_method=merge`.
4. Post-merge proof: fetch and show the merge commit on `origin/develop`.
5. Close the bead with the full evidence trail; retire the lane; report the landing to
   the coordinator.

## Security advisory triage

Sealed Mimosa scans recur on disposed anchors forever. Rules D1–D5 in
[ADR 059](../adr/059-mimosa-advisory-triage-policy.md) decide every class: trust
boundaries (D1), sink-level containment with RED-proven tests (D2), vendored-fixture
isolation (D3), client-side sentinels (D4), and the disposition ledger (D5) — beads
carry scan ids, seals, evidence, and landing references; a re-reported anchor with a
bead disposition closes by reference.

## Handoff protocol

When a session must stop with work in flight:

1. The lane stays on disk; nothing is reverted to "look clean" — uncommitted adoption
   state is documented, not discarded.
2. The bead description becomes the playbook: exact next command, remaining steps,
   quirks; comments carry the full trail.
3. A cycle record is written to `~/.claude/plans/<date>-<topic>-handoff/` with the
   self-critique and the durable-source map.
4. The local handoff pointer (`/home/marlonsc/mcb/.zcode/handoff-*.md`) only routes to
   the sources above; it never duplicates them.

## Coordination

- `gc mail send human` at claim time and at landing time, always naming the bead,
  branch, and PR.
- Distribution lists for the remaining backlog go to the coordinator, not into ad-hoc
  files.
- Critique other sessions only with evidence; adopt their landed work, never their
  uncommitted drafts without coordination.
