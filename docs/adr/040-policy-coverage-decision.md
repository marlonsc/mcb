# ADR-040: Policy Coverage Decision

**Status:** ACCEPTED ✅  
**Date:** 2026-02-05  
**Decision Maker:** Marlon (Technical Lead)  
**Approval:** VOTED & APPROVED

---

## Decision

**Implement 11+ Complete Policies** for v0.2.0 instead of 4-policy MVP or 7-policy hybrid.

Full policy set:

### Tier 1: Essential (v0.1 baseline)
1. `RequireCleanWorktree` - No uncommitted changes
2. `RequireTests` - All tests pass (`make test`)
3. `WIPLimit` - Max N concurrent in_progress tasks
4. `BranchProtection` - Only specific branches allowed

### Tier 2: Collaboration (v0.2 adds)
5. `RequireChangeLog` - CHANGELOG.md entry added
6. `RequireCommitMessage` - Follows conventional commits
7. `RequireCodeReview` - At least N approvals

### Tier 3: Quality Gates (v0.2 adds)
8. `CodeCoverageThreshold` - >= N% coverage maintained
9. `SecurityScan` - No high-severity vulnerabilities
10. `DocumentationCheck` - Public APIs documented

### Tier 4: Advanced (v0.2 adds)
11. `ArchitectureValidation` - `make validate` passes
12. `PerformanceRegression` - Benchmarks within bounds

---

## Rationale

### Against Alternatives

**MVP (4 policies) — Rejected:**
- Covers only basic gates (clean/tests/wip/branch)
- Missing: changelog, commit msgs, code review
- Insufficient for production CI/CD pipelines
- **Cost**: Users must manually enforce changelog + commit conventions
- **Outcome**: Incomplete feature, requires v0.3 for completeness

**Hybrid (7 policies) — Considered:**
- Adds changelog, commits, code review
- Covers 80% of workflows
- Still missing: coverage, security, documentation
- **Issue**: Arbitrary stopping point (why 7 and not 8?)
- **Better**: Complete the feature now, not in phases

**Complete (11+ policies) — APPROVED:**
- Comprehensive CI/CD gate coverage
- Covers all common production workflows
- Extensible plugin architecture for future custom policies
- Only +2 weeks additional development
- **Benefit**: Ship production-ready from v0.2

### Research Precedent

#### GitHub Actions Checks
- ✅ Status checks (Tests pass)
- ✅ Required reviewers
- ✅ Code owners approval
- ✅ Branch protection rules
- ✅ Require conversation resolution
- ✅ Require signed commits
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- ✅ Require code scanning results

**Observation**: GitHub supports 9+ gate types, we're proposing 11.

#### Conventional Commits (Linux, Angular, Chromium)
- Mandatory commit message format
- Tools validate before transition (commitlint)
- Reject if non-conforming
- **Standard**: All major projects enforce

#### Code Review Gates (Gerrit, Phabricator)
- Minimum N approvals required
- Different reviewers for different paths
- SLA: reviews must happen in X time
- **Standard**: Production security baseline

#### Coverage Requirements (Code Climate, Codecov)
- Reject if coverage decreases
- Configurable threshold (80%, 90%, 95%)
- Report detail: file-by-file, not just percent
- **Standard**: Prevents coverage regressions

---

## Complete Policy Implementations

### 1. RequireCleanWorktree

```rust
pub struct RequireCleanWorktree;

impl Policy for RequireCleanWorktree {
    async fn evaluate(&self, trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        if context.git.is_clean() {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!("Dirty worktree: {:?}", context.git.modified_files)))
        }
    }
}
```

### 2. RequireTests

```rust
pub struct RequireTests {
    command: String,  // "make test" or custom
    timeout_secs: u64,
}

impl Policy for RequireTests {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&self.command)
            .timeout(Duration::from_secs(self.timeout_secs))
            .output()
            .await?;
        
        if output.status.success() {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!("Tests failed:\n{}", String::from_utf8_lossy(&output.stderr))))
        }
    }
}
```

### 3. WIPLimit

```rust
pub struct WIPLimit {
    max_in_progress: usize,
}

impl Policy for WIPLimit {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        let tracker_ctx = context.tracker.as_ref().ok_or(PolicyError::NoTrackerContext)?;
        
        if tracker_ctx.in_progress_count < self.max_in_progress {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "WIP limit exceeded: {} >= {}",
                tracker_ctx.in_progress_count, self.max_in_progress
            )))
        }
    }
}
```

### 4. BranchProtection

```rust
pub struct BranchProtection {
    allowed_branches: Vec<String>,  // ["main", "release/*", "hotfix/*"]
}

impl Policy for BranchProtection {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        let current = &context.git.current_branch;
        let allowed = self.allowed_branches.iter().any(|pattern| {
            glob::Pattern::new(pattern).unwrap().matches(current)
        });
        
        if allowed {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "Branch '{}' not allowed. Allowed: {:?}",
                current, self.allowed_branches
            )))
        }
    }
}
```

### 5. RequireChangeLog

```rust
pub struct RequireChangeLog {
    filename: String,  // "CHANGELOG.md"
}

impl Policy for RequireChangeLog {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        // Check if CHANGELOG modified in current commit
        let changelog_modified = context.git.modified_files.contains(&self.filename);
        
        if changelog_modified {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "{} must be updated",
                self.filename
            )))
        }
    }
}
```

### 6. RequireCommitMessage

```rust
pub struct RequireCommitMessage {
    // Conventional Commits: feat|fix|docs|style|refactor|test|chore
    pattern: Regex,  // r"^(feat|fix|docs|refactor|test|chore)(\(.+\))?!?: .+"
}

impl Policy for RequireCommitMessage {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        let msg = &context.git.head_commit.message;
        
        if self.pattern.is_match(msg) {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "Commit message must follow Conventional Commits: {}",
                self.pattern.as_str()
            )))
        }
    }
}
```

### 7. RequireCodeReview

```rust
pub struct RequireCodeReview {
    min_approvals: usize,
}

impl Policy for RequireCodeReview {
    async fn evaluate(&self, _trigger: &TransitionTrigger, context: &ProjectContext) -> Result<PolicyResult> {
        let tracker_ctx = context.tracker.as_ref().ok_or(PolicyError::NoTrackerContext)?;
        
        if tracker_ctx.code_review_approvals >= self.min_approvals {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "Need {} approvals, have {}",
                self.min_approvals, tracker_ctx.code_review_approvals
            )))
        }
    }
}
```

### 8. CodeCoverageThreshold

```rust
pub struct CodeCoverageThreshold {
    min_percent: f32,
}

impl Policy for CodeCoverageThreshold {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        // Parse coverage report (e.g., tarpaulin output)
        let output = tokio::process::Command::new("cargo")
            .args(&["tarpaulin", "--out", "Json"])
            .output()
            .await?;
        
        let coverage: f32 = parse_coverage_json(&output.stdout)?;
        
        if coverage >= self.min_percent {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "Coverage {} < {}%",
                coverage, self.min_percent
            )))
        }
    }
}
```

### 9. SecurityScan

```rust
pub struct SecurityScan;

impl Policy for SecurityScan {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        // Run cargo-audit
        let output = tokio::process::Command::new("cargo")
            .args(&["audit", "--json"])
            .output()
            .await?;
        
        let vulnerabilities = parse_audit_json(&output.stdout)?;
        let high_severity = vulnerabilities.iter().filter(|v| v.severity == "high").count();
        
        if high_severity == 0 {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject(format!(
                "{} high-severity vulnerabilities found",
                high_severity
            )))
        }
    }
}
```

### 10. DocumentationCheck

```rust
pub struct DocumentationCheck;

impl Policy for DocumentationCheck {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        // Run cargo-doc with warnings-as-errors
        let output = tokio::process::Command::new("cargo")
            .args(&["doc", "--no-deps", "--document-private-items"])
            .env("RUSTDOCFLAGS", "-D warnings")
            .output()
            .await?;
        
        if output.status.success() {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject("Documentation incomplete or warnings present".to_string()))
        }
    }
}
```

### 11. ArchitectureValidation

```rust
pub struct ArchitectureValidation;

impl Policy for ArchitectureValidation {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        let output = tokio::process::Command::new("make")
            .arg("validate")
            .output()
            .await?;
        
        if output.status.success() {
            Ok(PolicyResult::Allow)
        } else {
            Ok(PolicyResult::Reject("Architecture violations found (make validate)".to_string()))
        }
    }
}
```

### 12. PerformanceRegression

```rust
pub struct PerformanceRegression {
    baseline: HashMap<String, f64>,  // benchmark_name -> time_ms
    max_regression_percent: f32,
}

impl Policy for PerformanceRegression {
    async fn evaluate(&self, _trigger: &TransitionTrigger, _context: &ProjectContext) -> Result<PolicyResult> {
        let output = tokio::process::Command::new("cargo")
            .args(&["bench", "--", "--output-format", "bencher"])
            .output()
            .await?;
        
        let current = parse_bencher_output(&output.stdout)?;
        
        for (name, baseline_time) in &self.baseline {
            if let Some(current_time) = current.get(name) {
                let regression = ((current_time - baseline_time) / baseline_time) * 100.0;
                
                if regression > self.max_regression_percent as f64 {
                    return Ok(PolicyResult::Reject(format!(
                        "Performance regression: {} +{:.1}%",
                        name, regression
                    )));
                }
            }
        }
        
        Ok(PolicyResult::Allow)
    }
}
```

---

## Configuration Example

```toml
[policies.require_clean_worktree]
enabled = true
severity = "Violation"

[policies.require_tests]
enabled = true
command = "make test"
timeout_secs = 300

[policies.wip_limit]
enabled = true
max_in_progress = 5

[policies.branch_protection]
enabled = true
allowed_branches = ["main", "release/*", "hotfix/*"]

[policies.require_changelog]
enabled = true
filename = "CHANGELOG.md"

[policies.require_commit_message]
enabled = true
pattern = "^(feat|fix|docs|refactor|test|chore)(\\(.+\\))?!?: .+"

[policies.require_code_review]
enabled = true
min_approvals = 2

[policies.code_coverage_threshold]
enabled = false  # Optional
min_percent = 85.0

[policies.security_scan]
enabled = true

[policies.documentation_check]
enabled = true

[policies.architecture_validation]
enabled = true

[policies.performance_regression]
enabled = false  # Optional
max_regression_percent = 10.0
```

---

## Testing Strategy

**Estimated 80 test cases** (vs 40 for MVP):

- Each policy: 6-7 tests (allow, reject, edge cases)
- Composition: 8 tests (AND/OR/NOT combinations)
- Configuration: 8 tests (parsing, defaults, validation)
- Dry-run mode: 4 tests (policies don't execute side effects)
- Error handling: 8 tests (timeout, missing context, etc.)

---

## Implementation Timeline

```
Week 2, Days 1-2: Tiers 1-2 (existing 4 + changelog/commit/review)
Week 2, Days 3-4: Tier 3 (coverage/security/docs)
Week 2, Day 5 + Week 3, Day 1: Tier 4 (architecture/performance) + testing
```

---

## Related Decisions

- **ADR-036**: Policy Guard Framework (defines Policy trait)
- **ADR-039**: Context Freshness (policies require fresh context)
- **ADR-037**: Orchestrator (invokes policies during transitions)

---

## Approval Chain

- [ ] Technical Lead: ✅ VOTED (Marlon)
- [ ] Product: (Pending)
- [ ] Team: (Pending)

**Vote Status**: ACCEPTED by Marlon on 2026-02-05.

---

## Consequences

### Positive ✅

- Production-complete policy set from v0.2.0
- Covers all common CI/CD gates
- Extensible for custom policies in Phase 9+
- Aligns with industry standards (GitHub, GitLab)

### Trade-offs ⚠️

- **+2 weeks development** (vs 7-policy hybrid)
- More complex configuration (12 policies vs 7)
- Some policies optional (coverage, performance) — can disable in config

### Benefits 🎯

- Eliminates "but we need security checks in v0.3" complaint
- Comprehensive feature ready on day 1
- Users trust v0.2 for production CI/CD
