# mcb-scripts

<!-- TOC START -->
- [Purpose](#purpose)
- [Module Map](#module-map)
- [Collection Rules](#collection-rules)
- [Operation Flow](#operation-flow)
- [Integration Points](#integration-points)
- [Quality Gates](#quality-gates)
- [Governance Pointer](#governance-pointer)
<!-- TOC END -->

<!-- AUTO-GENERATED — DO NOT EDIT MANUALLY -->

+**Version**: `0.4.0` | **Python**: 3.13 | **Project class**: `domain`

> **Alpha (0.12.0).** This package is alpha quality. Every package in the
> workspace must be re-checked and re-validated at 0.12.0 before any promotion
> beyond alpha; treat interfaces as unstable.

## Purpose

Repository-local automation helpers for MCB

## Module Map

::: mcb_scripts
    options:
      members: false
      show_root_heading: false
      show_root_toc_entry: false
      show_source: false

## Collection Rules

Read
[`/flext/AGENTS.md`](https://github.com/flext-sh/flext/blob/0.12.0-dev/AGENTS.md)
§9 — Agent Execution Pre-requisites — for the canonical pre-change checklist
(parent FLEXT chain, Scope bootstrap, skill loading, zero-debt baseline,
slot registry verification).

## Operation Flow

- Public surface: see [`docs/index.md`](docs/index.md) and
  [`docs/api-reference/README.md`](docs/api-reference/README.md).
- Generated module overview:
  [`docs/api-reference/generated/overview.md`](docs/api-reference/generated/overview.md).
- Settings env prefix: see project `pyproject.toml` `[tool.flext]` and
  `FlextSettings` ConfigDict.

## Integration Points

- Parent FLEXT chain: read this project's `pyproject.toml` `dependencies` array
  filtered by `flext-*`. The FLEXT cascade is encoded in the inheritance lists
  of the facade classes listed under Module Map above.
- Public extensions exposed by this project: _none_.
- Library abstraction boundaries: see AGENTS.md §2.7.

## Quality Gates

Canonical `make` verbs (`check`, `test`, `fmt WHAT=apply APPLY=Y`, `val`,
`docs`) — see
[`/flext/AGENTS.md`](https://github.com/flext-sh/flext/blob/0.12.0-dev/AGENTS.md)
`Build & Test` and `Required Python quality gates`; selector routing is owned
universally by `config.AiHub.paths.agents_home`/`skills/make-check/SKILL.md`.

## Governance Pointer

- Engineering law:
  [`/flext/AGENTS.md`](https://github.com/flext-sh/flext/blob/0.12.0-dev/AGENTS.md)
- Governance + ADRs:
  [`/flext/docs/GOVERNANCE.md`](https://github.com/flext-sh/flext/blob/0.12.0-dev/docs/GOVERNANCE.md)
- Skills index:
  [`/flext/.agents/skills/`](https://github.com/flext-sh/flext/tree/0.12.0-dev/.agents/skills/)
- Onboarding:
  [`/flext/docs/guides/onboarding.md`](https://github.com/flext-sh/flext/blob/0.12.0-dev/docs/guides/onboarding.md)
- Full project portal: [`docs/index.md`](docs/index.md).
