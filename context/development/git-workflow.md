# Git Workflow

Last updated: 2026-02-15 (America/Sao_Paulo)

## Purpose

Document the actual git workflow, branch strategy, and CI gates for MCB.

## Branch Strategy

| Branch | Purpose | Protection |
|--------|---------|------------|
| `main` | Stable default branch | PR required, CI must pass |
| `release/v*` | Active release work (e.g., `release/v0.2.1`) | PR to main |
| `fix/*` | Bug fix branches | PR to release or main |
| `ci/*` | CI configuration changes | PR to main |
| `beads-sync` | Beads tracker auto-sync | Auto-managed |

**Current active branch**: `release/v0.2.1`

## Commit Convention

Conventional Commits format. Observed distribution (last 100 commits):

| Type | Count | Usage |
|------|-------|-------|
| `refactor` | 16 | Internal restructuring without behavior change |
| `fix` | 14 | Bug fixes (often with scope: `fix(validate)`, `fix(ci)`) |
| `feat` | 11 | New features (`feat(validate)`, `feat(docs)`) |
| `docs` | 8 | Documentation changes |
| `fix(ci)` | 7 | CI pipeline fixes |
| `chore` | 2 | Maintenance tasks |
| `test` | 1 | Test-only changes |
| `perf` | 1 | Performance improvements |

**Format**: `type(scope): description -- detail`

Examples from this repo:
```
feat(validate): add path-based rule engine for file placement validation
fix: resolve all CI gate failures -- UUID session round-trips, ORG016 violations
refactor: enforce single source of truth -- eliminate hardcoded fallbacks
```

## CI Pipeline (`.github/workflows/ci.yml`)

Triggers on PR to `main`. Smart change detection skips irrelevant jobs.

### Job Sequence

```
changes (detect) + classify (draft/bot/full)
  |-- lint (Rust 2024)
  |-- test (matrix: OS x Rust version)
  |-- startup-smoke (after lint)
  |-- validate (architecture, after smoke)
  |-- audit (cargo-audit, full only)
  |-- golden-tests (after test, full only)
  |-- coverage (after test, full only)
  |-- release-build (3 targets, full only)
  +-- rust-ci (consolidated gate)
```

### PR Classification

| PR Type | Policy |
|---------|--------|
| **Draft** | All heavy jobs skipped |
| **Bot (Dependabot)** | Simplified: lint + test + startup + validate |
| **Human (ready)** | Full suite: all 8+ jobs required |

## Session Close Protocol

```bash
git status                    # Check what changed
git add <files>               # Stage code changes
bd sync                       # Commit beads changes
git commit -m "type(scope): description"
bd sync                       # Commit any new beads changes
git push                      # Push to remote
```

**Work is NOT done until pushed.**

## Sources

- `.github/workflows/ci.yml`
- `Makefile`, `make/quality.mk`
- `git log --format='%s' -100` (commit analysis)

## Update Notes

- 2026-02-15: Full rewrite with CI pipeline detail, commit stats, branch strategy.
- 2026-02-11: Initial generic placeholder.
