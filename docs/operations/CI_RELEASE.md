<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# CI/CD and Release Process - v0.2.1
<!-- markdownlint-disable MD024 -->

## Overview

This document describes the **PR-first CI/CD pipeline** and automated release process for Memory Context Browser (v0.2.1+). The system is designed for enterprise-grade quality gates with intelligent conditional execution.

**Key Principles**:

- **PRs are the single source of truth** - All validation happens on pull requests
- **Conditional policies** - Draft/Bot/Ready PRs run different job sets
- **Deploy-only main** - Push-to-main triggers ONLY deployment (no redundant CI)
- **Tag-triggered releases** - Semantic versioning with automated binary distribution

## Architecture

```ascii
┌─────────────────┐
│  Pull Request   │──┬─→ Draft PR: Gate check only (~30s)
│   (to main)     │  ├─→ Bot PR: Simplified suite (~3-5min)
└─────────────────┘  └─→ Ready PR: Full suite (~10-15min)
                          ├─ Cross-platform matrix
                          ├─ Coverage + Golden tests
                          ├─ Release binaries
                          └─ CodeQL (after gate, non-blocking)

┌─────────────────┐
│ Push to main    │──→ GitHub Pages Deploy ONLY
│  (after merge)  │    (No CI - validated in PR)
└─────────────────┘

┌─────────────────┐
│  Tag push (v*)  │──→ Release Workflow
│                 │    ├─ Build binaries (Linux/macOS/Windows)
└─────────────────┘    └─ Create GitHub Release
```

## Table of Contents

1. [Local Validation (Pre-commit)](#local-validation-pre-commit)
2. [PR-First CI Pipeline](#pr-first-ci-pipeline)
3. [GitHub Pages Deployment](#github-pages-deployment)
4. [Automated Releases](#automated-releases)
5. [Workflow Files Reference](#workflow-files-reference)
6. [Troubleshooting](#troubleshooting)

---

## Local Validation (Pre-commit)

### Installing Git Hooks

Install pre-commit hooks that validate code before each commit:

```bash
cp scripts/hooks/pre-commit .git/hooks/ && chmod +x .git/hooks/pre-commit
```

This installs `.git/hooks/pre-commit` which runs validation checks automatically.

### What Pre-commit Validates

The pre-commit hook runs the same checks as the CI pipeline but **skips tests** for fast feedback (\u003c 30 seconds typical):

```bash
# Step 1: Lint checks (Rust 2024 compliance)
make lint MCB_CI=1

# Step 2: Architecture validation (QUICK mode, no tests)
make validate QUICK=1
```

**Validation includes**:

- Format check (rustfmt)
- Clippy lints with Rust 2024 edition compatibility
- Architecture validation (imports, dependencies, layer boundaries)
- No test execution (tests run in CI after push)

### Running Pre-commit Manually

To run pre-commit validation without committing:

```bash
# Run exactly what pre-commit hook runs
make lint MCB_CI=1 && make validate QUICK=1

# Or run full CI pipeline locally (matches GitHub exactly)
make ci
```

### Bypassing Pre-commit (Not Recommended)

If you need to bypass pre-commit checks temporarily:

```bash
git commit --no-verify
```

⚠️ **Warning**: The commit will still fail in GitHub CI if it doesn't pass validation.

---

## PR-First CI Pipeline

### Overview

**v0.2.1 introduces PR-first gating** - all correctness checks happen on pull requests, not on push-to-main. This eliminates redundant validation and provides intelligent conditional execution based on PR type.

### Workflow File

**`.github/workflows/ci.yml`**

**Triggers**: `pull_request` events (opened, synchronize, reopened, ready_for_review, converted_to_draft) targeting `main`

**Required Status Check**: `CI / Rust CI (PR consolidated)` - This is the ONLY required check and MUST NOT change (referenced in repository rulesets).

### PR Classification

The `classify` job determines which jobs to run based on PR state:

| Classification | Detection | Behavior |
| ---------------- | ----------- | ---------- |
| **Draft PR** | `github.event.pull_request.draft == true` | Skip all heavy jobs, gate passes immediately |
| **Bot PR** | `github.event.pull_request.user.type == 'Bot'` | Run simplified suite (Linux+stable only) |
| **Ready PR** | Non-draft, non-bot | Run full suite (cross-platform, coverage, golden, binaries) |

### Job Execution Matrix

| Job | Draft | Bot | Ready | Purpose |
| ----- | ------- | ----- | ------- | --------- |
| `classify` | ✅ | ✅ | ✅ | Detect PR type |
| `changes` | ✅ | ✅ | ✅ | Path-based filtering |
| `lint` | ❌ | ✅ | ✅ | Format + clippy (Rust 2024) |
| `test` | ❌ | ✅ (Linux+stable) | ✅ (full matrix) | Unit + integration tests |
| `startup-smoke` | ❌ | ✅ | ✅ | DDL/init validation |
| `validate` | ❌ | ✅ | ✅ | Architecture checks |
| `audit` | ❌ | ❌ | ✅ | Security audit |
| `golden-tests` | ❌ | ❌ | ✅ | Acceptance tests |
| `coverage` | ❌ | ❌ | ✅ | Code coverage |
| `release-build` | ❌ | ❌ | ✅ | Binary builds (3 platforms) |
| `rust-ci` (GATE) | ✅ | ✅ | ✅ | **Required gate check** |
| `analyze` (CodeQL) | ❌ | ❌ | ✅ (after gate) | Security analysis |

### Gate Check Logic

The `rust-ci` job is the **required gate check** that enforces branch protection. It succeeds based on PR type:

**Draft PRs**:

- All heavy jobs skipped
- Gate check passes immediately (no dependencies failed)
- Merge approval granted in ~30 seconds
- **Purpose**: Enables fast iteration during development

**Bot PRs (Dependabot)**:

- Runs simplified suite: lint + test (Linux+stable only) + startup + validate
- Gate check waits for these 4 jobs
- Passes in ~3-5 minutes
- **Purpose**: Fast feedback for dependency updates without expensive cross-platform testing

**Ready PRs (Human)**:

- Runs full suite: all jobs except `analyze` (CodeQL)
- Gate check waits for: lint, test (all platforms), startup, validate, audit, golden, coverage, release-build
- Passes in ~5-10 minutes
- **Purpose**: Comprehensive validation before merge

**CodeQL (analyze job)**:

- Depends on `rust-ci` gate check
- Runs AFTER gate passes (non-blocking)
- Only executes for Ready PRs
- **Purpose**: Security analysis without delaying merge approval

### Test Matrix

**Ready PRs** run a full cross-platform matrix:

| Dimension | Values |
| ----------- | -------- |
| **OS** | ubuntu-latest, macos-latest, windows-latest |
| **Rust** | stable, beta |

**Bot/Simplified PRs** run:

- OS: ubuntu-latest only
- Rust: stable only

### Viewing CI Results

```bash
# List recent CI runs
gh run list --workflow=ci.yml --limit=5

# View specific run
gh run view \u003crun-id\u003e

# Watch a run in real-time
gh run watch \u003crun-id\u003e

# View PR checks
gh pr checks \u003cpr-number\u003e
```

### Local CI Matching

To run the **exact same pipeline locally** before pushing:

```bash
# Full CI pipeline (matches Ready PR)
make ci

# This runs:
# 1. Lint (Rust 2024 compliance)
# 2. Unit and integration tests (4 threads to prevent timeouts)
# 3. Architecture validation (strict mode)
# 4. Golden acceptance tests (2 threads)
# 5. Security audit
# 6. Documentation build
```

---

## GitHub Pages Deployment

### Overview

GitHub Pages deployment is **decoupled from CI** - it runs ONLY on push-to-main, after PR validation is complete. This eliminates redundant validation.

### Workflow File

**`.github/workflows/pages.yml`**

**Triggers**: `push` to `main` branch

**Jobs**:

1. Build mdBook documentation
2. Build Rust API docs (rustdoc)
3. Combine outputs
4. Deploy to GitHub Pages

**Permissions**: Minimal - `contents: read`, `pages: write`, `id-token: write`

### What Gets Deployed

```ascii
https://marlonsc.github.io/mcb/
├── /                   # mdBook (user guide, architecture, ops docs)
├── /api/               # Rust API documentation (rustdoc)
└── /diagrams/          # Architecture diagrams (PlantUML)
```

### Manual Pages Deployment

Normally automatic, but you can trigger manually:

```bash
# Push to main triggers pages workflow automatically
git push origin main

# Or create workflow_dispatch event (if enabled)
gh workflow run pages.yml
```

### Pages URL

Live documentation is available at:

```ascii
https://marlonsc.github.io/mcb/
```

---

## Automated Releases

### Overview

Releases are triggered by **tag pushes matching `v*` pattern**. The release workflow builds binaries for all platforms and creates a GitHub Release with downloadable artifacts.

### Workflow File

**`.github/workflows/release.yml`**

**Triggers**: `push` tags matching `v*` (e.g., `v0.2.1`, `v1.0.0`)

**Jobs**:

1. Build release binaries (Linux, macOS, Windows)
2. Create GitHub Release
3. Upload binary artifacts
4. Generate changelog from git log

### Release Process

#### Step 1: Version Bump

```bash
# Bump version in Cargo.toml (choose one)
make version BUMP=patch  # 0.2.0 → 0.2.1
make version BUMP=minor  # 0.2.1 → 0.3.0
make version BUMP=major  # 0.2.1 → 1.0.0

# Or manually edit Cargo.toml
vim Cargo.toml  # Update version = "0.2.1"
```

#### Step 2: Commit Version Bump

```bash
# Commit the version change
git add Cargo.toml Cargo.lock
git commit -m "chore(release): bump version to v0.2.1"
git push
```

#### Step 3: Create and Push Tag

```bash
# Create annotated tag
git tag -a v0.2.1 -m "Release v0.2.1"

# Push tag to trigger release workflow
git push origin v0.2.1
```

#### Step 4: Monitor Release Workflow

```bash
# Watch release workflow
gh run list --workflow=release.yml --limit=1
gh run watch \u003crun-id\u003e
```

### Release Artifacts

Each release includes pre-compiled binaries:

| Platform | Binary Name | Target Triple |
| ---------- | ------------- | --------------- |
| **Linux** | `mcb-x86_64-unknown-linux-gnu` | x86_64-unknown-linux-gnu |
| **macOS** | `mcb-x86_64-apple-darwin` | x86_64-apple-darwin |
| **Windows** | `mcb-x86_64-pc-windows-msvc.exe` | x86_64-pc-windows-msvc |

### Release Notes

Automatically generated from:

- Git log since previous tag
- Conventional commit messages
- CHANGELOG.md (if updated manually)

### Downloading Releases

Releases are available at:

```ascii
https://github.com/marlonsc/mcb/releases
```

Or via CLI:

```bash
# List releases
gh release list

# Download latest release
gh release download

# Download specific version
gh release download v0.2.1
```

---

## Workflow Files Reference

### `.github/workflows/ci.yml` (PR Validation)

**Purpose**: Validate pull requests with conditional policies

**Key Features**:

- PR type classification (Draft/Bot/Ready)
- Path-based filtering (skip CI for docs-only changes)
- Cross-platform testing (Linux/macOS/Windows)
- Required gate check (`rust-ci`)
- CodeQL security analysis (non-blocking)

**Concurrency**: `group: ci-${{ github.head_ref }}`, `cancel-in-progress: true`

**Timeouts**:

- Most jobs: 15 minutes
- Test job: 30 minutes
- Coverage: 30 minutes
- Release build: 20 minutes

### `.github/workflows/pages.yml` (Documentation Deployment)

**Purpose**: Deploy documentation to GitHub Pages

**Key Features**:

- Builds mdBook + rustdoc
- Combines outputs into single site
- Deploys to `gh-pages` branch

**Concurrency**: `group: pages`, `cancel-in-progress: false` (no cancellation - deployments must complete)

**Timeouts**: 20 minutes per job

### `.github/workflows/release.yml` (Binary Distribution)

**Purpose**: Create GitHub Releases with binary artifacts

**Key Features**:

- Builds release binaries (optimized, stripped)
- Cross-platform compilation
- Automatic changelog generation
- Artifact upload

**Concurrency**: `group: release-${{ github.ref }}`, `cancel-in-progress: false` (releases must complete)

**Timeouts**: 30 minutes per build job

### `.github/workflows/auto-reviewer.yml` (Dependabot Automation)

**Purpose**: Auto-merge Dependabot PRs (patch/minor updates)

**Triggers**: Dependabot PRs only (`user.login == 'dependabot[bot]'`)

**Actions**:

- Fetch Dependabot metadata
- Enable auto-merge for patch/minor updates
- Require manual review for major updates

---

## Troubleshooting

### Draft PR Not Skipping Jobs

**Problem**: Draft PR runs full suite instead of gate check only

**Solution**:

```bash
# Verify PR is marked as draft
gh pr view \u003cpr-number\u003e --json isDraft

# If not draft, convert it
gh pr ready \u003cpr-number\u003e --undo

# Check classify job output
gh run view \u003crun-id\u003e --log | grep "is_draft"
```

### Bot PR Running Full Suite

**Problem**: Dependabot PR runs coverage/golden tests

**Solution**:

```bash
# Verify user type detection
gh api /repos/marlonsc/mcb/pulls/\u003cpr-number\u003e | jq '.user.type'

# Should return "Bot"
# If not, check classify job logic in .github/workflows/ci.yml
```

### CodeQL Blocking Merge

**Problem**: CodeQL job prevents PR from merging

**Cause**: CodeQL is NOT a required check. If it's blocking, check repository ruleset configuration.

**Solution**:

```bash
# Verify required checks
gh api /repos/marlonsc/mcb/rulesets | jq '.[] | select(.name == "main") | .rules[] | select(.type == "required_status_checks")'

# Should ONLY show: "CI / Rust CI (PR consolidated)"
# CodeQL should NOT be in required checks list
```

### CI Fails But Pre-commit Passed Locally

**Possible Causes**:

1. Different environment (macOS vs Linux)
2. Different Rust versions
3. Cache issues
4. Race conditions in tests

**Solutions**:

```bash
# Run exact CI validation locally
make ci

# Clear cache and rebuild
make clean
cargo build

# Check Rust version matches
rustc --version  # Should be stable
```

### Tests Timeout in CI

**Problem**: `test` job timeout after 30 minutes

**Solutions**:

1. **Increase timeout** in `.github/workflows/ci.yml`:

   ```yaml
   timeout-minutes: 45
   ```

2. **Reduce parallelization**:

   ```yaml
   - run: make test THREADS=2
   ```

3. **Run tests locally** to identify slow tests:

   ```bash
   make test THREADS=4 VERBOSE=1
   ```

### Release Build Fails

**Problem**: `release-build` job fails with compilation error

**Checklist**:

1. All CI checks passed before release job?
2. Built locally successfully?

   ```bash
   make build RELEASE=1
   ```

3. No uncommitted changes?

   ```bash
   git status
   ```

### GitHub Release Not Created

**Problem**: Tag pushed but release workflow didn't complete

**Solution**:

```bash
# View release workflow runs
gh run list --workflow=release.yml --limit=5

# View specific run logs
gh run view \u003crun-id\u003e --log

# Common issues:
# - Pre-release validation failed (check test/lint/audit logs)
# - Tag format incorrect (must be v* like v0.2.1)
# - Artifacts failed to upload (check permissions)
```

### Pages Deployment Stuck

**Problem**: Pages workflow running but site not updating

**Solution**:

```bash
# Check pages workflow status
gh run list --workflow=pages.yml --limit=3

# View deployment status
gh api /repos/marlonsc/mcb/pages/builds/latest

# Common issues:
# - Pages not enabled in repo settings
# - Permissions insufficient (needs pages: write)
# - Branch protection preventing gh-pages push
```

---

## CI/CD Best Practices

### For Developers

1. **Use Draft PRs during development**
   - Faster iteration (~30s gate check vs ~10min full suite)
   - Convert to Ready when seeking review

2. **Run `make ci` before pushing**
   - Catches issues locally before CI
   - Saves CI runner time

3. **Keep PRs up-to-date**
   - Strict mode requires branch to be current with main
   - Rebase frequently: `git pull --rebase origin main`

4. **Don't bypass pre-commit hooks**
   - They exist to catch issues early
   - Bypassing wastes CI time

### For Reviewers

1. **Wait for gate check to pass**
   - `rust-ci` job is the required check
   - CodeQL runs after, don't wait for it

2. **Check classification is correct**
   - Draft: Only gate check should run
   - Bot: Simplified suite
   - Ready: Full suite

3. **Review CodeQL findings**
   - Security analysis runs after gate
   - Not blocking, but should be addressed

### For Release Managers

1. **Always tag from main**
   - Ensure PR merged and validated before tagging
   - Never tag from feature branches

2. **Use semantic versioning**
   - Patch: Bug fixes (0.2.0 → 0.2.1)
   - Minor: New features (0.2.1 → 0.3.0)
   - Major: Breaking changes (0.2.1 → 1.0.0)

3. **Verify release artifacts**
   - Download binaries after release
   - Test on each platform
   - Validate changelog accuracy

---

## Command Reference

### Local Development

```bash
# Install pre-commit hooks
cp scripts/hooks/pre-commit .git/hooks/ && chmod +x .git/hooks/pre-commit

# Pre-commit validation (lint + validate QUICK)
make lint MCB_CI=1 && make validate QUICK=1

# Full CI pipeline (matches Ready PR)
make ci

# Individual checks
make lint MCB_CI=1        # Format & clippy
make test                 # All tests
make validate             # Architecture validation
make audit                # Security audit
make docs                 # Documentation build
make coverage MCB_CI=1    # Code coverage
```

### PR Management

```bash
# Create draft PR
gh pr create --draft --title "feat: ..." --body "..."

# Convert to ready for review
gh pr ready \u003cpr-number\u003e

# Convert back to draft
gh pr ready \u003cpr-number\u003e --undo

# Check PR status
gh pr view \u003cpr-number\u003e
gh pr checks \u003cpr-number\u003e
```

### CI Monitoring

```bash
# View workflow runs
gh run list --workflow=ci.yml --limit=5

# Watch a run
gh run watch \u003crun-id\u003e

# View job logs
gh run view \u003crun-id\u003e --log -j \u003cjob-name\u003e
```

### Release Management

```bash
# Bump version
make version BUMP=patch

# Create and push tag
git tag -a v0.2.1 -m "Release v0.2.1"
git push origin v0.2.1

# List releases
gh release list

# Download release
gh release download v0.2.1
```

---

## See Also

- [CI Optimization Strategy](./CI_OPTIMIZATION.md) - v0.2.1 PR-first details
- [CI PR Policies](./CI_PR_POLICIES.md) - Draft/Bot/Ready deep-dive
- [Deployment Guide](./DEPLOYMENT.md) - Installation and configuration
- [CHANGELOG](./CHANGELOG.md) - Release history
- [Architecture](../architecture/ARCHITECTURE.md) - System design

---

**Last Updated**: 2026-02-13
**Version**: 0.2.1
**Status**: Current (In Review - PR #94)
