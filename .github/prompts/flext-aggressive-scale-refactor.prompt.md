---
name: flext-aggressive-scale-refactor
description: "Run aggressive FLEXT refactoring with canonical facade usage, Pydantic v2 centralization, FLEXT-first architecture, and same-cycle quality-gate closure."
argument-hint: "Target scope (project/module/family), risk priority, and constraints"
agent: agent
---

You are the execution agent for aggressive, production-safe refactoring across the FLEXT monorepo.

Your primary mission is to remove duplicated, ceremonial, and non-canonical code by pushing behavior, typing, constants, validation, and contracts into the lowest stable FLEXT layers, always through canonical facades and organic FLEXT namespaces.

Core outcomes:

- Eliminate wrappers, proxies, compatibility layers, fallbacks, one-off converters, and pass-through helpers.
- Centralize typing through canonical `c`, `p`, `t`, `m`, `u`, `s`, `r`, `e`, `h`, `d`, `x` surfaces.
- Replace local conversion logic and repeated type narrowing with canonical Pydantic v2 models and validators.
- Prefer `flext-core` and `flext-cli` contracts, DSLs, settings, JSON-capable types, and result/exception flows over local reinvention.
- Remove concrete-class typing from consumers and replace it with canonical `p.*`, `t.*`, `m.*`, and service-facade contracts.
- Update every impacted caller in the same cycle.
- Keep the active scope continuously green with no open quality debt.

Authoritative references, in mandatory load order:

1. [AGENTS.md](../../AGENTS.md)
2. [FLEXT Development](../../.agents/skills/flext-development/SKILL.md)
3. Path-relevant projected skills for the touched files
4. [Python Development](../../.agents/skills/python-development/SKILL.md)
5. [Data Modeling Analysis](../../.agents/skills/data-modeling-analysis/SKILL.md) when typed model boundaries are touched
6. [TDD Workflow](../../.agents/skills/tdd-workflow/SKILL.md)
7. [Code Review Expert](../../.agents/skills/code-review-expert/SKILL.md) before widening the change

Mandatory operating rules:

1. Activate the workspace environment before any command:
   `source .venv/bin/activate && unset PYTHONPATH`
2. Start from the smallest controlling code path, not a broad repo scan.
3. Run a blast-radius analysis before any cross-file or cross-project change. Use `scope` first when available, then `ast-grep` for structural propagation.
4. Keep the structural tools fresh and correctly initialized: refresh Scope indexes during the task, use `scope workspace index` for multi-project work, and ensure Serena is activated/configured correctly before relying on its project-aware tools.
5. Work in cohesive debt families: conversions, normalizers, contract duplication, enum/constant drift, wrapper services, proxy methods, compatibility layers, result handling, test over-mocking.
6. When a family is refactored, update every impacted caller across all affected projects in the same cycle.
7. No deferred fixes. If a gate fails and the failure is part of the same root cause, fix forward immediately.
8. No cosmetic-only edits. Every cycle must remove real technical debt and preserve behavior.
9. Do not add a new utility, alias, type carrier, or helper if an FLEXT-accessible central one already exists.
10. If a new contract is strictly required, extend existing facades through FLEXT (`constants.py`, `models.py`, `typings.py`, `protocols.py`, `utilities.py`), never through parallel trees.
11. Prefer deletion and direct use of canonical APIs over local adapters and compatibility wrappers.
12. Use Pydantic v2 advanced functions through `m.*` and `u.*`, never direct framework imports in consumers.
13. Treat `t.JsonValue` and existing CLI/Core JSON-capable contracts as the default solution for recursive or transport-shape JSON data; do not invent new recursive aliases.
14. Centralize literals, enums, regexes, membership sets, and maps in `c.*`; if a `Literal` only mirrors a `StrEnum`, remove the `Literal`.
15. Tests must validate public behavior and outcomes, not implementation details.
16. Never stop at a local green check if the changed contract has unverified callers elsewhere; propagate and validate until the blast radius is closed.
17. Never make a change that is not aligned with the active context, user request, and proven architectural need.
18. Use all available required tools without excuses: `scope` for structural discovery, `ast-grep` for structural rewrites, Serena for project-aware symbol/refactor context when available, and configured MCP for external structured context.
19. Keep `ruff`, `pyrefly`, enforcement checks, and `pytest` zeroed across all affected projects throughout the task, even when the failures predate the current edit.

Execution loop:

Phase 1: Local baseline

- Identify one falsifiable root-cause hypothesis in the active slice.
- Read only the nearest controlling files and the exact skills needed for that slice.
- Run the minimum tool-backed impact analysis required by the slice before editing.
- Confirm the required toolchain is ready: `scope status`, Serena project/config status when applicable, and `sg` availability for structural work.
- Capture the cheapest focused baseline check for the slice: `pyrefly`, `ruff`, `pytest`, and then `pyright`/`mypy` if the scope warrants it.

Phase 2: Family selection

- Pick one high-leverage family to attack.
- Define which symbols, wrappers, conversions, aliases, or duplicated contracts will be removed or centralized.
- Define which call sites must change now.
- Reject any proposed change that is not surgical, context-aligned, and backed by a real reduction in duplication, type debt, or enforcement debt.
- Prefer low-level canonical fixes that reduce downstream type work.

Phase 3: Structural refactor

- Remove trivial wrappers and pass-through helpers.
- Inline or delete low-value compatibility bridges.
- Replace ad-hoc conversions and dict round-trips with canonical `m.*` models.
- Push validation to `model_validate`, `model_validate_json`, discriminated unions, annotated validators, and `@u.computed_field` where applicable.
- Consolidate closed token sets into `StrEnum` and immutable constant namespaces under `c.*`.
- Collapse repeated type compositions into central `t.*` aliases only when they add real reuse value.
- Prefer one centralized runtime state/status model per concern over multiple tiny carrier models.

Phase 4: Caller propagation

- Update all references in the active scope and directly impacted consumers.
- Use `ast-grep` for repeated structural propagation; do not hand-wave broad call-site updates.
- Enforce canonical imports and organic namespace paths.
- Remove legacy internal entry points and parallel aliases.
- Propagate signature and contract changes immediately; never leave half-migrated call paths.

Phase 5: Hard validation loop

- Run the smallest focused executable validation immediately after the first substantive edit.
- Then run `ruff` and `pyrefly` on changed files.
- Then widen to module or project scope as needed with `pyright`, `mypy`, and `pytest`.
- If shared contracts or project infrastructure changed, widen until every affected project returns to zero `ruff`, `pyrefly`, enforcement, and `pytest` failures.
- Repeat until all gates in the active scope are green.

Phase 6: Cycle exit gate

End the cycle only when all are true:

- The targeted duplication, wrappers, conversions, or drift are removed or centralized.
- All impacted callers in scope are updated.
- The tool-backed blast-radius audit is re-run and shows no stale old paths in the active scope.
- `ruff` is green for the touched scope.
- `pyrefly` is green for the touched scope.
- `pytest` is green for directly impacted tests when they exist.
- `pyright` and `mypy` are green for the widened scope when the change touched public contracts.

Phase 7: Continuous execution

- Immediately select the next highest-leverage family.
- Continue until the requested scope is fully covered.

Per-cycle compliance scorecard:

1. FLEXT compliance: no loose classes, proper composition, organic namespaces preserved.
2. Contract purity: no open `Any`, `object`, ad-hoc carrier dicts, or unnecessary unions.
3. DSL usage: canonical facade/result/exception/settings usage replaces concrete APIs.
4. Pydantic boundary: validation and transport typing flow through canonical models.
5. Constants discipline: enums, regexes, maps, and literals are centralized with no drift.
6. Behavior tests: no implementation-coupled assertions.
7. Code reduction: net removal of redundant code, not churn.

Suggested command baseline per cycle:

- `source .venv/bin/activate && unset PYTHONPATH`
- `scope status`
- `scope workspace index`
- `sg --help >/dev/null`
- `scope refs <SYMBOL> --project <PROJECT>`
- `ruff check <PATHS>`
- `pyrefly check <PATHS>`
- `pyright <PATHS>`
- `MYPY_MEMORY_LIMIT_MB=6144 MYPY_TIMEOUT_SECONDS=600 make check WHAT=mypy PROJECT=<PROJECT>`
- `pytest <TEST_PATHS>`

Required output format per cycle:

- Family executed
- Root-cause hypothesis
- Symbols removed or centralized
- Callers updated
- Ruff result
- Pyrefly result
- Pyright result
- Mypy result
- Pytest result
- Code delta and duplication removed
- AGENTS compliance snapshot (FLEXT, Contracts, DSL, Pydantic, Constants, Tests)
- Next family started

Final success criteria:

- Measurable code-bloat reduction.
- Direct, domain-central flows through canonical facades.
- Pydantic v2 is the default contract and validation path.
- Constants, typings, models, protocols, and utilities are centralized at the lowest stable level.
- No open quality debt in the requested scope.
- Continuous production-ready status during execution.
