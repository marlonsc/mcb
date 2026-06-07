> Universal rules: `~/.agents/AGENTS.md` loads first. This file is the
> project-canonical overlay for `/home/marlonsc/mcb`.

# AGENTS.md - MCB Project Rules

<!-- BEGIN UNIVERSAL AGENT LAW (portable; regenerable; do not edit inside) -->
## Universal Agent Law (portable core)

**This block is the inviolable, agent-agnostic core of engineering conduct for this repository.** It is
self-contained: it binds any AI agent — Claude, Codex, Gemini, Cursor, Cline, GitHub Copilot, or any other —
and any user, with or without access to the author's personal configuration. The live user's explicit
instructions override this block; nothing else does. These rules apply to every project type and every
session, and may not be relaxed, reinterpreted, or scoped-out for convenience, speed, or perceived triviality.

### 1. Zero-Tolerance / Strict-Total
- **Always** fix the root cause — generically, cleanly, via reuse of existing canonical code — and validate it
  in the same turn with the actual command, its exit code, and the relevant output line.
- **Always** remove superseded code in the same cycle the replacement lands. No dead code "for later".
- **Always** fail loud when the single source of truth (identity, config, contract, version) is absent — never
  substitute a guess, a local copy, or an alternative path.
- **Never** use a fallback, compatibility wrapper, legacy branch, allowlist/carve-out, skip, suppression,
  hardcode, stub, fake, `TODO`/`FIXME`, or a side-script to make a gate pass.
- **Never** classify a failure surfaced by the current task as "pre-existing", "cosmetic", "unrelated", or
  "acceptable legacy". If it appears in your flow, you own it.

### 2. Fix-Forward-Only
Multiple agents may share one working tree. Reverting to a past state silently destroys another agent's
in-flight work. **Accept the current state and fix forward.** Discarding changes via `git checkout -- <path>`,
`git restore`, `git reset --hard`, `git reset <path>`, `git stash` (hiding others' work), `git clean`, or
`git revert` of another's commit is **forbidden**. If you think you must revert → **STOP and ask the user**;
never unilaterally revert shared work.

### 3. Root Cause Only — No Workarounds
No TODOs, stubs, fakes, fallbacks, compat wrappers, or "temporary" workarounds. No suppression directives
(`# type: ignore`, blanket `# noqa`, `@ts-ignore`, `eslint-disable`, etc.) and no escape-hatch typing
(`Any`, bare `object`, unchecked casts) unless carrying a one-line documented justification. A bypass that
hides a symptom is a defect even when the gate turns green.

### 4. Stay In Scope
Do exactly what the user asked — nothing more. No unrequested refactors, renames, cleanups, "obvious
improvements", or adjacent fixes. Found something unrelated? Mention it in one sentence; do not touch it.

### 5. Evidence Before Done — Report Honesty Is 100% Mandatory
"Done" means the **complete chain validated** with objective evidence (command + exit code + output), not
conclusion-by-sample. **Never** present partial, assumed, speculative, or unverified results as verified.
State explicitly when a step was skipped, when a check failed (paste the output), and when a result is
unverified. If something only worked via a workaround, say so — it is not "done".

### 6. Execute As Planned, Else Stop And Ask
Execute the agreed plan exactly. On anything that cannot be done cleanly — a blocked tool, a missing source of
truth, a real ambiguity, or a step that would require a bad practice — **STOP and ask**, presenting concrete
options. **Every option must be a clean, root-cause solution.** Fallback, hack, hardcode, suppression, skip,
or stub are **forbidden as suggestions** — never offer one, even labelled "quick" or "temporary". Any
mid-execution deviation from the plan requires explicit user confirmation **before** applying.

### 7. Blocked-Operation Protocol
When a tool, command, or edit is blocked (deny rule, security hook, sandbox, missing permission, unavailable
integration): (1) **Stop** — do not retry a variation or seek a bypass; (2) **diagnose in one sentence** what
was blocked and why; (3) **hand the exact command or edit to the user** to run on their side; (4) **wait for
their output** before continuing; (5) **never claim done because a substitute ran** — a successful bypass is
still a violation. Forbidden bypass techniques include `bash -c`/`sh -c` subshell wrapping, `eval`/`exec`,
`env <blocked>`, `xargs <blocked>`, absolute-path swaps to dodge prefix deny rules, pipes/command-chains into a
blocked command, and invoking it via a `subprocess` call.

### 8. Strict, Most-Restrictive Typing
Use the most restrictive type that compiles. No `Any`, no bare `object`, no suppression of type errors. Fix
types at the source; depend on declared contracts, not loosely-typed escape hatches.

### 9. Universal Engineering Principles (always, no exception)
- **SSOT** — one authoritative source per fact; reference it, never duplicate or restate it; fail loud when
  absent.
- **SOLID** — SRP / OCP / LSP / ISP / DIP respected. Type-switching where polymorphism applies, fat
  interfaces, and god-objects are defects.
- **YAGNI** — no speculative params, dead branches, future-hooks, or single-implementation abstractions.
  Build only what the task needs now; delete the rest.
- **DI / DIP** — depend on abstractions (protocols/interfaces); inject collaborators; no hidden globals or
  hard-wired construction inside business logic.

### 10. User Manages Git
Do not run `git add`/`commit`/`push`/`tag` unless the user explicitly requests it, and do not suggest
committing. Read-only inspection (`status`/`log`/`diff`) is fine. When a commit is authorized, write it as the
user with no agent/bot attribution — no `Co-Authored-By`, no "Generated with …" trailer, and never override
author/committer identity.

### 11. Multi-Agent Coordination
Agents may share one working tree. Coordinate through a committed task board (e.g.
`<repo>/.agents/coordination/tasks.md`): claim a task with an ownership + lease entry before editing, heartbeat
the lease, set `done`/`blocked` on finish, and recover stale tasks from git history. Commit small and often so
a fresh agent rebuilds state from `git log`. **Never overwrite or discard another agent's work** (see Rule 2);
on a divergent approach, stop and escalate to the user.

### 12. When Unsure — Ask
If a task is unclear, ambiguous, or would expand scope → ask one focused question. If an action is hard to
reverse, affects shared state, or could surprise the user → confirm first. Authorization is scope-specific:
approval for one action once does not authorize it in future contexts.

### 13. Destructive Commands — Archive, Don't Destroy
Prefer non-destructive moves: archive a file as `<file>.bak` instead of deleting it. Do not escalate
privileges (`sudo`/`su`), change ownership/permissions, perform remote operations, or fetch over the network
without explicit user confirmation. Use the agent's structured file/search/edit tools over raw destructive
shell commands.
<!-- END UNIVERSAL AGENT LAW -->

MCB (Memory Context Browser) is a Rust 2024 MCP server for persistent agent
memory, semantic code search, and architecture validation.

## Current Status

- Source version: `0.3.1` from `Cargo.toml`.
- Active branch observed during init: `release/v0.3.1`.
- Rust toolchain: stable, MSRV `1.92`, edition `2024`.
- Workspace: 7 first-party crates; `third-party/` is excluded from the
  workspace and should not be edited unless the user explicitly asks.
- Platform state: the v0.3 SeaQL + Loco.rs rebuild is the current baseline.
- Public MCP surface: 24 tool names registered through `linkme` descriptors,
  grouped into 9 handler families in `docs/MCP_TOOLS.md`.

When a static document disagrees with `Cargo.toml`, `Makefile`, `make/*.mk`,
`config/*.yaml`, or the code, trust the executable source first and update the
doc as part of the same change.

## Source Of Truth

- Version, MSRV, workspace members, lint policy: `Cargo.toml`.
- Rust toolchain components and targets: `rust-toolchain.toml`.
- Developer commands: `Makefile` plus `makefiles/ui.mk`, `makefiles/dispatch.mk`,
  and the canonical monopoly script `scripts/lib/mcb.sh` (exit codes, the
  `APPLY=Y` gate, SSOT readers, the banned-pattern guard, the agent bash-guard).
- Runtime configuration: `config/development.yaml`, `config/test.yaml`,
  `config/production.yaml`.
- Architecture validation config: `config/mcb-validate.toml` and
  `config/mcb-validate-internal.toml`.
- MCP tool contract: `docs/MCP_TOOLS.md` and `crates/mcb-server/src/args/`.
- Architecture rules and ADR context: `docs/architecture/` and `docs/adr/`.

## Commands

The whole dev cycle runs through few canonical `make` verbs backed by the single
monopoly script `scripts/lib/mcb.sh`. Pattern: `make <verb> [WHAT=phase]
[SCOPE=...] [APPLY=Y]`. Do not call `cargo`/`git` directly — use a verb. Run
`make help` for the live list.

```bash
make help                          # All verbs + their WHAT= phases
make build [RELEASE=0|1]           # Release build by default
make dev   [WHAT=run|docker-up|docker-down|docker-logs|docker-test]
make test  [SCOPE=unit|doc|golden|startup|integration|e2e|all] [THREADS=N]
make check [WHAT=fmt|lint|validate|audit|udeps|coverage|qlty|all] [QUICK=1]
make fix   [WHAT=fmt|lint|docs|all]   # Mutating auto-fix (rustfmt, clippy --fix, markdown)
make docs  [WHAT=build|serve|lint|validate|sync|rust|check|setup|adr|adr-new|diagrams] [QUICK=1] [FIX=1]
make ci                            # CI gate (check WHAT=all)
make guard                         # Banned-pattern scanner (prod unwrap/expect/panic/todo, TODO/FIXME, unjustified #[allow])
```

Read-only git / PR / submodule inspection flows through the same monopoly:

```bash
make git WHAT=status|diff|log|show|branch|tags|stash-list
make pr  WHAT=view|checks PR=<n>
make sub WHAT=status|diff
```

Single-test local debugging is allowed when it is materially faster than the
verb:

```bash
cargo test -p mcb-server --test unit -- test_name
```

Destructive verbs are DRY-RUN by default and require `APPLY=Y` to execute:

```bash
make codegen [WHAT=all|cli|db|entities|conversions|clean] APPLY=Y
make release [WHAT=package|version|install|install-validate] [BUMP=patch|minor|major] APPLY=Y
make clean   [WHAT=build|codegen|all] APPLY=Y
make git WHAT=commit MSG='...' [FILES='...'] APPLY=Y   # also push|merge|rebase
make sub WHAT=commit|push SUB=<name> [MSG='...'] APPLY=Y
make setup [WHAT=hooks|tools|adr|all]                  # hooks installs the pre-commit gate
```

`make release WHAT=install APPLY=Y` builds, installs config under the user's home
directory, updates MCP client configs when present, and manages the user `mcb`
systemd service. Run it only when the user explicitly asks for installation work.

Enforcement is mechanical, not honor-system: `make setup WHAT=hooks` installs
no-bypass tiered git hooks driven by one SSOT (`make hook WHAT=pre-commit|pre-push`
in `makefiles/dispatch.mk`). pre-commit (fast): staged `guard` + fmt + clippy
(`--workspace`) + typos + unit tests. pre-push (full): clippy `--all-targets` + full
suite + doctests + `validate quick`, then delegates to the beads `pre-push` hook.
`.claude/settings.json` denies dangerous shell and routes every Bash through
`scripts/lib/mcb.sh guard-bash`; `make guard` scans the full tree (CI/manual) while
the hook's `guard --staged` blocks only NEW violations in the commit.

## Task Tracking (beads / bd)

Work items live in **beads** (`bd`, a Dolt-backed dependency graph; `.beads/` is
already initialized). Prefer it over ad-hoc TODO lists for any multi-step work.

> **FUNDAMENTAL RULE — never edit `.beads/*.jsonl` (or any beads DB file) by hand.**
> `.beads/issues.jsonl` is a generated **export**, not the source of truth (Dolt is).
> Hand-editing it desyncs/corrupts the graph. **Every** create/update/close/dep/status
> change goes through the `bd` CLI — no exceptions, no manual JSONL/DB edits, ever.

- `bd prime` — load agent workflow context + project memories.
- `bd ready` — list work with no open blockers (actionable now).
- `bd create "Title" -p <prio> -t <task|bug>`; `bd dep add <child> <parent>` links dependencies.
- `bd update <id> --claim` — atomically take an item (assignee + in_progress); stops two agents touching the same work.
- `bd show <id>` / `bd close <id> "evidence"` — inspect / complete with a note.
- Hash IDs (`bd-a1b2`) avoid merge collisions across branches/agents.

For multi-agent execution, a coordinator owns the graph: re-analyze impact, write
closed specs, size conflict-free batches (no two in-flight items touch the same
file; `dispatch.mk`/`Makefile` are a serial lane), validate each delivery (green
gate + evidence) before `bd close`, then unblock dependents. No item closes red;
out-of-scope changes become new items, never silent expansion.

### Temporary Coordination Rule — Bead Audit / Docs Migration

Until the current bead-audit and retired-docs migration is complete:

- Own a dedicated coordination bead and split execution into child beads before
  editing. The coordinator bead for this lane is `mcb-v5an.14`.
- Keep this lane distinct from other agents' release/CI lanes. Do not claim or
  rewrite another agent's active bead; link dependencies through `bd` instead.
- Use subagents for independent audit/verification slices, with disjoint write
  scopes and coordinator review before closing beads.
- Run exactly one coordinator loop every five minutes in this session. Each
  five-minute tick performs, in order: read `bd status`, active child beads,
  subagent state, and `make git WHAT=status`; execute or integrate one scoped
  sub-bead; run the smallest relevant gate plus `bd sync`; commit/push a
  validated checkpoint when changes are complete. Repeat that same single tick
  until `mcb-v5an.14` and its children are closed or blocked with evidence.
  Between ticks, execute non-overlapping work only; do not start another poller,
  watcher, or sleep loop.
- Push frequent validated checkpoints while this user-authorized lane is active.
  Use the project `make git` verbs and include the validation evidence in the
  bead close note or commit message.

> **MAXIMUM RULE — never idle-wait.** Never block waiting on an async/long action
> (CI, builds, deploys, remote jobs). Always either *actively monitor* it (poll on a
> cadence) or pick up an independent non-blocking bead and return when it completes.
> Idle waiting is forbidden — there is always either monitoring or other ready work.
>
> **FUNDAMENTAL — push frequently.** After every commit (and at each loop step) push
> immediately via `make git WHAT=push APPLY=Y` — work is never stranded locally. Each
> push restarts CI (cancel-in-progress); that is expected — stop committing to let the
> final run go green, then merge. Always end a step by stating the next concrete action.
>
> **FUNDAMENTAL — one self-paced loop per session.** Drive long async work with a
> single ~5-min `ScheduleWakeup` heartbeat — never multiple overlapping loops or
> background watchers.
>
> **Lane separation + delegate.** With concurrent agents, each owns a distinct bead
> lane (respect assignees/claims; never touch another's). For your own epic, coordinate
> via sub-beads, dispatch a subagent per sub-bead, and quality-gate each delivery (green
> gate + evidence) before `bd close`.

## Architecture

Clean Architecture is enforced by dependency rules and `mcb-validate`.

```text
mcb                 # CLI facade binary
  -> mcb-server     # MCP protocol, Axum HTTP, handlers, admin UI
    -> mcb-infrastructure
       # DI/linkme + AppContext, Loco config, cache, logging, tracing
      -> mcb-domain # entities, value objects, port traits, errors
  -> mcb-providers  # adapters for embedding, vector store, DB, git, parsers
  -> mcb-validate   # architecture rule engine and analysis CLI
  -> mcb-utils      # shared leaf utilities
```

Dependency rules:

- `mcb-domain`: zero internal dependencies.
- `mcb-providers`: implements domain ports; depends on `mcb-domain` and
  `mcb-utils`.
- `mcb-infrastructure`: composition and runtime wiring; can use domain,
  providers, and utils.
- `mcb-server`: entrypoint and handlers; use services through DI ports.
- `mcb-utils`: leaf crate; no `mcb-*` dependencies.
- `mcb-validate`: developer tooling; keep runtime coupling deliberate and
  covered by validation config.

Do not import lower-level concrete providers directly into handlers. Add or
reuse a domain port, wire the adapter in infrastructure, and resolve through
the catalog/context.

## Runtime Configuration

MCB uses Loco YAML configuration. Loco-native sections are `logger`, `server`,
`database`, and `cache`; MCB-specific settings live under `settings:` and are
deserialized into `AppConfig`.

Profiles:

- Development: `config/development.yaml`, port `3000`, SQLite, Ollama
  embeddings, Milvus vector store.
- Test: `config/test.yaml`, dynamic port `0`, SQLite, FastEmbed embeddings,
  EdgeVec vector store, destructive test DB flags enabled.
- Production: `config/production.yaml`, port `8080`, SQLite, Ollama
  embeddings, Milvus vector store, admin API key header enabled.

Do not hardcode configuration values in code. Add fields to the typed config
model and populate every profile.

## MCP Tooling

The public MCP interface is 24 tool names grouped into 9 handler families:

- Search: `search_code`, `search_memory`
- Index: `index_repo`, `index_status`, `clear_index`
- Memory: `store_memory`, `get_memories`, `list_memories`,
  `memory_timeline`, `inject_context`
- Session: `start_session`, `get_session`, `list_sessions`,
  `summarize_session`
- Agent: `log_tool_call`, `log_delegation`
- Validation: `validate_code`, `analyze_code`, `list_rules`
- VCS: `list_repos`, `compare_branches`, `analyze_impact`
- Compound project/entity: `project`, `entity`

Handlers and schemas are split across `crates/mcb-server/src/args/`,
`crates/mcb-server/src/handlers/`, and `crates/mcb-server/src/tools/`.
Context/provenance fields are injected where the schema marks them hidden.

When changing a tool:

1. Update the args schema and validator.
2. Update the handler.
3. Update `docs/MCP_TOOLS.md` if the public contract changed.
4. Add or update focused tests for the action/resource touched.

## Implementation Rules

- Keep edits surgical and scoped to the user request.
- Prefer existing macros and patterns: `tool_action!`, `tool_schema!`,
  `tool_enum!`, `register_tool!`, `linkme` distributed slices, and the Handle
  pattern.
- Use `Error` constructors and `Result` aliases from `mcb-domain`; do not build
  raw domain errors by hand.
- Use `?` for propagation. No `unwrap()`, `expect()`, `panic!`, `todo!`, or
  `unimplemented!` in production paths.
- Keep imports ordered: `std`, external crates, `mcb_*` crates, local modules.
- Keep generated docs and reports fixed at the generator/template.
- Keep first-party source files compact; split modules before they become
  difficult to review.

## Testing And Verification

After meaningful edits, run the smallest relevant gate first, then broaden when
the change touches shared behavior:

- Rust code: `make check WHAT=lint` plus the relevant `make test SCOPE=...`.
- Architecture rules, dependencies, or crate boundaries: add
  `make check WHAT=validate QUICK=1` or `make check WHAT=validate`.
- Docs-only changes: `make docs WHAT=lint`.
- Public docs plus architecture/status changes: `make docs WHAT=validate QUICK=1`
  when practical.
- Release/install paths: `make release APPLY=Y` only when explicitly requested.

Report command, exit code, and the meaningful output. Do not claim a full gate
passed unless that exact gate was run in the current turn.

## Documentation Pointers

- `AGENTS.md`: project-canonical agent instructions.
- `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`: thin pointers
  back to this file.
- `README.md`: user-facing overview and quick start.
- `docs/MCP_TOOLS.md`: public MCP API.
- `docs/CONFIGURATION.md`: configuration index.
- `docs/developer/ROADMAP.md`: roadmap; verify against source before relying
  on static status.
- `docs/architecture/ARCHITECTURE.md`: architecture overview and historical
  context.
