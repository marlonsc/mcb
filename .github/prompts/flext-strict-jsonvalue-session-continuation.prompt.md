---
name: flext-strict-jsonvalue-session-continuation
description: "Continue a FLEXT simplification/refactoring session with aggressive removal of dict/list annotations, collections.abc-first contracts, centralized JsonValue typing, and Pydantic v2 usage through m.* models."
argument-hint: "Target scope, active failing files or commands, and any hard constraints"
agent: agent
---

You are continuing an in-progress FLEXT simplification/refactoring session. Do not restart discovery from zero. Recover the current slice quickly, fix the highest-leverage root cause, validate immediately, and keep going until the requested scope is clean.

Mandatory load order:

1. [AGENTS.md](../../AGENTS.md)
2. [FLEXT Development](../../.agents/skills/flext-development/SKILL.md)
3. Path-relevant projected skills for the touched files
4. [Python Development](../../.agents/skills/python-development/SKILL.md)
5. [Data Modeling Analysis](../../.agents/skills/data-modeling-analysis/SKILL.md) when typed model boundaries are touched
6. [TDD Workflow](../../.agents/skills/tdd-workflow/SKILL.md) when tests are touched

Session recovery protocol:

- Start from the current editor file, IDE diagnostics, latest failing terminal commands, `git diff`, and the most recently touched files.
- If the user provides failing files, errors, or commands, treat them as the primary anchor.
- Form one local root-cause hypothesis first, run one cheap discriminating check, then edit.
- After the first substantive edit, immediately run `pyrefly` and `ruff` on the touched files.
- Continue in small root-cause batches until the requested slice is green.
- Do not ask to start. Start.

Primary priorities in strict order:

1. Delete wrappers, proxies, helper chains, compatibility layers, and pass-through conversion functions.
2. Replace concrete collection annotations such as `dict`, `list`, `set`, and `tuple` with `collections.abc` contracts whenever mutability is not the real boundary requirement.
3. Prefer `Mapping`, `MutableMapping`, `Sequence`, `Collection`, `Iterable`, `Callable`, and `Set` from `collections.abc` over concrete container types.
4. Centralize transport and recursive JSON typing around canonical `t.JsonValue`, `t.JsonMapping`, `t.JsonList`, and `m.Dict`.
5. Use Pydantic v2 through canonical models and facades only: `m.BaseModel`, `m.Field`, `m.TypeAdapter`, `m.ConfigDict`, `u.computed_field`, and project `m.*` models.
6. Eliminate repeated narrowing by fixing the upstream contract shape. Prefer one validated model or adapter at the boundary over many downstream `isinstance` checks.
7. Prefer `Sequence` and `Mapping` in parameters. Only use mutable concrete forms when mutation is part of the actual public contract.
8. Remove local JSON carriers, ad-hoc recursive aliases, raw `dict[str, Any]`, and helper families such as `as_dict`, `as_list`, `as_map`, `to_dict`, `normalize_*`, and trivial `ensure_*` wrappers when canonical validation can absorb the work.
9. Remove `cast`, `Any`, bare `object`, `model_rebuild()`, direct consumer-side `pydantic` imports, and compatibility shims.
10. Preserve canonical aliases and organic FLEXT namespaces: `c`, `m`, `p`, `t`, `u`, `r`, `e`, `h`, `s`, `d`, `x`.

Implementation rules:

- Import interface types from `collections.abc` in application code.
- Treat `t.JsonValue` as the default recursive JSON transport contract.
- Validate or normalize containers once at the boundary using canonical models or `m.TypeAdapter`, then pass typed models or canonical aliases downstream.
- Prefer one central state or payload model over many tiny carrier dicts.
- Do not add a new helper if an existing canonical model, adapter, or utility can absorb the logic.
- Do not preserve legacy entry points for backward compatibility.
- Use `ast-grep` for repeated structural propagation.
- If a root cause exists in direct callers or direct callees, propagate in the same cycle.

Eliminate these patterns early:

- `dict[str, Any]`, `list[Any]`, `Mapping[str, Any]`, open `object` payloads
- helpers that only reshape containers or silence typing friction
- empty-container fallbacks without explicit canonical typing
- direct `from pydantic import ...` in consumer projects
- repeated conversion round-trips between dicts, lists, and small carrier models
- type narrowing that exists only because the boundary contract is too weak

Validation loop:

1. `pyrefly check <touched files>`
2. `ruff check <touched files>`
3. Widen to `pyright`, `mypy`, `pytest`, or `make check PROJECT=<project>` only when the slice or contract warrants it
4. If public contracts changed, keep widening until impacted callers are green

Required output on each cycle:

- root cause being attacked
- helpers or wrappers removed
- concrete collection annotations replaced by `collections.abc` contracts
- typing centralized into `t.JsonValue` or canonical `m.*` models
- validations run and exact outcome
- next file or family started

Non-negotiables:

- No workarounds
- No compatibility layers
- No partial cleanup reported as complete
- No broad rewrites without impact analysis
- No direct framework imports when canonical facades already exist
- No new helper proliferation to hide weak typing

Execution start checklist:

1. Recover context from current file, diagnostics, latest failures, and local diff.
2. Pick the smallest failing slice with the highest leverage.
3. Fix the root cause.
4. Validate immediately.
5. Continue until the requested slice is clean.

Optional context to fill when starting a new session:

- Scope: <project/module/files>
- Current failing commands: <paste here>
- Current failing files: <paste here>
- Explicit constraints: <paste here>
