# ADR 058: Gas Town Workflow Adoption

## Status

Accepted

## Context

The MCB project was previously bound to the FLEXT workflow infrastructure
(`flext-infra` codegen, `make work`, pre-commit hook gating, `flext-cli` /
`flext-core` Python packages). That coupling created duplicate machinery with
Gas Town (`gt sling` / `gt done`, Refinery, `bd hooks`) and prevented the repo
from landing cleanly on GitHub under the operator's standard flow.

The operator required a complete cutover: disable flext pre-commit / pre-push /
`make work` requirements, migrate docs/skills/agents/rules/commands/ADRs to
canonical Gas Town, and remove all incompatible or duplicate flext/ai-hub
machinery.

## Decision

Adopt Gas Town as the sole workflow owner and remove the flext-layer
user-facing requirements:

- Lane lifecycle moves from `make work WHAT=start|status|land|finish` to
  `gt sling` / `gt done` inside the Refinery lane.
- Git hook ownership moves from `.pre-commit-config.yaml` / `pre-commit install`
  to `bd hooks install` (pre-commit, post-merge, pre-push, post-checkout,
  prepare-commit-msg).
- The `work` Make verb is removed from the public verb allowlist.
- `.pre-commit-config.yaml` is deleted from the repo.
- Docs, CONTRIBUTING, ONBOARDING, ARCHITECTURE_BOUNDARIES, and ADR-036 are
  updated to reference Gas Town / beads hooks.
- `FLEXT-INFRA-FIX-REQUEST.md` is deleted (obsolete flext defect doc).
- Test surface `tests/python/scripts_lib/test_make_surface.py` is updated to
  assert Gas Town hook ownership and the absence of `make work`.

## Consequences

### Positive

- Single workflow owner (Gas Town) instead of parallel FLEXT + Gas Town
  machinery.
- Lane lifecycle, bead tracker, and git hooks all route through one socket
  executor per event type.
- Repo lands on GitHub without flext pre-commit / `make work` gating.

### Negative

- The underlying `flext-infra` generated artifacts (Makefile, CI workflows,
  `.mise.toml`) remain in place as functional infrastructure until a follow-up
  migration replaces them with native equivalents.
- Python developer scripts under `src/mcb_scripts/` that import `flext_core`
  / `flext_cli` remain functional only while those pinned dependencies are
  present in `pyproject.toml`.
