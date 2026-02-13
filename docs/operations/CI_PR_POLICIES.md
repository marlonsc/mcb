<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD040 MD041 MD060 -->
# CI PR Policies - Draft/Bot/Ready Classification

## Overview

This document provides a comprehensive reference for the **PR classification system** introduced in v0.2.1. It explains how Draft, Bot, and Ready PRs are detected, what jobs execute for each type, and how to troubleshoot policy behavior.

**Target Audience**: Developers, reviewers, CI maintainers

**Related Docs**:
- [CI Optimization Strategy](./CI_OPTIMIZATION.md) - High-level strategy and impact analysis
- [CI/CD and Release Process](./CI_RELEASE.md) - Complete workflow reference

---

## Table of Contents

1. [Classification Logic](#classification-logic)
2. [Job Execution Policies](#job-execution-policies)
3. [Workflow Examples](#workflow-examples)
4. [Troubleshooting](#troubleshooting)
5. [Advanced Topics](#advanced-topics)

---

## Classification Logic

### Overview

The `classify` job in `.github/workflows/ci.yml` detects PR type and sets output variables that control downstream job execution.

### Detection Rules

```yaml
classify:
  runs-on: ubuntu-latest
  outputs:
    is_draft: ${{ github.event.pull_request.draft }}
    is_bot: ${{ github.event.pull_request.user.type == 'Bot' }}
    is_fork: ${{ github.event.pull_request.head.repo.fork }}
    run_full: ${{ steps.classify.outputs.run_full }}
    run_simplified: ${{ steps.classify.outputs.run_simplified }}
```

### Classification Algorithm

```
1. IS_DRAFT = github.event.pull_request.draft
2. IS_BOT = github.event.pull_request.user.type == 'Bot'
3. IS_FORK = github.event.pull_request.head.repo.fork

4. IF IS_DRAFT == true:
     run_full = false
     run_simplified = false
     → Draft Policy

5. ELSE IF IS_BOT == true:
     run_full = false
     run_simplified = true
     → Bot Policy

6. ELSE:
     run_full = true
     run_simplified = false
     → Ready Policy
```

### Output Variables

| Variable | Type | Purpose |
|----------|------|---------|
| `is_draft` | boolean | True if PR is in draft state |
| `is_bot` | boolean | True if PR author user type is 'Bot' |
| `is_fork` | boolean | True if PR is from a forked repository |
| `run_full` | boolean | True → run full suite (Ready PRs) |
| `run_simplified` | boolean | True → run simplified suite (Bot PRs) |

### Examples

| PR State | is_draft | is_bot | run_full | run_simplified | Policy |
|----------|----------|--------|----------|----------------|--------|
| Draft PR (human) | true | false | false | false | Draft |
| Draft PR (bot) | true | true | false | false | Draft |
| Ready PR (human) | false | false | true | false | Ready |
| Dependabot PR | false | true | false | true | Bot |
| Renovate PR | false | true | false | true | Bot |

---

## Job Execution Policies

### Policy Matrix

| Job | Draft | Bot | Ready | Condition |
|-----|-------|-----|-------|-----------|
| **classify** | ✅ | ✅ | ✅ | Always runs |
| **changes** | ✅ | ✅ | ✅ | Always runs |
| **lint** | ❌ | ✅ | ✅ | `run_full == true OR run_simplified == true` |
| **test** | ❌ | ✅ | ✅ | `run_full == true OR run_simplified == true` |
| **startup-smoke** | ❌ | ✅ | ✅ | `run_full == true OR run_simplified == true` |
| **validate** | ❌ | ✅ | ✅ | `run_full == true OR run_simplified == true` |
| **audit** | ❌ | ❌ | ✅ | `run_full == true` |
| **golden-tests** | ❌ | ❌ | ✅ | `run_full == true` AND `needs.test.result == 'success'` |
| **coverage** | ❌ | ❌ | ✅ | `run_full == true` AND `needs.test.result == 'success'` |
| **release-build** | ❌ | ❌ | ✅ | `run_full == true` |
| **rust-ci (GATE)** | ✅ | ✅ | ✅ | Always runs (required check) |
| **analyze (CodeQL)** | ❌ | ❌ | ✅ | `run_full == true` AND `needs.rust-ci.result == 'success'` |
| **auto-merge-dependabot** | ❌ | ✅* | ❌ | `user.login == 'dependabot[bot]'` |

\* Auto-merge job is in a separate workflow (`auto-reviewer.yml`)

### Draft PR Policy

**Purpose**: Enable fast iteration during development

**Jobs Executed**:
- `classify` - Detect Draft state
- `changes` - Path filtering
- `rust-ci` - **Gate check (PASSES immediately)**

**Jobs Skipped**:
- ALL validation jobs (lint, test, validate, etc.)
- ALL heavy jobs (coverage, golden, binaries)
- CodeQL security analysis

**Gate Check Logic**:
```yaml
rust-ci:
  needs: [changes, classify, lint, test, startup-smoke, validate, audit, golden-tests, coverage, release-build]
  if: always()
  # If ALL dependencies are skipped (Draft policy), gate check PASSES
  # Skipped jobs are treated as successful for dependency purposes
```

**Time to Gate**: ~30 seconds

**Use Cases**:
- Work-in-progress PRs
- Experimental branches
- Iterative development
- Code sharing before formal review

### Bot PR Policy

**Purpose**: Fast feedback for automated dependency updates

**Jobs Executed**:
- `classify` - Detect Bot user type
- `changes` - Path filtering
- `lint` - Rust 2024 compliance (Linux+stable)
- `test` - Unit + integration tests (Linux+stable ONLY, no matrix)
- `startup-smoke` - DDL/init validation
- `validate` - Architecture checks
- `rust-ci` - **Gate check (waits for above 4 jobs)**

**Jobs Skipped**:
- Cross-platform testing (macOS, Windows)
- Rust beta testing
- Coverage analysis
- Golden acceptance tests
- Release binaries
- Security audit
- CodeQL

**Test Matrix** (Simplified):
- OS: ubuntu-latest only
- Rust: stable only

**Time to Gate**: ~3-5 minutes

**Use Cases**:
- Dependabot PRs (patch/minor version bumps)
- Renovate PRs
- GitHub Actions version updates
- Automated maintenance PRs

**Bot Detection**:
Currently detects:
- `github.event.pull_request.user.type == 'Bot'`

This catches:
- `dependabot[bot]`
- `renovate[bot]`
- `github-actions[bot]`
- Any GitHub App with Bot user type

### Ready PR Policy

**Purpose**: Comprehensive validation before merge

**Jobs Executed**:
- `classify` - Detect Ready state
- `changes` - Path filtering
- `lint` - Rust 2024 compliance
- `test` - **Full cross-platform matrix**
- `startup-smoke` - DDL/init validation
- `validate` - Architecture checks
- `audit` - Security audit (cargo-audit)
- `golden-tests` - Acceptance tests
- `coverage` - Code coverage (tarpaulin)
- `release-build` - Binary builds (Linux/macOS/Windows)
- `rust-ci` - **Gate check (waits for ALL above jobs)**
- `analyze` - **CodeQL (runs AFTER gate check)**

**Test Matrix** (Full):
- OS: ubuntu-latest, macos-latest, windows-latest
- Rust: stable, beta

**Time to Gate**: ~5-10 minutes (CodeQL adds ~5-10min after)

**Use Cases**:
- Human PRs ready for review
- Non-draft PRs from team members
- PRs awaiting merge approval

---

## Workflow Examples

### Example 1: Draft PR Lifecycle

**Scenario**: Developer creates draft PR, iterates, then marks ready

```
1. Create draft PR:
   gh pr create --draft --title "feat: add feature X"
   
   → classify detects is_draft=true
   → ALL heavy jobs skip
   → rust-ci gate PASSES immediately (~30s)
   
2. Push commits (iterating):
   git push
   
   → Same behavior: gate passes in ~30s
   → Fast feedback loop
   
3. Mark ready for review:
   gh pr ready 123
   
   → classify detects is_draft=false, is_bot=false
   → run_full=true
   → Full suite triggers
   → rust-ci gate waits for all jobs (~5-10min)
   → CodeQL runs after gate passes
```

**Commands**:
```bash
# Create draft
gh pr create --draft

# Check status
gh pr view 123 --json isDraft

# Convert to ready
gh pr ready 123

# Convert back to draft
gh pr ready 123 --undo
```

### Example 2: Dependabot PR

**Scenario**: Dependabot opens PR for minor version bump

```
1. Dependabot creates PR:
   PR opened by dependabot[bot]
   
   → classify detects is_bot=true
   → run_simplified=true
   → Lint + Test (Linux+stable) + Startup + Validate
   → rust-ci gate waits for these 4 jobs (~3-5min)
   → CodeQL SKIPPED
   → Coverage SKIPPED
   → Cross-platform SKIPPED
   
2. Gate check passes:
   → auto-reviewer.yml workflow triggers
   → Checks if patch/minor update
   → Enables auto-merge if appropriate
   
3. Auto-merge completes:
   → PR merges automatically
   → Main push triggers pages.yml (docs deploy)
```

**Commands**:
```bash
# View Dependabot PRs
gh pr list --author app/dependabot

# Check classification
gh run view \u003crun-id\u003e --log | grep "is_bot"

# Manual merge (if auto-merge disabled)
gh pr merge \u003cpr-number\u003e --squash
```

### Example 3: Ready PR with CodeQL

**Scenario**: Human opens PR, full suite runs including CodeQL

```
1. Create PR:
   gh pr create --title "fix: resolve bug Y"
   
   → classify detects run_full=true
   → Full suite starts
   
2. Jobs execute in parallel:
   lint, test (full matrix), validate, audit, startup-smoke
   ↓
   After test completes:
   golden-tests, coverage, release-build
   
3. rust-ci gate check:
   → Waits for ALL jobs above
   → Gate PASSES after ~5-10 minutes
   → **PR now mergeable** (gate check satisfied)
   
4. CodeQL (analyze job):
   → Depends on rust-ci (waits for gate)
   → Starts AFTER gate passes
   → Runs for ~5-10 additional minutes
   → **Does NOT block merge** (not a required check)
```

**Timeline**:
```
t=0:     PR opened, jobs start
t=3-5m:  Lint, startup, validate complete
t=5-8m:  Test matrix completes
t=8-10m: Coverage, golden, binaries complete
t=10m:   rust-ci GATE CHECK PASSES → PR mergeable
t=10m:   CodeQL starts (non-blocking)
t=15m:   CodeQL completes (optional)
```

**Commands**:
```bash
# Watch CI progress
gh run watch \u003crun-id\u003e

# Check gate check status
gh pr checks 123 | grep "Rust CI"

# Merge after gate passes (don't wait for CodeQL)
gh pr merge 123 --squash
```

---

## Troubleshooting

### Draft PR Running Full Suite

**Problem**: Draft PR executes lint, test, and other heavy jobs

**Diagnosis**:
```bash
# Check if PR is actually draft
gh pr view \u003cpr-number\u003e --json isDraft
# Expected: {"isDraft": true}

# Check classify job output
gh run view \u003crun-id\u003e --log | grep "is_draft"
# Expected: is_draft=true

# Check run_full output
gh run view \u003crun-id\u003e --log | grep "run_full"
# Expected: run_full=false
```

**Common Causes**:
1. PR not marked as draft in GitHub UI
2. Classify job failed to execute
3. Workflow file syntax error

**Solutions**:
```bash
# Convert to draft
gh pr ready \u003cpr-number\u003e --undo

# Or via web UI:
# PR page → Convert to draft

# Re-run workflow
gh run rerun \u003crun-id\u003e
```

### Bot PR Running Full Suite

**Problem**: Dependabot PR executes cross-platform tests, coverage, CodeQL

**Diagnosis**:
```bash
# Check user type
gh api /repos/marlonsc/mcb/pulls/\u003cpr-number\u003e | jq '.user.type'
# Expected: "Bot"

# Check classify job output
gh run view \u003crun-id\u003e --log | grep "is_bot"
# Expected: is_bot=true

# Check run_simplified output
gh run view \u003crun-id\u003e --log | grep "run_simplified"
# Expected: run_simplified=true
```

**Common Causes**:
1. User type not detected as 'Bot'
2. Workflow condition logic error
3. Job `if` conditions incorrect

**Solutions**:
```bash
# Verify Dependabot user type
gh api /repos/marlonsc/mcb/pulls/\u003cpr-number\u003e | jq '.user.login, .user.type'
# Should show: "dependabot[bot]", "Bot"

# If user type is wrong, check workflow file:
# .github/workflows/ci.yml line 74:
# IS_BOT: ${{ github.event.pull_request.user.type == 'Bot' }}
```

### CodeQL Blocking Merge

**Problem**: PR cannot merge because CodeQL is running or failed

**Diagnosis**:
```bash
# Check required status checks
gh api /repos/marlonsc/mcb/branch-protection/main | jq '.required_status_checks.contexts'
# Should NOT include "Analyze (rust)" or CodeQL jobs

# Check repository ruleset
gh api /repos/marlonsc/mcb/rulesets | jq '.[] | select(.name == "main")'
```

**Expected Behavior**:
- CodeQL (`analyze` job) is NOT a required check
- Only `CI / Rust CI (PR consolidated)` is required
- PRs can merge while CodeQL is running

**If CodeQL is blocking**:
This indicates a configuration error in repository rulesets.

**Solution**:
```bash
# Repository settings → Rules → Rulesets
# Edit "main" ruleset
# Required status checks should ONLY list:
#   - "CI / Rust CI (PR consolidated)"
# Remove any CodeQL/Analyze checks from required list
```

### Gate Check Failing on Draft PR

**Problem**: `rust-ci` job fails on draft PR

**Expected Behavior**: Gate check should PASS on draft PRs (all dependencies skipped)

**Diagnosis**:
```bash
# Check gate check logic
gh run view \u003crun-id\u003e --log -j "Rust CI (PR consolidated)"

# Look for job dependency results
# All should be "skipped" for draft PRs
```

**Common Causes**:
1. A job ran that should have been skipped
2. A job failed before being skipped
3. `if: always()` condition missing on rust-ci job

**Solution**:
Check `.github/workflows/ci.yml`:
```yaml
rust-ci:
  needs: [changes, classify, lint, test, ...]
  if: always()  # ← MUST be present
  # This allows gate to pass even if all dependencies skip
```

### Jobs Not Skipping on Draft PR

**Problem**: Jobs like `lint`, `test` execute on draft PR when they should skip

**Diagnosis**:
```bash
# Check job conditions
gh run view \u003crun-id\u003e --json jobs | jq '.jobs[] | {name, conclusion}'

# Jobs should show "conclusion": "skipped" for:
# - lint
# - test
# - validate
# - coverage
# - golden-tests
# - etc.
```

**Common Causes**:
1. Job `if` condition incorrect or missing
2. `run_full` or `run_simplified` outputs not set correctly

**Solution**:
Check job conditions in `.github/workflows/ci.yml`:
```yaml
test:
  needs: [changes, classify]
  if: |
    (needs.classify.outputs.run_full == 'true' || 
     needs.classify.outputs.run_simplified == 'true')
  # ↑ This condition MUST be present
```

### Full Suite Not Running on Ready PR

**Problem**: Ready PR (non-draft) skips coverage, golden tests, or cross-platform

**Diagnosis**:
```bash
# Verify PR is not draft
gh pr view \u003cpr-number\u003e --json isDraft
# Expected: {"isDraft": false}

# Check classify outputs
gh run view \u003crun-id\u003e --log | grep -E "(run_full|is_draft)"
# Expected: is_draft=false, run_full=true
```

**Common Causes**:
1. PR converted to draft accidentally
2. Classify job outputs incorrect
3. Path filtering excluding all changes

**Solution**:
```bash
# If PR is draft, convert to ready
gh pr ready \u003cpr-number\u003e

# Check path filtering
gh run view \u003crun-id\u003e --log -j "Detect Changes"
# Verify src=true for code changes
```

---

## Advanced Topics

### Path Filtering Integration

Jobs combine classification AND path filtering:

```yaml
test:
  needs: [changes, classify]
  if: |
    (needs.classify.outputs.run_full == 'true' || 
     needs.classify.outputs.run_simplified == 'true') &&
    needs.changes.outputs.src == 'true'
```

**Behavior**:
- If code changes (`src=true`): classification policy applies
- If no code changes (`src=false`): job skips regardless of classification

**Example**: Draft PR with docs-only changes
- `is_draft=true` → run_full=false
- `src=false` → even if run_full were true, jobs would skip

### Fork PR Handling

Fork PRs have special restrictions:

```yaml
classify:
  outputs:
    is_fork: ${{ github.event.pull_request.head.repo.fork }}
```

**Fork PR Restrictions**:
- No access to repository secrets
- Limited permissions for security
- Cannot trigger certain workflows

**Jobs affected**:
```yaml
coverage:
  if: |
    needs.classify.outputs.run_full == 'true' &&
    needs.classify.outputs.is_fork == 'false'
```

Coverage requires upload to Codecov (needs secrets), so forks skip it.

### CodeQL Timing

CodeQL (`analyze` job) is strategically positioned:

```yaml
analyze:
  needs: [changes, classify, rust-ci]  # Depends on gate check
  if: needs.classify.outputs.run_full == 'true'
```

**Why AFTER rust-ci**:
1. **Non-blocking merge**: Gate passes before CodeQL starts
2. **Saves time**: No need to wait for CodeQL to merge
3. **Still enforced**: CodeQL failures still reported, just not blocking

**Timeline**:
```
0-5min:  Lint, test, validate (parallel)
5-10min: Coverage, golden, binaries
10min:   rust-ci GATE PASSES → PR mergeable
10min:   CodeQL STARTS (non-blocking)
15min:   CodeQL completes
```

### Concurrency Groups

Each workflow has concurrency settings:

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.head_ref || github.ref }}
  cancel-in-progress: true
```

**Behavior**:
- New push cancels previous run for same PR
- Saves runner time
- Prevents queue buildup

**Example**:
```text
1. Push commit A → CI run starts
2. Push commit B → Run A cancelled, Run B starts
3. Only Run B completes
```

### Auto-Merge Integration

Dependabot PRs integrate with auto-merge workflow:

**`.github/workflows/auto-reviewer.yml`**:
```yaml
auto-merge-dependabot:
  if: github.event.pull_request.user.login == 'dependabot[bot]'
  steps:
    - name: Enable auto-merge for patch and minor
      if: |
        steps.metadata.outputs.update-type == 'version-update:semver-patch' ||
        steps.metadata.outputs.update-type == 'version-update:semver-minor'
      run: gh pr merge --auto --squash "$PR_URL"
```

**Flow**:
1. Dependabot opens PR
2. CI runs (bot policy: simplified suite)
3. Gate check passes (~3-5min)
4. Auto-reviewer enables auto-merge
5. PR merges automatically when approved

**Major updates**:
- Require manual review (no auto-merge)
- Comment added explaining manual review needed

---

## Policy Comparison Table

### Job Execution Summary

| Job | Draft | Bot | Ready | Why Draft Skips | Why Bot Skips Some |
|-----|-------|-----|-------|-----------------|---------------------|
| classify | ✅ | ✅ | ✅ | Always needed | Always needed |
| changes | ✅ | ✅ | ✅ | Path filtering | Path filtering |
| lint | ❌ | ✅ | ✅ | WIP code, fast iteration | Run for bots |
| test | ❌ | ✅ (simple) | ✅ (full) | WIP code | Deps rarely break platforms |
| startup-smoke | ❌ | ✅ | ✅ | WIP code | Important for deps |
| validate | ❌ | ✅ | ✅ | WIP code | Architecture checks critical |
| audit | ❌ | ❌ | ✅ | WIP code | Bots don't introduce vulns |
| golden-tests | ❌ | ❌ | ✅ | WIP code | Acceptance tests expensive |
| coverage | ❌ | ❌ | ✅ | WIP code | Coverage tracking for humans |
| release-build | ❌ | ❌ | ✅ | WIP code | Cross-compile expensive |
| rust-ci (gate) | ✅ | ✅ | ✅ | Required check | Required check |
| analyze (CodeQL) | ❌ | ❌ | ✅ (after) | Security not needed | Bots don't write unsafe code |

### Time Comparison

| Metric | Draft | Bot | Ready |
|--------|-------|-----|-------|
| **Time to Gate** | ~30 seconds | ~3-5 minutes | ~5-10 minutes |
| **Total CI Time** | ~30 seconds | ~3-5 minutes | ~15-20 minutes* |
| **Jobs Executed** | 2 | 6 | 13 |
| **Platforms Tested** | 0 | 1 (Linux) | 3 (Linux/macOS/Windows) |
| **Rust Versions** | 0 | 1 (stable) | 2 (stable/beta) |

\* Total includes CodeQL which runs after gate (non-blocking)

### Cost Comparison (Estimated)

Based on GitHub Actions runner minutes:

| PR Type | Runner Minutes | Monthly (est.) | Annual (est.) |
|---------|----------------|----------------|---------------|
| **Draft** (50/month) | 0.5 min/PR | 25 min | 300 min |
| **Bot** (20/month) | 4 min/PR | 80 min | 960 min |
| **Ready** (30/month) | 15 min/PR | 450 min | 5400 min |
| **TOTAL** | - | 555 min/month | 6660 min/year |

**v0.1.4 Comparison** (all PRs ran full suite):
- 100 PRs/month × 10 min = 1000 min/month
- **Savings**: 445 min/month (44% reduction)

---

## See Also

- [CI Optimization Strategy](./CI_OPTIMIZATION.md) - Strategic overview and impact analysis
- [CI/CD and Release Process](./CI_RELEASE.md) - Complete workflow reference
- `.github/workflows/ci.yml` - Workflow implementation
- Repository Ruleset ID: 12225448 - Branch protection configuration

---

**Last Updated**: 2026-02-13
**Version**: 0.2.1
**Status**: Current (In Review - PR #94)
