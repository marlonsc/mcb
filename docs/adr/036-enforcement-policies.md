<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
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

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR-036: Enforcement Layer — Policies and Guards

## Status

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.

**Accepted** — 2026-02-06

- **Deciders:** Project team
- **Depends on:** [ADR-034](./034-workflow-core-fsm.md) (Workflow Core FSM), [ADR-035](./035-context-scout.md) (Context Scout)
- **Related:** [ADR-029](./050-manual-composition-root-dill-removal.md) (Hexagonal DI, superseded by ADR-050), [ADR-023](./023-inventory-to-linkme-migration.md) (linkme), [ADR-051](./051-seaql-loco-platform-rebuild.md) (Figment)
- **Series:**[ADR-034](./034-workflow-core-fsm.md) → [ADR-035](./035-context-scout.md) →**ADR-036** → [ADR-037](./037-workflow-orchestrator.md)

## Context

ADR-034 defines the workflow FSM with state transitions. ADR-035 provides typed `ProjectContext` snapshots. Before a transition is executed, the system must validate that project conditions are met — this is the role of**policy guards**.

Today, enforcement is either absent or ad-hoc:

| Scenario | Current Behavior | Desired Behavior |
| ---------- | ----------------- | ------------------ |
| Commit with dirty worktree | Allowed (no check) | Block or warn depending on transition |
| Start execution with 5 tasks already in-progress | Allowed | Block: WIP limit exceeded |
| Branch name doesn't follow convention | No validation | Warn: `feature/...` or `fix/...` expected |
| Deploy without passing tests | Depends on CI (external) | Guard: test suite must pass before verification |

**This ADR** defines a composable policy system where individual policies implement a shared trait, can be combined (AND/OR), and are configured via `mcb.toml`. Policies receive a `ProjectContext` (ADR-035) and a `TransitionTrigger` (ADR-034), returning a `PolicyResult` with typed violations.

### Requirements

- Individual policies implement a common trait
- Policies composable via AND/OR combinators
- Configurable per-project via `mcb.toml` (enable/disable, thresholds)
- Two evaluation modes: fail-fast (stop on first error) and collect-all (gather all violations)
- Severity levels: Error (blocks transition), Warning (logged but allowed), Info (informational)
- Extensible: new policies can be added without modifying existing code

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

### 5. Governance SSOT Boundary

This ADR defines the policy guard architecture: domain ports, composition, and provider boundaries. It does not own live operational rules.

Live enforcement policy is maintained in executable SSOTs:

- `AGENTS.md` for agent and operator coordination rules.
- `Makefile`, `makefiles/*.mk`, and `scripts/lib/mcb.sh` for local gates and command contracts.
- `.github/workflows/ci.yml` for CI wiring.
- `config/mcb-validate*.toml` for architecture validation rules.

The SSOT ownership decision is recorded in
`docs/adr/057-multi-agent-coordination-ssot-consolidation.md`; that ADR records
the boundary, but executable behavior remains in the sources above.

Policy lifecycle surfaces remain compile-time checks, local hooks, pre-transition workflow guards, CI gates, pre-merge checks, runtime guard evaluation, and post-merge monitoring. Their exact command lines, trigger order, and remediation text are read from the executable sources above, not restated here.

When a process rule changes, update its SSOT first and cite it from this ADR if architecture context is still useful. Do not add policy snippets, hook pseudo-code, GitHub Actions fragments, or remediation recipes to ADRs.

## 6. Guard Provider Implementation

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

### 7. Provider Registration (linkme)

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

### 8. Module Locations

| Crate | Path | Content |
| ------- | ------ | --------- |
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

- **Composable**: Policies combined via AND/OR without modifying each other.
- **Configurable**: Per-project settings via `mcb.toml`. Enable/disable and adjust thresholds without code changes.
- **Severity levels**: Errors block, warnings log. Teams choose enforcement strictness.
- **Extensible**: New policies implement `Policy` trait and register via linkme. No existing code modified.
- **Dry-run**: Policies can be tested without blocking transitions.
- **Context-aware**: Policies receive full `ProjectContext` (ADR-035), enabling rich conditions.

### Negative

- **Runtime evaluation cost**: Each transition evaluates all applicable policies. Mitigated by `applies_to()` filter and fail-fast mode.
- **Test command execution**: `RequireTestsPolicy` spawns a subprocess (e.g., `cargo test`). This is slow (seconds-to-minutes). Only triggered on `StartVerification`.
- **Config complexity**: 11 policies with individual settings adds config surface area. Mitigated by sensible defaults and disabled-by-default for non-essential policies (e.g., `version_bump`, `documentation_update`).
- **No runtime policy addition**: Policies are built at startup from config. Adding a new policy requires restart. Runtime dynamic policies deferred.

## Alternatives Considered

### Alternative 1: Tower-Style Middleware

- **Description:** Model policies as Tower `Layer`/`Service` middleware wrapping the FSM transition.
- **Pros:** Established pattern. Rich ecosystem (tower-HTTP, tower-retry).
- **Cons:** Tower is designed for request/response pipelines, not FSM transitions. Adaptation is awkward. Requires tower dependency.
- **Rejection reason:** Unnecessary complexity. The `Policy` trait with AND/OR composition is simpler and purpose-built.

### Alternative 2: Database-Driven Policies

- **Description:** Store policy configurations in SQLite and evaluate dynamically.
- **Pros:** Runtime reconfiguration without restart. Policy versioning.
- **Cons:** Adds query overhead per evaluation. Config is already in `mcb.toml` (Figment standard).
- **Rejection reason:** Over-engineering for 11 built-in policies. File-based config is sufficient and matches ADR-025 convention.

### Alternative 3: Hard-Coded Checks (No Policy Framework)

- **Description:** Embed checks directly in the WorkflowService transition logic.
- **Pros:** Simplest implementation. No trait, no composition.
- **Cons:** Not extensible. Every new check requires modifying WorkflowService. No per-project configuration.
- **Rejection reason:** Violates open/closed principle. Policy framework pays for itself after the second policy.

## Implementation Notes

### Code Changes

1. Add `policy.rs` entities to `mcb-domain/src/entities/`
2. Add `policy_guard.rs` and `policy.rs` ports to `mcb-domain/src/ports/providers/`
3. Add `GUARD_PROVIDERS` slice to `mcb-application/src/registry/`
4. Add `guard/` module to `mcb-providers/src/` with provider, composition, and**11 built-in policies**
5. Add `PoliciesConfig` and 11 settings structs to `mcb-infrastructure/src/config/`
6. Add `[policies]` section to `config/default.toml` with configurations for all 11 policies

> **v0.3.0 Migration Note:** Configuration is now Loco YAML (`config/development.yaml`, `config/test.yaml`), not Figment TOML (`config/default.toml`).

### Testing

- Unit tests: Each of the 11 policies with pass/fail cases (minimum 2 tests per policy = 22 tests)
- Unit tests: `PolicyResult::merge()`, `format_violations()`, severity handling
- Unit tests: `AllPolicies` (fail-fast and collect-all modes), `AnyPolicy` combinator
- Unit tests: Deny-wins semantics, ERROR vs WARNING enforcement
- Integration tests: `ConfigurablePolicyGuard` with real config, all 11 policies enabled/disabled
- Integration tests: Lifecycle points (compile-time, pre-commit, pre-transition, CI-time, post-merge)
- Estimated: **~80+ tests** (11 policies × 2 + integration + composition + semantics)

### Performance Targets

| Operation | Target |
| ----------- | -------- |
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

- `RequireTestsPolicy` executes a shell command from `mcb.toml`. The config file must be trusted (same as any TOML config). No user-supplied input reaches the command.

## References

- [gatehouse](https://docs.rs/gatehouse/latest/gatehouse/) — Policy composition patterns (evaluated)
- [ADR-034: Workflow Core FSM](./034-workflow-core-fsm.md) — `TransitionTrigger` consumed by guards
- [ADR-035: Context Scout](./035-context-scout.md) — `ProjectContext` consumed by guards
- [ADR-051: SeaQL + Loco.rs Platform Rebuild](./051-seaql-loco-platform-rebuild.md) — Config pattern
- [ADR-029: Hexagonal Architecture](./050-manual-composition-root-dill-removal.md) — DI pattern (superseded by ADR-050)
