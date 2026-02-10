---
adr: 36
title: Enforcement Layer — Policies and Guards
status: ACCEPTED
created: 
updated: 2026-02-06
related: [23, 25, 29]
supersedes: []
superseded_by: []
implementation_status: Complete
---

## ADR-036: Enforcement Layer — Policies and Guards

## Status

**Proposed** — 2026-02-05

-   **Deciders:** Project team
-   **Depends on:** [ADR-034](./034-workflow-core-fsm.md) (Workflow Core FSM), [ADR-035](./035-context-scout.md) (Context Scout)
-   **Related:** [ADR-029](./029-hexagonal-architecture-dill.md) (Hexagonal DI), [ADR-023](./023-inventory-to-linkme-migration.md) (linkme), [ADR-025](./025-figment-configuration.md) (Figment)
-   **Series:** [ADR-034](./034-workflow-core-fsm.md) → [ADR-035](./035-context-scout.md) → **ADR-036** → [ADR-037](./037-workflow-orchestrator.md)

## Context

ADR-034 defines the workflow FSM with state transitions. ADR-035 provides typed `ProjectContext` snapshots. Before a transition is executed, the system must validate that project conditions are met — this is the role of **policy guards**.

Today, enforcement is either absent or ad-hoc:

| Scenario | Current Behavior | Desired Behavior |
|----------|-----------------|------------------|
| Commit with dirty worktree | Allowed (no check) | Block or warn depending on transition |
| Start execution with 5 tasks already in-progress | Allowed | Block: WIP limit exceeded |
| Branch name doesn't follow convention | No validation | Warn: `feature/...` or `fix/...` expected |
| Deploy without passing tests | Depends on CI (external) | Guard: test suite must pass before verification |

**This ADR** defines a composable policy system where individual policies implement a shared trait, can be combined (AND/OR), and are configured via `mcb.toml`. Policies receive a `ProjectContext` (ADR-035) and a `TransitionTrigger` (ADR-034), returning a `PolicyResult` with typed violations.

### Requirements

-   Individual policies implement a common trait
-   Policies composable via AND/OR combinators
-   Configurable per-project via `mcb.toml` (enable/disable, thresholds)
-   Two evaluation modes: fail-fast (stop on first error) and collect-all (gather all violations)
-   Severity levels: Error (blocks transition), Warning (logged but allowed), Info (informational)
-   Extensible: new policies can be added without modifying existing code

## Decision

### 1. Policy Trait Design

```rust
// mcb-domain/src/ports/providers/policy_guard.rs

use crate::entities::context::ProjectContext;
use crate::entities::policy::{PolicyConfig, PolicyResult};
use crate::entities::workflow::TransitionTrigger;
use crate::errors::WorkflowError;

/// Port for policy evaluation.
///
/// Evaluates all active policies against a transition context.
/// Consumed by WorkflowService (ADR-037) before executing FSM transitions.
#[async_trait::async_trait]
pub trait PolicyGuardProvider: Send + Sync {
    /// Evaluate all active policies for a transition.
    /// Returns a merged PolicyResult with all violations.
    async fn evaluate(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError>;

    /// List all registered policies with their current configuration.
    async fn list_policies(&self) -> Result<Vec<PolicyConfig>, WorkflowError>;

    /// Check if a specific policy would pass (without blocking the transition).
    async fn dry_run(
        &self,
        policy_name: &str,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError>;
}
```

### 2. Individual Policy Trait

```rust
// mcb-domain/src/ports/providers/policy.rs

use crate::entities::context::ProjectContext;
use crate::entities::policy::PolicyResult;
use crate::entities::workflow::TransitionTrigger;
use crate::errors::WorkflowError;

/// Individual policy that evaluates one condition.
///
/// Policies are composed into a PolicyGuardProvider via AND/OR combinators.
/// Each policy has a name, priority, and configurable severity.
#[async_trait::async_trait]
pub trait Policy: Send + Sync {
    /// Unique name for this policy (e.g., "wip_limit", "clean_worktree").
    fn name(&self) -> &str;

    /// Description of what this policy enforces.
    fn description(&self) -> &str;

    /// Evaluation priority (lower = evaluated first). Default: 100.
    fn priority(&self) -> u32 { 100 }

    /// Whether this policy applies to the given trigger.
    /// Returning false skips evaluation (not a violation).
    fn applies_to(&self, trigger: &TransitionTrigger) -> bool;

    /// Evaluate the policy. Returns Ok(PolicyResult) even on violation
    /// (violation is a valid outcome, not an error).
    async fn check(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError>;
}
```

### 3. Domain Entities

```rust
// mcb-domain/src/entities/policy.rs

use serde::{Deserialize, Serialize};

/// Severity of a policy violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Blocks the transition. Must be resolved.
    Error,
    /// Logged but does not block. Advisory.
    Warning,
    /// Informational only.
    Info,
}

/// A single policy violation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// Which policy produced this violation.
    pub policy_name: String,
    /// Human-readable message.
    pub message: String,
    /// Severity level.
    pub severity: Severity,
    /// Optional suggestion for resolution.
    pub suggestion: Option<String>,
}

/// Aggregated result of evaluating one or more policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    /// True if no Error-level violations found.
    pub allowed: bool,
    /// All violations (errors, warnings, and info).
    pub violations: Vec<Violation>,
}

impl PolicyResult {
    /// All policies passed, no violations.
    pub fn pass() -> Self {
        Self { allowed: true, violations: Vec::new() }
    }

    /// Create a failed result with a single violation.
    pub fn deny(policy_name: &str, message: &str) -> Self {
        Self {
            allowed: false,
            violations: vec![Violation {
                policy_name: policy_name.to_string(),
                message: message.to_string(),
                severity: Severity::Error,
                suggestion: None,
            }],
        }
    }

    /// Create a warning (allowed but flagged).
    pub fn warn(policy_name: &str, message: &str) -> Self {
        Self {
            allowed: true,
            violations: vec![Violation {
                policy_name: policy_name.to_string(),
                message: message.to_string(),
                severity: Severity::Warning,
                suggestion: None,
            }],
        }
    }

    /// Merge another result into this one. Allowed = both allowed.
    pub fn merge(&mut self, other: PolicyResult) {
        self.allowed = self.allowed && other.allowed;
        self.violations.extend(other.violations);
    }

    /// True if any Error-level violations exist.
    pub fn has_errors(&self) -> bool {
        self.violations.iter().any(|v| v.severity == Severity::Error)
    }

    /// True if any Warning-level violations exist.
    pub fn has_warnings(&self) -> bool {
        self.violations.iter().any(|v| v.severity == Severity::Warning)
    }

    /// Format violations for display.
    pub fn format_violations(&self) -> String {
        self.violations
            .iter()
            .map(|v| {
                let icon = match v.severity {
                    Severity::Error => "ERROR",
                    Severity::Warning => "WARN",
                    Severity::Info => "INFO",
                };
                let suggestion = v.suggestion.as_deref().unwrap_or("");
                if suggestion.is_empty() {
                    format!("[{icon}] {}: {}", v.policy_name, v.message)
                } else {
                    format!("[{icon}] {}: {} (fix: {suggestion})", v.policy_name, v.message)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Configuration for a single policy (from mcb.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub severity: Severity,
    /// Policy-specific settings (JSON).
    pub settings: serde_json::Value,
}
```

### 4. Policy Composition

```rust
// mcb-providers/src/guard/composition.rs

use mcb_domain::ports::providers::policy::Policy;
use mcb_domain::entities::policy::PolicyResult;

/// AND combinator: all policies must pass.
pub struct AllPolicies {
    policies: Vec<Box<dyn Policy>>,
    fail_fast: bool,
}

impl AllPolicies {
    pub fn new(policies: Vec<Box<dyn Policy>>, fail_fast: bool) -> Self {
        let mut sorted = policies;
        sorted.sort_by_key(|p| p.priority());
        Self { policies: sorted, fail_fast }
    }
}

#[async_trait::async_trait]
impl Policy for AllPolicies {
    fn name(&self) -> &str { "all" }
    fn description(&self) -> &str { "All sub-policies must pass" }

    fn applies_to(&self, trigger: &TransitionTrigger) -> bool {
        self.policies.iter().any(|p| p.applies_to(trigger))
    }

    async fn check(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError> {
        let mut result = PolicyResult::pass();

        for policy in &self.policies {
            if !policy.applies_to(trigger) {
                continue;
            }

            let sub_result = policy.check(trigger, context).await?;
            result.merge(sub_result);

            if self.fail_fast && result.has_errors() {
                return Ok(result);
            }
        }

        Ok(result)
    }
}

/// OR combinator: at least one policy must pass.
pub struct AnyPolicy {
    policies: Vec<Box<dyn Policy>>,
}

#[async_trait::async_trait]
impl Policy for AnyPolicy {
    fn name(&self) -> &str { "any" }
    fn description(&self) -> &str { "At least one sub-policy must pass" }

    fn applies_to(&self, trigger: &TransitionTrigger) -> bool {
        self.policies.iter().any(|p| p.applies_to(trigger))
    }

    async fn check(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError> {
        let mut all_violations = Vec::new();

        for policy in &self.policies {
            if !policy.applies_to(trigger) {
                continue;
            }

            let sub_result = policy.check(trigger, context).await?;
            if sub_result.allowed {
                return Ok(PolicyResult::pass());
            }
            all_violations.extend(sub_result.violations);
        }

        Ok(PolicyResult {
            allowed: false,
            violations: all_violations,
        })
    }
}
```

### 5. Policy Lifecycle (All 5 Trigger Points)

Policies execute at **five distinct points** in the workflow lifecycle. Understanding these points is essential for determining when a policy applies and what context is available.

#### 5.1 Compile-Time (Static Analysis)

**When**: During project build/compilation.

**Trigger**: Implicit — triggered by `cargo build` or CI pipeline.

**Context Available**:

-   Source code on disk
-   AST (from Rust compiler)
-   Type information (Rust compiler)

**Policies Applicable**:

-   **Format Validation** — Check Rust code formatting (`rustfmt`)
-   **Syntax Checking** — Ensure code compiles (`cargo check`)
-   **Type Safety** — Rust compiler enforces type checking automatically

**Severity Mapping**:

-   Format violations: WARN (style suggestion)
-   Syntax errors: ERROR (blocks compilation)
-   Type errors: ERROR (compiler enforces)

**Example**:

```rust
// Compile-time policies run in the build pipeline
// cargo build -> cargo check -> rustfmt check -> proceed or fail
if !code_compiles {
    return Err(PolicyViolation { 
        policy: "syntax_check", 
        message: "Code does not compile", 
        severity: ERROR 
    });
}
```

---

#### 5.2 Pre-Commit (Local Checks)

**When**: Before `git commit` is created locally.

**Trigger**: Git pre-commit hook installed by mcb.

**Context Available**:

-   Staged files (git index)
-   Unstaged changes
-   Untracked files
-   Working tree state

**Policies Applicable**:

-   **Code Style** — Run `rustfmt` and `cargo clippy` on staged files
-   **Trailing Whitespace** — Detect and block commits with trailing spaces
-   **File Size Limits** — Reject large binary files (e.g., > 10MB)
-   **Commit Message Format** — Validate conventional commit format
-   **Branch Naming** — Ensure branch follows naming convention

**Severity Mapping**:

-   Style violations: WARN (can auto-fix with `cargo fmt`)
-   Large files: ERROR (block commit)
-   Bad commit message: ERROR (block commit)
-   Bad branch name: WARN or ERROR (configurable)

**Example**:

```bash
# Git pre-commit hook pseudo-code
git diff --staged | check_formatting()  # WARN if rustfmt violations
git diff --staged | check_whitespace()  # ERROR if trailing spaces
ls $(git diff --cached --name-only) | check_file_size()  # ERROR if > limit
validate_commit_message()  # ERROR if not conventional commit
```

---

#### 5.3 Pre-Transition (FSM Guard)

**When**: After a user invokes a workflow command but **before** the FSM state changes.

**Trigger**: WorkflowService invokes `PolicyGuardProvider::evaluate()` before executing transition logic.

**Context Available**:

-   Full `ProjectContext` (from ADR-035) including:
    -   Git state (branch, worktree status, commits)
    -   Issue tracker state (open/in-progress/closed counts)
    -   Task WIP counts
    -   Environment details (project root, config)
-   `TransitionTrigger` (which command triggered this)

**Policies Applicable**:

-   **WIP Limit** — Prevent starting new tasks if too many in-progress
-   **Clean Worktree** — Block transition if git is dirty
-   **Test Results** — Require tests to pass before starting verification
-   **Branch Protection** — Disallow transitions on protected branches (e.g., main)
-   **Orchestrator Checks** — Verify preconditions before any transition

**Severity Mapping**:

-   WIP exceeded: ERROR (blocks transition)
-   Dirty worktree: ERROR (blocks verification)
-   Tests failing: ERROR (blocks verification)
-   Branch mismatch: ERROR (blocks certain transitions)

**Example**:

```rust
// Pre-transition evaluation in WorkflowService
#[async_trait::async_trait]
impl WorkflowService {
    pub async fn start_verification(&self, claim: &str) -> Result<(), WorkflowError> {
        let context = self.context_scout.gather().await?;
        
        // BEFORE state transition, evaluate policies
        let policy_result = self.policy_guard.evaluate(
            &TransitionTrigger::StartVerification { claim },
            &context
        ).await?;
        
        if !policy_result.allowed {
            return Err(WorkflowError::PolicyViolation {
                violations: policy_result.violations,
            });
        }
        
        // Now safe to proceed with FSM transition
        self.fsm.transition(State::Verifying).await?;
        Ok(())
    }
}
```

---

#### 5.4 CI-Time (Continuous Integration)

**When**: During GitHub Actions / CI pipeline execution on a pull request or push.

**Trigger**: `.github/workflows/ci.yml` executes tests and checks.

**Context Available**:

-   Source code in CI environment
-   Test results (pass/fail, coverage %)
-   Security scan results (`cargo audit`, `cargo deny`)
-   Code coverage metrics (`cargo tarpaulin`)
-   Commit metadata (author, message, diff)

**Policies Applicable**:

-   **Require Tests** — All tests pass (no failing tests)
-   **Code Coverage** — Changed code covered by ≥70% of tests
-   **Security Scan** — No critical CVEs reported by `cargo audit`
-   **License Compliance** — `cargo deny` passes (approved licenses only)
-   **Dependency Check** — No yanked or banned dependencies

**Severity Mapping**:

-   Test failure: ERROR (blocks merge)
-   Coverage below threshold: ERROR (blocks merge)
-   Critical CVE: ERROR (blocks merge)
-   License violation: ERROR (blocks merge)
-   Deprecation warning: WARN (logged but allowed)

**Example**:

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      # Policy: Require Tests
      - name: Run tests
        run: cargo test --release
        # Blocks merge if any test fails (ERROR)
      
      # Policy: Code Coverage
      - name: Coverage check
        run: cargo tarpaulin --out Xml --fail-under 70
        # Blocks merge if coverage < 70% (ERROR)
      
      # Policy: Security Scan
      - name: Audit dependencies
        run: cargo audit --deny warnings
        # Blocks merge on critical CVE (ERROR)
      
      # Policy: License Compliance
      - name: Check licenses
        run: cargo deny check
        # Blocks merge on unapproved license (ERROR)
```

---

#### 5.5 Post-Merge (Verification)

**When**: After a PR is merged to the main branch.

**Trigger**: GitHub Actions hook on `push` to `main` branch.

**Context Available**:

-   Merged commit on main
-   Release notes or changelog
-   Updated documentation
-   Previous release version

**Policies Applicable**:

-   **Smoke Tests on Main** — Run a minimal test suite to ensure main is healthy
-   **Documentation Updated** — Verify CHANGELOG.md or ARCHITECTURE.md were updated if code changed
-   **Version Bumped** — Check that Cargo.toml version was incremented (semantic versioning)
-   **Release Artifacts Generated** — Ensure release binary built successfully

**Severity Mapping**:

-   Smoke test failure: ERROR (requires hotfix)
-   Docs not updated: WARN (logged, create follow-up issue)
-   Version not bumped: WARN (logged, fix in next release)
-   Build failure: ERROR (requires immediate fix)

**Example**:

```yaml
# .github/workflows/post-merge.yml
name: Post-Merge Verification

on:
  push:
    branches:
      - main

jobs:
  smoke-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      # Policy: Smoke Tests on Main
      - name: Run smoke tests
        run: cargo test --release --test '*'
        # Failure: ERROR (requires hotfix to main)
      
      # Policy: Documentation Updated
      - name: Verify docs updated
        run: |
          if git diff HEAD~1 --name-only | grep -q "\.rs$"; then
            if ! git diff HEAD~1 --name-only | grep -qE "(CHANGELOG|ARCHITECTURE|README)"; then
              echo "WARN: Code changed but docs not updated"
            fi
          fi
        # Violation: WARN (advisory)
      
      # Policy: Version Bump
      - name: Check version increment
        run: |
          PREV_VERSION=$(git show HEAD~1:Cargo.toml | grep "^version" | head -1)
          CURR_VERSION=$(grep "^version" Cargo.toml | head -1)
          if [ "$PREV_VERSION" == "$CURR_VERSION" ]; then
            echo "WARN: Version not bumped"
          fi
        # Violation: WARN (advisory)
```

---

### 6. Built-In Policies (11 Total)

Each policy is fully defined with name, trigger point(s), description, severity level, configuration, and remediation guidance.

#### 6.1 WIP Limit

**Policy Name**: `wip_limit`

**Trigger Points**: Pre-Transition (FSM Guard)

**Description**: Prevents operators from starting new tasks when too many are already in-progress. Enforces work-in-progress limits to reduce context-switching and bottlenecks.

**Severity**: ERROR (blocks transition)

**Configuration**:

```toml
[policies.wip_limit]
enabled = true
severity = "error"
max_in_progress = 3
```

**Validation Logic**:

```rust
if context.tracker.in_progress.len() >= config.max_in_progress {
    return Err(PolicyViolation {
        policy: "wip_limit",
        message: format!("WIP limit exceeded: {} in-progress (max: {})",
            context.tracker.in_progress.len(),
            config.max_in_progress),
        severity: ERROR,
        suggestion: "Complete or close existing in-progress issues before starting new work"
    });
}
```

**Remediation**:

```bash
# To resolve WIP limit violation:
bd list --status=in_progress  # See what's in-progress
bd close <id>                  # Close completed tasks
bd update <id> --status=completed  # Mark as complete
# Then retry the transition
```

---

#### 6.2 Clean Worktree

**Policy Name**: `clean_worktree`

**Trigger Points**: Pre-Transition (FSM Guard) — applies to `StartVerification`, `CompletePhase`, `EndSession`

**Description**: Ensures no uncommitted changes exist before important transitions. Prevents lost work and ensures reproducible builds.

**Severity**: ERROR (blocks transition)

**Configuration**:

```toml
[policies.clean_worktree]
enabled = true
severity = "error"
allow_untracked = true  # Ignore untracked files (only check staged/unstaged)
```

**Validation Logic**:

```rust
let dirty_count = git.staged_files + git.unstaged_files + git.conflicted_files
    + if config.allow_untracked { 0 } else { git.untracked_files };

if dirty_count > 0 {
    let details = format!("{} staged, {} unstaged, {} conflicted",
        git.staged_files, git.unstaged_files, git.conflicted_files);
    return Err(PolicyViolation {
        policy: "clean_worktree",
        message: format!("Worktree not clean: {}", details),
        severity: ERROR,
        suggestion: "Commit or stash changes before this transition"
    });
}
```

**Remediation**:

```bash
# To resolve dirty worktree:
git status                    # See what's dirty
git add .                     # Stage changes
git commit -m "work in progress"  # Commit with message
# Or stash if unsure:
git stash
# Then retry the transition
```

---

#### 6.3 Branch Naming

**Policy Name**: `branch_naming`

**Trigger Points**: Pre-Transition (FSM Guard) — applies to context discovery

**Description**: Validates that branch names follow the project convention. Enforces consistency and makes automation easier.

**Severity**: ERROR (blocks transition) or WARN (advisory, configurable)

**Configuration**:

```toml
[policies.branch_naming]
enabled = true
severity = "error"
# Regex pattern: feature/*, fix/*, release/*, docs/*, etc.
pattern = "^(feature|fix|release|docs|chore|test|refactor)/[a-z0-9-]+$"
expected_format = "feature/foo-bar, fix/bug-123, release/1.2.3"
```

**Validation Logic**:

```rust
let regex = Regex::new(&config.pattern)?;
if !regex.is_match(&context.git.branch) {
    return Err(PolicyViolation {
        policy: "branch_naming",
        message: format!("Branch '{}' doesn't match convention", context.git.branch),
        severity: ERROR,
        suggestion: format!("Expected format: {}", config.expected_format)
    });
}
```

**Remediation**:

```bash
# To resolve branch naming violation:
# Option 1: Create new branch with correct name
git checkout -b feature/correct-name
git cherry-pick <commit-hash>  # Copy work
git push -u origin feature:correct-name

# Option 2: Rename existing branch
git branch -m feature/correct-name
git push -u origin feature:correct-name

# Then retry the transition
```

---

#### 6.4 Require Tests

**Policy Name**: `require_tests`

**Trigger Points**: CI-Time (GitHub Actions), Pre-Transition (StartVerification)

**Description**: Ensures test suite passes before verification begins. Prevents shipping untested code.

**Severity**: ERROR (blocks merge/verification)

**Configuration**:

```toml
[policies.require_tests]
enabled = true
severity = "error"
test_command = "cargo test --release"
timeout_seconds = 300
```

**Validation Logic**:

```rust
let output = Command::new("sh")
    .arg("-c")
    .arg(&config.test_command)
    .current_dir(&context.project_root)
    .timeout(Duration::from_secs(config.timeout_seconds))
    .output()
    .await?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(PolicyViolation {
        policy: "require_tests",
        message: format!("Test suite failed (exit code: {:?})", output.status.code()),
        severity: ERROR,
        suggestion: format!("Fix failing tests. Error output:\n{}", &stderr[..200.min(stderr.len())])
    });
}
```

**Remediation**:

```bash
# To resolve test failure:
cargo test --release          # Run tests locally to see failures
# Fix code / tests as needed
cargo test --release          # Verify tests pass
git add .
git commit -m "fix: tests passing"
# Then retry the transition
```

---

#### 6.5 Changelog Check

**Policy Name**: `changelog_check`

**Trigger Points**: Pre-Merge (GitHub Actions check)

**Description**: Verifies that CHANGELOG.md was updated when code changes. Ensures release notes stay current.

**Severity**: WARN (allowed but logged as advisory)

**Configuration**:

```toml
[policies.changelog_check]
enabled = true
severity = "warning"
changelog_file = "CHANGELOG.md"
```

**Validation Logic**:

```rust
// Check if code files changed in commit/PR
let code_changed = diff.files
    .iter()
    .any(|f| f.ends_with(".rs") || f.ends_with(".toml"));

// Check if CHANGELOG was updated
let changelog_changed = diff.files
    .iter()
    .any(|f| f.ends_with("CHANGELOG.md"));

if code_changed && !changelog_changed {
    return Ok(PolicyViolation {
        policy: "changelog_check",
        message: "Code changed but CHANGELOG.md not updated",
        severity: WARN,  // Advisory only
        suggestion: "Update CHANGELOG.md with entry for this change"
    });
}
```

**Remediation**:

```bash
# To resolve changelog warning:
# Edit CHANGELOG.md and add entry at top
vim CHANGELOG.md
# Follow "Unreleased" section format:
# ## [Unreleased]
# ### Added
# - New feature description
# ### Fixed
# - Bug fix description

git add CHANGELOG.md
git commit --amend  # or new commit if allowed
# Violation is advisory (WARN), so doesn't block merge
```

---

#### 6.6 Commit Message Format

**Policy Name**: `commit_message_format`

**Trigger Points**: Pre-Commit (Git hook)

**Description**: Validates commit messages follow conventional commit format. Enables automated changelog generation and semantic versioning.

**Severity**: ERROR (blocks commit)

**Configuration**:

```toml
[policies.commit_message_format]
enabled = true
severity = "error"
format = "conventional"  # Options: "conventional", "custom"
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "perf"]
require_scope = false    # If true, enforce type(scope): message
example = "feat(core): add policy system"
```

**Validation Logic**:

```rust
// Conventional commit: type(scope)?: message
// Example: "feat(guards): add WIP limit policy"
let pattern = r"^(feat|fix|docs|style|refactor|test|chore|perf)(\([a-z-]+\))?: .+";
let regex = Regex::new(pattern)?;

if !regex.is_match(&commit_message) {
    return Err(PolicyViolation {
        policy: "commit_message_format",
        message: "Commit message doesn't follow conventional commit format",
        severity: ERROR,
        suggestion: format!("Expected format: {}", config.example)
    });
}
```

**Remediation**:

```bash
# To resolve commit message format violation:
git commit --amend -m "feat(guards): add WIP limit policy"
# Follow format: type(scope): description
# Types: feat, fix, docs, style, refactor, test, chore, perf
# Scope: optional, brief module name (guards, core, config, etc.)
# Description: imperative mood, lowercase first letter
```

---

#### 6.7 Code Review Gate

**Policy Name**: `code_review_gate`

**Trigger Points**: Pre-Merge (GitHub Actions check before merging PR)

**Description**: Requires PR approval by one or more reviewers before merging. Enforces peer review discipline.

**Severity**: ERROR (blocks merge)

**Configuration**:

```toml
[policies.code_review_gate]
enabled = true
severity = "error"
min_approvals = 1         # Minimum approvals required
require_dismissal = true  # Must dismiss old reviews if code changed
allowed_reviewers = ["*"] # Can specify list of allowed reviewers or ["*"] for any
```

**Validation Logic**:

```rust
let approved_by = pr.reviews
    .iter()
    .filter(|r| r.status == "approved" && r.state == "submitted")
    .count();

if approved_by < config.min_approvals {
    return Err(PolicyViolation {
        policy: "code_review_gate",
        message: format!("PR has {} approval(s), requires {}",
            approved_by, config.min_approvals),
        severity: ERROR,
        suggestion: format!("Request review from {} reviewer(s)",
            config.min_approvals - approved_by)
    });
}

// Check if changes requested
if pr.reviews.iter().any(|r| r.state == "changes_requested") {
    return Err(PolicyViolation {
        policy: "code_review_gate",
        message: "Changes requested by reviewer",
        severity: ERROR,
        suggestion: "Address reviewer feedback and request re-review"
    });
}
```

**Remediation**:

```bash
# To resolve code review gate violation:
# 1. Ensure PR is created and pushed
git push -u origin feature/my-feature

# 2. Request review from a colleague on GitHub
# (Navigate to PR, request reviewer)

# 3. Address feedback from review
# Make requested changes:
git add .
git commit -m "fix: address review feedback"
git push

# 4. Reviewer approves (GitHub UI)
# Then merge is allowed
```

---

#### 6.8 Code Coverage

**Policy Name**: `code_coverage`

**Trigger Points**: CI-Time (GitHub Actions, blocks merge)

**Description**: Ensures new/changed code is covered by tests above a threshold (e.g., 70%). Maintains code quality and reduces regressions.

**Severity**: ERROR (blocks merge)

**Configuration**:

```toml
[policies.code_coverage]
enabled = true
severity = "error"
threshold = 70           # Minimum coverage % required
tool = "cargo-tarpaulin" # Options: "cargo-tarpaulin", "cargo-llvm-cov"
scope = "changed-lines"  # Coverage of changed lines only, not entire codebase
```

**Validation Logic**:

```rust
// Run coverage tool
let output = Command::new("cargo")
    .args(&["tarpaulin", "--out", "Xml", "--exclude-files", "tests/*"])
    .output()
    .await?;

// Parse coverage XML
let coverage_pct = parse_coverage_xml(&output.stdout)?;

if coverage_pct < config.threshold {
    return Err(PolicyViolation {
        policy: "code_coverage",
        message: format!("Code coverage {}% is below threshold {}%",
            coverage_pct, config.threshold),
        severity: ERROR,
        suggestion: "Add tests to reach coverage threshold"
    });
}
```

**Remediation**:

```bash
# To resolve code coverage violation:
cargo tarpaulin --out Html  # Generate coverage report
# Open tarpaulin-report.html to see uncovered lines
# Add tests for uncovered code:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_violation_case() {
        // Test the uncovered code path
    }
}
cargo test                  # Verify tests pass
cargo tarpaulin --out Xml   # Re-run coverage
# Repeat until threshold reached
```

---

#### 6.9 Security Scan

**Policy Name**: `security_scan`

**Trigger Points**: CI-Time (GitHub Actions, blocks merge)

**Description**: Runs `cargo audit` and `cargo deny` to detect CVEs, yanked dependencies, and unapproved licenses. Prevents shipping vulnerable code.

**Severity**: ERROR (blocks merge)

**Configuration**:

```toml
[policies.security_scan]
enabled = true
severity = "error"
audit_enabled = true      # Run cargo audit
deny_enabled = true       # Run cargo deny
fail_on = "warnings"      # "warnings" blocks on advisory, "denies" only on hard denies
```

**Validation Logic**:

```rust
// Run cargo audit
let audit_output = Command::new("cargo")
    .args(&["audit", "--deny", config.fail_on])
    .output()
    .await?;

if !audit_output.status.success() {
    let stderr = String::from_utf8_lossy(&audit_output.stderr);
    return Err(PolicyViolation {
        policy: "security_scan",
        message: format!("Cargo audit failed: vulnerabilities or banned deps found"),
        severity: ERROR,
        suggestion: "Run `cargo audit` locally to see details and update dependencies"
    });
}

// Run cargo deny
let deny_output = Command::new("cargo")
    .args(&["deny", "check"])
    .output()
    .await?;

if !deny_output.status.success() {
    return Err(PolicyViolation {
        policy: "security_scan",
        message: "Cargo deny failed: unapproved license or other issue",
        severity: ERROR,
        suggestion: "Update deny.toml or dependencies to comply with policy"
    });
}
```

**Remediation**:

```bash
# To resolve security scan violation:
cargo audit              # See vulnerabilities
cargo update             # Update to patched versions
# Or if no patch available:
cargo audit --deny warnings --ignore <advisory-id>

cargo deny check          # See license/policy issues
# Edit deny.toml to allow approved licenses
vim Cargo.deny

cargo audit              # Verify resolved
cargo deny check
# Then retry merge
```

---

#### 6.10 Version Bump

**Policy Name**: `version_bump`

**Trigger Points**: Post-Merge (verification on main)

**Description**: Validates that semantic versioning was followed in version bumps. Tracks version evolution and enables automated release notes.

**Severity**: WARN (advisory, logged but doesn't block)

**Configuration**:

```toml
[policies.version_bump]
enabled = true
severity = "warning"
format = "semantic"  # Options: "semantic" (x.y.z), "calver" (YYYY.MM.DD)
file = "Cargo.toml"  # Location of version
```

**Validation Logic**:

```rust
// Extract version from current commit
let current_version = parse_cargo_toml("Cargo.toml")?;
let prev_version = parse_cargo_toml_at_commit("HEAD~1:Cargo.toml")?;

if current_version == prev_version {
    return Ok(PolicyViolation {
        policy: "version_bump",
        message: format!("Version not bumped (still {})", current_version),
        severity: WARN,  // Advisory only
        suggestion: "Update version in Cargo.toml using semantic versioning"
    });
}

// Validate semantic versioning (x.y.z)
if !Regex::new(r"^\d+\.\d+\.\d+$")?.is_match(&current_version) {
    return Ok(PolicyViolation {
        policy: "version_bump",
        message: format!("Version '{}' doesn't follow semantic versioning", current_version),
        severity: WARN,
        suggestion: "Use format: major.minor.patch (e.g., 1.2.3)"
    });
}
```

**Remediation**:

```bash
# To resolve version bump warning:
# In Cargo.toml, update version following semantic versioning:
# - Patch (0.1.1 -> 0.1.2) for bug fixes
# - Minor (0.1.0 -> 0.2.0) for new features
# - Major (1.0.0 -> 2.0.0) for breaking changes

vim Cargo.toml
# Change: version = "0.1.0"
#    To:  version = "0.2.0"

git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
# WARN is advisory, so doesn't block merge
# But fix it for consistent release management
```

---

#### 6.11 Documentation Update

**Policy Name**: `documentation_update`

**Trigger Points**: Post-Merge (verification after merge to main)

**Description**: Ensures project documentation (README.md, ARCHITECTURE.md, docs/) is updated when code changes. Keeps documentation in sync with implementation.

**Severity**: WARN (advisory, logged but doesn't block)

**Configuration**:

```toml
[policies.documentation_update]
enabled = true
severity = "warning"
required_docs = ["README.md", "ARCHITECTURE.md"]  # Files that should be updated if code changes
code_patterns = ["src/**/*.rs", "Cargo.toml"]     # Patterns that trigger doc requirement
```

**Validation Logic**:

```rust
// Check if code files changed
let code_changed = diff.files
    .iter()
    .any(|f| {
        config.code_patterns.iter().any(|pattern| {
            glob_match(pattern, f)
        })
    });

if !code_changed {
    return Ok(PolicyResult::pass());  // No code change, skip check
}

// Check if docs were updated
let docs_changed = config.required_docs
    .iter()
    .any(|doc| diff.files.iter().any(|f| f.ends_with(doc)));

if code_changed && !docs_changed {
    return Ok(PolicyViolation {
        policy: "documentation_update",
        message: "Code changed but required documentation not updated",
        severity: WARN,  // Advisory only
        suggestion: "Update README.md or ARCHITECTURE.md to reflect code changes"
    });
}
```

**Remediation**:

```bash
# To resolve documentation update warning:
# 1. Create follow-up PR or issue for documentation
bd create --title="Docs: Update README for feature X" --type=task --priority=2

# 2. Or immediately update docs:
vim README.md          # Add feature to features list
vim ARCHITECTURE.md    # Update architecture docs if applicable
git add README.md ARCHITECTURE.md
git commit -m "docs: update for new feature X"
git push

# WARN is advisory, so merge can proceed
# But create follow-up issue if docs not updated immediately
```

---

### 7. Deny-Wins Semantics & Conflict Resolution

This section defines how policy violations are evaluated, combined, and enforced.

#### 7.1 Severity Levels

**ERROR (Hard Deny)**

-   **Definition**: Policy violation blocks the transition.
-   **Behavior**: If ANY ERROR-level violation exists, the entire transition is rejected.
-   **Operator Action**: Must fix violation or explicitly override (with reason).
-   **Transition**: Blocked until resolved.

**WARNING (Soft Advisory)**

-   **Definition**: Policy violation is logged but does not block the transition.
-   **Behavior**: Operator is warned; transition proceeds.
-   **Operator Action**: Recommended to fix, but not required.
-   **Transition**: Allowed to proceed.

**INFO (Informational)**

-   **Definition**: Policy observation logged for informational purposes only.
-   **Behavior**: No enforcement; purely advisory.
-   **Operator Action**: Optional reading/acting.
-   **Transition**: Allowed to proceed.

#### 7.2 Deny-Wins Logic (Fail-Closed)

**Core Principle**: "Deny wins" — the system defaults to DENY unless ALL policies permit the transition.

**Evaluation Algorithm**:

```rust
pub fn evaluate_policies(policies: Vec<PolicyResult>) -> PolicyResult {
    let mut combined = PolicyResult::pass();
    
    for policy_result in policies {
        // If ANY error found, combined becomes denied
        if !policy_result.allowed && policy_result.has_errors() {
            combined.allowed = false;
        }
        
        // Merge all violations (for reporting)
        combined.violations.extend(policy_result.violations);
    }
    
    combined
}

// In transition logic:
if !combined.allowed {
    // Block transition (ERROR exists)
    return Err(WorkflowError::PolicyViolation {
        violations: combined.violations,
        remediation: format_remediation(&combined.violations),
    });
}

// If only WARNINGs, proceed but log them
if combined.has_warnings() {
    warn!("Policy warnings (non-blocking): {}", 
        combined.format_violations());
}

// All ERROR checks passed, proceed with transition
return Ok(());
```

#### 7.3 Operator Override

**When ERROR violations exist**, operators can override with an explicit reason:

```rust
// mcb-domain/src/ports/providers/policy_guard.rs

#[async_trait::async_trait]
pub trait PolicyGuardProvider: Send + Sync {
    /// Evaluate policies and optionally allow override for ERROR violations.
    async fn evaluate_with_override(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
        override_reason: Option<&str>,
    ) -> Result<PolicyResult, WorkflowError>;
}
```

**Override Usage**:

```rust
// If policy evaluation fails:
let result = policy_guard.evaluate(trigger, context).await?;

if !result.allowed {
    // Option 1: Fix the violation (recommended)
    // ... fix code, commit, try again ...
    
    // Option 2: Override (operator must provide reason)
    let override_reason = "Emergency: hotfix for production issue";
    let result = policy_guard.evaluate_with_override(
        trigger,
        context,
        Some(override_reason),
    ).await?;
    
    // Override logged to audit trail
    audit_log::log_policy_override(
        operator,
        trigger,
        result.violations,
        override_reason,
    );
}
```

**Audit Trail**: All overrides are logged with:

-   Operator ID
-   Timestamp
-   Override reason
-   Policies bypassed
-   Violation details

#### 7.4 Policy Composition (AND/OR)

**AND Combinator (AllPolicies)**:

-   **Logic**: ALL policies must pass (or be WARN-only).
-   **Short-Circuit**: Stops on first ERROR (fail-fast mode).
-   **Use Case**: Default composition for most transitions.

```rust
pub struct AllPolicies {
    policies: Vec<Box<dyn Policy>>,
    fail_fast: bool,  // If true, stop on first error
}

impl AllPolicies {
    pub async fn check(&self) -> PolicyResult {
        let mut result = PolicyResult::pass();
        
        for policy in &self.policies {
            let sub_result = policy.check(trigger, context).await?;
            result.merge(sub_result);
            
            // Short-circuit on ERROR if fail_fast enabled
            if self.fail_fast && result.has_errors() {
                return Ok(result);
            }
        }
        
        Ok(result)
    }
}
```

**OR Combinator (AnyPolicy)**:

-   **Logic**: AT LEAST ONE policy must pass.
-   **No Short-Circuit**: Evaluates all policies to collect violations.
-   **Use Case**: Rare; example: "approve via email OR Slack message".

```rust
pub struct AnyPolicy {
    policies: Vec<Box<dyn Policy>>,
}

impl AnyPolicy {
    pub async fn check(&self) -> PolicyResult {
        let mut all_violations = Vec::new();
        
        for policy in &self.policies {
            let sub_result = policy.check(trigger, context).await?;
            
            // If any policy passes, we're done
            if sub_result.allowed && !sub_result.has_errors() {
                return Ok(PolicyResult::pass());
            }
            
            all_violations.extend(sub_result.violations);
        }
        
        // None passed
        Ok(PolicyResult {
            allowed: false,
            violations: all_violations,
        })
    }
}
```

#### 7.5 Conflict Resolution

**Scenario 1: Multiple ERROR policies violated**

Policy A (Clean Worktree): ERROR — "Worktree dirty"
Policy B (Require Tests): ERROR — "Tests failed"

**Resolution**:

-   Both errors reported to operator.
-   Transition blocked.
-   Operator must fix both issues.
-   Or override with reason (audit logged).

```rust
let policy_result = guard.evaluate(trigger, context).await?;

if !policy_result.allowed {
    eprintln!("Multiple policy violations:");
    for violation in &policy_result.violations {
        if violation.severity == Severity::Error {
            eprintln!("  ERROR [{}]: {}", violation.policy_name, violation.message);
            if let Some(suggestion) = &violation.suggestion {
                eprintln!("    → {}", suggestion);
            }
        }
    }
    // Block transition
    return Err(WorkflowError::PolicyViolation { violations: ... });
}
```

**Scenario 2: ERROR and WARNING mix**

Policy A (Clean Worktree): ERROR — "Worktree dirty"
Policy B (Changelog Check): WARNING — "Changelog not updated"

**Resolution**:

-   ERROR blocks transition.
-   WARNING noted but not considered for blocking.
-   Fix ERROR first, then proceed (WARNING allowed through).

```rust
if policy_result.has_errors() {
    // Block on errors
    return Err(WorkflowError::PolicyViolation { ... });
}

if policy_result.has_warnings() {
    // Log warnings but proceed
    warn!("Policy warnings: {}", policy_result.format_violations());
}

// Transition allowed
```

**Scenario 3: Conflicting policies (policy A vs policy B)**

Policy A requires: Branch must start with "feature/"
Policy B requires: Branch must start with "release/"

**Resolution**:

-   Configuration should prevent such conflicts.
-   During design: only one applies (branch naming conflicts are scope issue).
-   If unavoidable: operator chooses which policy to override.

```toml
# mcb.toml - ensure non-conflicting policies
[policies.branch_naming]
pattern = "^(feature|fix|release|docs)/[a-z0-9-]+$"
# This pattern allows all three prefixes, no conflict
```

#### 7.6 Remediation Guidance

Each policy violation includes a **suggestion** field for remediation:

```rust
pub struct Violation {
    pub policy_name: String,
    pub message: String,
    pub severity: Severity,
    pub suggestion: Option<String>,  // How to fix
}
```

**Remediation Examples**:

| Policy | Violation | Suggestion |
|--------|-----------|-----------|
| clean_worktree | "Worktree not clean" | "Commit or stash changes before this transition" |
| require_tests | "Test suite failed" | "Run `cargo test` locally; fix failing tests" |
| wip_limit | "WIP limit exceeded" | "Complete or close existing in-progress issues" |
| code_coverage | "Coverage 65% < 70%" | "Add tests to reach coverage threshold" |
| security_scan | "Critical CVE found" | "Run `cargo audit` and update vulnerable deps" |
| branch_naming | "Branch doesn't match" | "Expected format: feature/*, fix/*, release/*" |
| code_review_gate | "No approvals" | "Request review from a colleague on GitHub" |
| commit_message_format | "Bad format" | "Use format: type(scope): message" |
| changelog_check | "CHANGELOG not updated" | "Update CHANGELOG.md with entry" |
| version_bump | "Version not bumped" | "Update Cargo.toml using semantic versioning" |
| documentation_update | "Docs not updated" | "Update README.md or ARCHITECTURE.md" |

### 7.7 Policy Evaluation Context

Policies receive **rich context** for evaluation:

```rust
pub struct ProjectContext {
    // Git state
    pub git: GitContext {
        branch: String,
        commit_sha: String,
        staged_files: usize,
        unstaged_files: usize,
        untracked_files: usize,
        conflicted_files: usize,
    },
    
    // Issue tracker state
    pub tracker: TrackerContext {
        open_count: usize,
        in_progress: Vec<Issue>,
        closed_count: usize,
    },
    
    // Project metadata
    pub project_root: PathBuf,
    pub config: ProjectConfig,
}

pub enum TransitionTrigger {
    StartExecution { claim: String },
    StartVerification,
    CompletePhase { phase: Phase },
    EndSession,
    ContextDiscovered { ... },
    // ... others
}
```

This enables **context-aware policies** that adapt to project state, not just static rules

---

### 8. Configuration (Figment)

```toml
# config/default.toml — [policies] section (all 11 policies)

[policies]
# Global enable/disable for all policy evaluation.
enabled = true
# fail_fast = true stops on first error. false collects all violations.
fail_fast = false

# Pre-Transition Policies (FSM Guard)
[policies.wip_limit]
enabled = true
severity = "error"
max_in_progress = 3

[policies.clean_worktree]
enabled = true
severity = "error"
allow_untracked = true

# Pre-Commit Policies (Git Hook)
[policies.branch_naming]
enabled = true
severity = "error"
pattern = "^(feature|fix|release|docs|chore|test|refactor)/[a-z0-9-]+$"
expected_format = "feature/*, fix/*, release/*, docs/*, etc."

[policies.commit_message_format]
enabled = true
severity = "error"
format = "conventional"
types = ["feat", "fix", "docs", "style", "refactor", "test", "chore", "perf"]
require_scope = false
example = "feat(guards): add WIP limit policy"

# CI-Time Policies (GitHub Actions)
[policies.require_tests]
enabled = true
severity = "error"
test_command = "cargo test --release"
timeout_seconds = 300

[policies.code_coverage]
enabled = true
severity = "error"
threshold = 70
tool = "cargo-tarpaulin"
scope = "changed-lines"

[policies.security_scan]
enabled = true
severity = "error"
audit_enabled = true
deny_enabled = true
fail_on = "warnings"

# Pre-Merge / Post-Merge Policies
[policies.code_review_gate]
enabled = true
severity = "error"
min_approvals = 1
require_dismissal = true

[policies.changelog_check]
enabled = true
severity = "warning"
changelog_file = "CHANGELOG.md"

[policies.version_bump]
enabled = false  # Optional; enable for strict version control
severity = "warning"
format = "semantic"
file = "Cargo.toml"

[policies.documentation_update]
enabled = false  # Optional; enable to require docs updates
severity = "warning"
required_docs = ["README.md", "ARCHITECTURE.md"]
code_patterns = ["src/**/*.rs", "Cargo.toml"]
```

**Configuration Notes**:

-   **Pre-Transition**: `wip_limit`, `clean_worktree` — applied before FSM transition
-   **Pre-Commit**: `branch_naming`, `commit_message_format` — applied by git pre-commit hook
-   **CI-Time**: `require_tests`, `code_coverage`, `security_scan` — applied during GitHub Actions
-   **Pre-Merge**: `code_review_gate` — applied before merge on GitHub
-   **Post-Merge**: `changelog_check`, `version_bump`, `documentation_update` — applied after merge to main

```rust
// mcb-infrastructure/src/config/policies.rs — Updated for 11 policies

use mcb_domain::entities::policy::Severity;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PoliciesConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub fail_fast: bool,
    pub wip_limit: Option<PolicyEntryConfig<WipLimitSettings>>,
    pub clean_worktree: Option<PolicyEntryConfig<CleanWorktreeSettings>>,
    pub branch_naming: Option<PolicyEntryConfig<BranchNamingSettings>>,
    pub commit_message_format: Option<PolicyEntryConfig<CommitMessageSettings>>,
    pub require_tests: Option<PolicyEntryConfig<RequireTestsSettings>>,
    pub code_coverage: Option<PolicyEntryConfig<CodeCoverageSettings>>,
    pub security_scan: Option<PolicyEntryConfig<SecurityScanSettings>>,
    pub code_review_gate: Option<PolicyEntryConfig<CodeReviewSettings>>,
    pub changelog_check: Option<PolicyEntryConfig<ChangelogSettings>>,
    pub version_bump: Option<PolicyEntryConfig<VersionBumpSettings>>,
    pub documentation_update: Option<PolicyEntryConfig<DocsUpdateSettings>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PolicyEntryConfig<S> {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_error")]
    pub severity: Severity,
    #[serde(flatten)]
    pub settings: S,
}

fn default_true() -> bool { true }
fn default_error() -> Severity { Severity::Error }

// Settings structs for each policy
#[derive(Debug, Clone, Deserialize)]
pub struct WipLimitSettings { pub max_in_progress: u32 }

#[derive(Debug, Clone, Deserialize)]
pub struct CleanWorktreeSettings { pub allow_untracked: bool }

#[derive(Debug, Clone, Deserialize)]
pub struct BranchNamingSettings {
    pub pattern: String,
    pub expected_format: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitMessageSettings {
    pub format: String,           // "conventional" or "custom"
    pub types: Vec<String>,       // ["feat", "fix", "docs", ...]
    pub require_scope: bool,
    pub example: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequireTestsSettings {
    pub test_command: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeCoverageSettings {
    pub threshold: u32,
    pub tool: String,             // "cargo-tarpaulin" or "cargo-llvm-cov"
    pub scope: String,            // "changed-lines" or "all"
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityScanSettings {
    pub audit_enabled: bool,
    pub deny_enabled: bool,
    pub fail_on: String,          // "warnings" or "denies"
}

#[derive(Debug, Clone, Deserialize)]
pub struct CodeReviewSettings {
    pub min_approvals: u32,
    pub require_dismissal: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChangelogSettings {
    pub changelog_file: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionBumpSettings {
    pub format: String,           // "semantic" or "calver"
    pub file: String,             // Usually "Cargo.toml"
}

#[derive(Debug, Clone, Deserialize)]
pub struct DocsUpdateSettings {
    pub required_docs: Vec<String>,
    pub code_patterns: Vec<String>,
}
```

### 9. Guard Provider Implementation

```rust
// mcb-providers/src/guard/provider.rs

use mcb_domain::ports::providers::policy::Policy;
use mcb_domain::ports::providers::policy_guard::PolicyGuardProvider;

pub struct ConfigurablePolicyGuard {
    policies: Vec<Box<dyn Policy>>,
    fail_fast: bool,
    enabled: bool,
}

impl ConfigurablePolicyGuard {
    pub fn from_config(config: &PoliciesConfig) -> Result<Self, WorkflowError> {
        let mut policies: Vec<Box<dyn Policy>> = Vec::new();

        if let Some(ref wip) = config.wip_limit {
            if wip.enabled {
                policies.push(Box::new(WipLimitPolicy {
                    config: WipLimitConfig { max_in_progress: wip.settings.max_in_progress },
                    severity: wip.severity.clone(),
                }));
            }
        }

        if let Some(ref cw) = config.clean_worktree {
            if cw.enabled {
                policies.push(Box::new(CleanWorktreePolicy {
                    config: CleanWorktreeConfig { allow_untracked: cw.settings.allow_untracked },
                    severity: cw.severity.clone(),
                }));
            }
        }

        if let Some(ref bn) = config.branch_naming {
            if bn.enabled {
                let regex = regex::Regex::new(&bn.settings.pattern)
                    .map_err(|e| WorkflowError::ContextError {
                        message: format!("Invalid branch_naming pattern: {e}"),
                    })?;
                policies.push(Box::new(BranchNamingPolicy {
                    config: BranchNamingConfig {
                        pattern: bn.settings.pattern.clone(),
                        expected_format: bn.settings.expected_format.clone(),
                    },
                    regex,
                    severity: bn.severity.clone(),
                }));
            }
        }

        if let Some(ref rt) = config.require_tests {
            if rt.enabled {
                policies.push(Box::new(RequireTestsPolicy {
                    config: RequireTestsConfig {
                        test_command: rt.settings.test_command.clone(),
                        timeout_seconds: rt.settings.timeout_seconds,
                    },
                    severity: rt.severity.clone(),
                }));
            }
        }

        // Sort by priority
        policies.sort_by_key(|p| p.priority());

        Ok(Self {
            policies,
            fail_fast: config.fail_fast,
            enabled: config.enabled,
        })
    }
}

#[async_trait::async_trait]
impl PolicyGuardProvider for ConfigurablePolicyGuard {
    async fn evaluate(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError> {
        if !self.enabled {
            return Ok(PolicyResult::pass());
        }

        let all = AllPolicies::new(
            self.policies.iter().map(|p| /* clone/wrap */ ).collect(),
            self.fail_fast,
        );
        all.check(trigger, context).await
    }

    async fn list_policies(&self) -> Result<Vec<PolicyConfig>, WorkflowError> {
        Ok(self.policies.iter().map(|p| PolicyConfig {
            name: p.name().to_string(),
            description: p.description().to_string(),
            enabled: true,
            severity: Severity::Error,
            settings: serde_json::Value::Null,
        }).collect())
    }

    async fn dry_run(
        &self,
        policy_name: &str,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult, WorkflowError> {
        let policy = self.policies.iter().find(|p| p.name() == policy_name)
            .ok_or_else(|| WorkflowError::ContextError {
                message: format!("Policy not found: {policy_name}"),
            })?;
        policy.check(trigger, context).await
    }
}
```

### 10. Provider Registration (linkme)

```rust
// mcb-application/src/registry/guard.rs

use mcb_domain::ports::providers::policy_guard::PolicyGuardProvider;
use std::sync::Arc;

pub struct GuardProviderEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub factory: fn(&figment::Figment) -> Result<Arc<dyn PolicyGuardProvider>, Box<dyn std::error::Error + Send + Sync>>,
}

#[linkme::distributed_slice]
pub static GUARD_PROVIDERS: [GuardProviderEntry] = [..];
```

```rust
// mcb-providers/src/guard/mod.rs

#[linkme::distributed_slice(GUARD_PROVIDERS)]
static CONFIGURABLE_GUARD: GuardProviderEntry = GuardProviderEntry {
    name: "configurable",
    description: "Policy guard with mcb.toml-based configuration",
    factory: configurable_guard_factory,
};

fn configurable_guard_factory(
    config: &figment::Figment,
) -> Result<Arc<dyn PolicyGuardProvider>, Box<dyn std::error::Error + Send + Sync>> {
    let policies_config: PoliciesConfig = config.extract_inner("policies")?;
    Ok(Arc::new(ConfigurablePolicyGuard::from_config(&policies_config)?))
}
```

### 11. Module Locations

| Crate | Path | Content |
|-------|------|---------|
| `mcb-domain` | `src/entities/policy.rs` | `Severity`, `Violation`, `PolicyResult`, `PolicyConfig` |
| `mcb-domain` | `src/ports/providers/policy_guard.rs` | `PolicyGuardProvider` trait |
| `mcb-domain` | `src/ports/providers/policy.rs` | `Policy` trait (individual policies) |
| `mcb-application` | `src/registry/guard.rs` | `GUARD_PROVIDERS` slice |
| `mcb-providers` | `src/guard/mod.rs` | Module root + linkme registration |
| `mcb-providers` | `src/guard/provider.rs` | `ConfigurablePolicyGuard` (all 11 policies) |
| `mcb-providers` | `src/guard/composition.rs` | `AllPolicies`, `AnyPolicy` combinators |
| `mcb-providers` | `src/guard/policies/wip_limit.rs` | `WipLimitPolicy` (policy #1) |
| `mcb-providers` | `src/guard/policies/clean_worktree.rs` | `CleanWorktreePolicy` (policy #2) |
| `mcb-providers` | `src/guard/policies/branch_naming.rs` | `BranchNamingPolicy` (policy #3) |
| `mcb-providers` | `src/guard/policies/commit_message_format.rs` | `CommitMessageFormatPolicy` (policy #6) |
| `mcb-providers` | `src/guard/policies/require_tests.rs` | `RequireTestsPolicy` (policy #4) |
| `mcb-providers` | `src/guard/policies/code_coverage.rs` | `CodeCoveragePolicy` (policy #8) |
| `mcb-providers` | `src/guard/policies/security_scan.rs` | `SecurityScanPolicy` (policy #9) |
| `mcb-providers` | `src/guard/policies/code_review_gate.rs` | `CodeReviewGatePolicy` (policy #7) |
| `mcb-providers` | `src/guard/policies/changelog_check.rs` | `ChangelogCheckPolicy` (policy #5) |
| `mcb-providers` | `src/guard/policies/version_bump.rs` | `VersionBumpPolicy` (policy #10) |
| `mcb-providers` | `src/guard/policies/documentation_update.rs` | `DocumentationUpdatePolicy` (policy #11) |
| `mcb-infrastructure` | `src/config/policies.rs` | `PoliciesConfig` + 11 policy settings structs |

## Consequences

### Positive

-   **Composable**: Policies combined via AND/OR without modifying each other.
-   **Configurable**: Per-project settings via `mcb.toml`. Enable/disable and adjust thresholds without code changes.
-   **Severity levels**: Errors block, warnings log. Teams choose enforcement strictness.
-   **Extensible**: New policies implement `Policy` trait and register via linkme. No existing code modified.
-   **Dry-run**: Policies can be tested without blocking transitions.
-   **Context-aware**: Policies receive full `ProjectContext` (ADR-035), enabling rich conditions.

### Negative

-   **Runtime evaluation cost**: Each transition evaluates all applicable policies. Mitigated by `applies_to()` filter and fail-fast mode.
-   **Test command execution**: `RequireTestsPolicy` spawns a subprocess (e.g., `cargo test`). This is slow (seconds-to-minutes). Only triggered on `StartVerification`.
-   **Config complexity**: 11 policies with individual settings adds config surface area. Mitigated by sensible defaults and disabled-by-default for non-essential policies (e.g., `version_bump`, `documentation_update`).
-   **No runtime policy addition**: Policies are built at startup from config. Adding a new policy requires restart. Runtime dynamic policies deferred.

## Alternatives Considered

### Alternative 1: Tower-Style Middleware

-   **Description:** Model policies as Tower `Layer`/`Service` middleware wrapping the FSM transition.
-   **Pros:** Established pattern. Rich ecosystem (tower-HTTP, tower-retry).
-   **Cons:** Tower is designed for request/response pipelines, not FSM transitions. Adaptation is awkward. Requires tower dependency.
-   **Rejection reason:** Unnecessary complexity. The `Policy` trait with AND/OR composition is simpler and purpose-built.

### Alternative 2: Database-Driven Policies

-   **Description:** Store policy configurations in SQLite and evaluate dynamically.
-   **Pros:** Runtime reconfiguration without restart. Policy versioning.
-   **Cons:** Adds query overhead per evaluation. Config is already in `mcb.toml` (Figment standard).
-   **Rejection reason:** Over-engineering for 11 built-in policies. File-based config is sufficient and matches ADR-025 convention.

### Alternative 3: Hard-Coded Checks (No Policy Framework)

-   **Description:** Embed checks directly in the WorkflowService transition logic.
-   **Pros:** Simplest implementation. No trait, no composition.
-   **Cons:** Not extensible. Every new check requires modifying WorkflowService. No per-project configuration.
-   **Rejection reason:** Violates open/closed principle. Policy framework pays for itself after the second policy.

## Implementation Notes

### Code Changes

1.  Add `policy.rs` entities to `mcb-domain/src/entities/`
2.  Add `policy_guard.rs` and `policy.rs` ports to `mcb-domain/src/ports/providers/`
3.  Add `GUARD_PROVIDERS` slice to `mcb-application/src/registry/`
4.  Add `guard/` module to `mcb-providers/src/` with provider, composition, and **11 built-in policies**
5.  Add `PoliciesConfig` and 11 settings structs to `mcb-infrastructure/src/config/`
6.  Add `[policies]` section to `config/default.toml` with configurations for all 11 policies

### Testing

-   Unit tests: Each of the 11 policies with pass/fail cases (minimum 2 tests per policy = 22 tests)
-   Unit tests: `PolicyResult::merge()`, `format_violations()`, severity handling
-   Unit tests: `AllPolicies` (fail-fast and collect-all modes), `AnyPolicy` combinator
-   Unit tests: Deny-wins semantics, ERROR vs WARNING enforcement
-   Integration tests: `ConfigurablePolicyGuard` with real config, all 11 policies enabled/disabled
-   Integration tests: Lifecycle points (compile-time, pre-commit, pre-transition, CI-time, post-merge)
-   Estimated: **~80+ tests** (11 policies × 2 + integration + composition + semantics)

### Performance Targets

| Operation | Target |
|-----------|--------|
| `WipLimitPolicy.check()` | < 1ms (reads from `TrackerContext`, no I/O) |
| `CleanWorktreePolicy.check()` | < 1ms (reads from `GitContext`, no I/O) |
| `BranchNamingPolicy.check()` | < 1ms (regex match) |
| `CommitMessageFormatPolicy.check()` | < 1ms (regex match) |
| `CodeCoveragePolicy.check()` | < 1ms (reads from coverage report cache) |
| `SecurityScanPolicy.check()` | < test suite time (subprocess: `cargo audit`, `cargo deny`) |
| `RequireTestsPolicy.check()` | < test suite time (subprocess: runs full test suite) |
| `CodeReviewGatePolicy.check()` | < 100ms (GitHub API call) |
| `ChangelogCheckPolicy.check()` | < 1ms (file read + text check) |
| `VersionBumpPolicy.check()` | < 1ms (file read + version parse) |
| `DocumentationUpdatePolicy.check()` | < 1ms (file pattern matching) |
| **Full `evaluate()` (all fast policies)** | **< 20ms** (11 policies, excluding subprocess) |
| **Full `evaluate()` (with CI checks)** | **< test suite time + 100ms** (depends on test execution) |

### Security

-   `RequireTestsPolicy` executes a shell command from `mcb.toml`. The config file must be trusted (same as any TOML config). No user-supplied input reaches the command.

## References

-   [gatehouse](https://crates.io/crates/gatehouse) — Policy composition patterns (evaluated)
-   [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md) — `TransitionTrigger` consumed by guards
-   [ADR-035: Context Scout](./035-context-scout.md) — `ProjectContext` consumed by guards
-   [ADR-025: Figment Configuration](./025-figment-configuration.md) — Config pattern
-   [ADR-029: Hexagonal Architecture](./029-hexagonal-architecture-dill.md) — DI pattern
