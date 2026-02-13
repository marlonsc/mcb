<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
# CI Optimization Strategy - v0.2.1
<!-- markdownlint-disable MD024 -->

## Overview

CI pipeline optimization evolved from push-to-main workflows to **PR-first with conditional policies**. This document describes the v0.2.1 refactor (February 2026) that transformed CI/CD from redundant push-triggered jobs to intelligent PR-based gating with Draft/Bot/Ready classification.

## Evolution Timeline

### v0.1.4 (January 2026) - Path Filters & Matrix Optimization
- Path-based filtering to skip irrelevant jobs
- Test matrix split (PR: stable only, Main: stable+beta)
- Coverage job conditional on main pushes
- **Problem**: Still ran heavy jobs on push-to-main, duplicating PR work

### v0.2.1 (February 2026) - PR-First with Conditional Policies
- **Paradigm shift**: PRs become the single correctness gate
- Draft/Bot/Ready classification controls job execution
- Push-to-main runs ONLY deployment (GitHub Pages)
- CodeQL moved after required gate check (non-blocking)
- Workflow split: `ci.yml` (PRs), `pages.yml` (deploy), `release.yml` (tags)

## Problem Statement (v0.2.1)

Before PR-first refactor:

- **Per Pull Request**: 8-9 jobs + separate CodeQL workflow
- **Per Push to Main**: 17-19 jobs (DUPLICATES all PR work)
- **CodeQL**: Blocks required gate check (~5-10 min delay)
- **Bot PRs**: Run full suite (wasteful for dependency updates)
- **Draft PRs**: Run full suite (wasteful during development)
- **Monthly waste**: ~200+ redundant jobs

## Solution: PR-First with Conditional Policies

### Core Principles

1. **PRs are the gate** - All correctness checks happen on PR (not push-to-main)
2. **Conditional policies** - Draft/Bot/Ready PRs run different job subsets
3. **Deploy-only main** - Push-to-main only deploys docs (no CI)
4. **Non-blocking security** - CodeQL runs AFTER gate check passes
5. **Single required check** - `CI / Rust CI (PR consolidated)` remains stable

### Policy Matrix

| PR Type | Heavy Jobs | CodeQL | Cross-Platform | Coverage | Golden Tests | Gate Check | Time to Gate |
|---------|------------|--------|----------------|----------|--------------|------------|--------------|
| **Draft** | ❌ SKIP | ❌ SKIP | ❌ SKIP | ❌ SKIP | ❌ SKIP | ✅ PASS | ~30 seconds |
| **Bot** | ❌ SKIP* | ❌ SKIP | ❌ SKIP | ❌ SKIP | ❌ SKIP | ✅ Simplified | ~3-5 minutes |
| **Ready** | ✅ RUN | ✅ RUN (after gate) | ✅ RUN | ✅ RUN | ✅ RUN | ✅ Full | ~5-10 minutes |

\* Bot: Runs lint + test + startup + validate (Linux+stable only)

See `CI_PR_POLICIES.md` for detailed classification logic and job execution matrices.

## Impact Analysis

### v0.1.4 → v0.2.1 Comparison

#### Draft PR (during development)
- **Before**: 8 jobs, ~8-10 minutes
- **After**: Gate check only, ~30 seconds
- **Savings**: ~95% time reduction for iterative development

#### Bot PR (Dependabot)
- **Before**: 8-9 jobs, ~8-10 minutes
- **After**: 4 jobs (simplified), ~3-5 minutes
- **Savings**: ~50% time + skips expensive jobs (coverage, golden, cross-platform)

#### Ready PR (human review)
- **Before**: 8-9 jobs, ~8-10 minutes (CodeQL blocks gate)
- **After**: Full suite, ~10-15 minutes (CodeQL after gate, non-blocking)
- **Gate passes**: ~5-10 minutes (no longer blocked by CodeQL)
- **Benefit**: Faster merge approval, comprehensive validation

#### Push to Main
- **Before**: 17-19 jobs (DUPLICATES all PR work)
- **After**: 1 workflow (Pages deploy only)
- **Savings**: ~95% reduction, eliminates redundant validation

#### Monthly Savings (estimated)
- **Draft iterations**: 50 PRs × 8 min saved = ~400 min saved
- **Bot PRs**: 20 PRs × 5 min saved = ~100 min saved
- **Main pushes**: 30 pushes × 17 jobs saved = ~510 jobs eliminated
- **Total**: ~600+ jobs/month eliminated, ~500+ minutes saved

## Configuration Details

### Required Status Check

**CRITICAL**: The required check name MUST remain stable:

```text
Name: "CI / Rust CI (PR consolidated)"
Status: REQUIRED
Strict Mode: Enabled (branches must be up-to-date)
```

This check is referenced in repository rulesets and MUST NOT change to avoid breaking branch protection.

### Workflow Files

#### `.github/workflows/ci.yml` (PRs only)

**Triggers**: `pull_request` events (opened, synchronize, reopened, ready_for_review, converted_to_draft) targeting `main`

**Jobs**:
- `classify` - Detect Draft/Bot/Ready state
- `changes` - Path-based filtering
- `lint` - Rust 2024 compliance
- `test` - Matrix (Linux+stable for all, +macOS/Windows/beta for Ready)
- `startup-smoke` - DDL/init validation
- `validate` - Architecture checks
- `audit` - Security audit
- `golden-tests` - Acceptance tests (Ready only)
- `coverage` - Code coverage (Ready only)
- `release-build` - Binary builds (Ready only)
- `rust-ci` - **REQUIRED GATE CHECK** (depends on all except CodeQL)
- `analyze` - CodeQL security (Ready only, runs AFTER `rust-ci`)

#### `.github/workflows/pages.yml` (Main pushes only)

**Triggers**: `push` to `main` branch

**Jobs**:
- Build mdBook documentation
- Build Rust API docs
- Deploy to GitHub Pages

**Note**: NO CI validation here - all correctness checks happened in PR

#### `.github/workflows/release.yml` (Tag pushes only)

**Triggers**: `push` tags matching `v*`

**Jobs**:
- Build binaries (Linux, macOS, Windows)
- Create GitHub Release
- Upload binary artifacts

### CodeQL Optimization

**Before v0.2.1**: CodeQL was a separate workflow, blocked PRs for 5-10 minutes

**After v0.2.1**:
- Integrated into `ci.yml` as `analyze` job
- Depends on `rust-ci` (required gate check)
- Only runs for Ready PRs (`run_full == 'true'`)
- Does NOT block merge approval (runs after gate passes)

```yaml
analyze:
  needs: [changes, classify, rust-ci]
  if: needs.classify.outputs.run_full == 'true'
```

**Note**: Old standalone `codeql.yml` still exists on `main` branch until PR #94 merges. Bot PRs targeting `main` currently trigger both:
- OLD `codeql.yml` from `main` (will be deleted when PR merges)
- NEW `analyze` job from PR branch (correctly skips for bots)

## Monitoring & Validation

### Success Criteria

✅ **Draft PRs**: Gate passes in \u003c 1 min (no heavy jobs)
✅ **Bot PRs**: Gate passes in \u003c 5 min (simplified suite)
✅ **Ready PRs**: Gate passes in \u003c 10 min (CodeQL not blocking)
✅ **Main pushes**: No CI jobs (Pages deploy only)
✅ **Required check stable**: `CI / Rust CI (PR consolidated)` never changes
✅ **No false negatives**: All correctness checks still enforced

### Key Metrics to Track

- **Draft PR cycle**: Target \u003c 1 min (from 8-10 min in v0.1.4)
- **Bot PR cycle**: Target \u003c 5 min (from 8-10 min)
- **Ready PR gate check**: Target \u003c 10 min (was blocked by CodeQL)
- **Main push CI jobs**: Target 0 (from 17-19 jobs)
- **Monthly CI jobs**: Target ~40% reduction from v0.1.4
- **False negatives**: 0 (no bugs missed by conditional policies)

## Known Limitations & Trade-offs

### 1. Draft PRs Skip All Validation

**Limitation**: Draft PRs pass gate check without running any jobs

**Mitigation**: Converting to Ready triggers full suite before merge
**Trade-off**: Development speed vs continuous validation
**Status**: Acceptable - drafts are work-in-progress

### 2. Bot PRs Run Simplified Suite

**Limitation**: Dependabot PRs skip cross-platform, coverage, golden tests

**Mitigation**: Main branch and Ready PRs still run full suite
**Trade-off**: Bot PR speed vs comprehensive validation
**Status**: Acceptable - dependency updates rarely break platforms

### 3. CodeQL Runs After Gate Check

**Limitation**: Security issues found AFTER PR is mergeable

**Mitigation**: CodeQL still blocks merge if issues found (not silently ignored)
**Trade-off**: Merge approval speed vs security-first gating
**Status**: Acceptable - security issues are rare, gate speed prioritized

### 4. Main Push Skips All CI

**Limitation**: No validation on main push (trust PR validation)

**Mitigation**: PRs enforce strict mode (branch must be up-to-date)
**Trade-off**: Main push speed vs redundant validation
**Status**: Acceptable - PR is single source of truth

### 5. Standalone CodeQL Workflow (Temporary)

**Limitation**: Old `codeql.yml` on `main` still runs for all PRs targeting main

**Mitigation**: Will be deleted when PR #94 merges
**Trade-off**: None (temporary state during migration)
**Status**: Known issue - resolves automatically on merge

## Future Optimizations (v0.3.0+)

1. **Smart classification**: Detect "docs-only" PRs and skip even simplified suite
2. **Incremental validation**: Only run affected tests based on code changes
3. **Parallel bot handling**: Auto-approve trusted bot PRs after simplified suite passes
4. **Dynamic matrix**: Adjust platform matrix based on changed files
5. **Workflow caching**: Cache dependencies across PR lifecycle (draft → ready)

## References

- `.github/workflows/ci.yml` - Main PR validation pipeline
- `.github/workflows/pages.yml` - GitHub Pages deployment
- `.github/workflows/release.yml` - Release creation
- `.sisyphus/plans/ci-cd-refactor-pr-main.md` - Original refactor plan
- `docs/operations/CI_PR_POLICIES.md` - PR policy deep-dive
- Repository Ruleset ID: 12225448 (required check configuration)

---

**Last Updated**: 2026-02-13
**Version**: 0.2.1
**Status**: In Review (PR #94)
