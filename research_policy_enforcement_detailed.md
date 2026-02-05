# Policy Enforcement & Composable Rules Frameworks: Comprehensive Research

## Executive Summary

This research analyzes policy enforcement patterns, composable rules frameworks, and workflow constraints across real-world CI/CD systems including OPA, HashiCorp Sentinel, GitLab, and GitHub Actions. The analysis identifies critical gaps in ADR-036 and provides concrete recommendations for a production-grade enforcement architecture.

---

## KEY FINDINGS

### 1. Conflict Resolution is Critical

Most systems lack explicit conflict resolution strategy. The safest default is **deny-wins semantics**: any deny blocks access regardless of other policies. This prevents accidental security bypasses.

### 2. Three Composition Patterns Dominate

-   **GitLab**: Hierarchical inheritance (parent → group → project)
-   **OPA**: Module-based imports with rule definitions
-   **Sentinel**: Policy packs with explicit merge strategies

### 3. Testing & Debugging Often Overlooked

Production systems need: unit tests, integration tests, dry-run modes, and rich error messages with remediation guidance.

### 4. WIP Limits & Clean Worktrees Need Policy Integration

These workflow constraints should be enforceable via pre-commit hooks integrated with CI gates, not just advisory.

---

## 1. POLICY GUARD PATTERNS

### GitHub Actions Approach

-   Policies define which Actions can run
-   Rulesets control deployments (newer)
-   Multiple policies combine as AND gates (all must pass)
-   Status checks are enforcement mechanism

### GitLab Approach (Pipeline Execution Policies)

-   Inject mandatory jobs into developer pipelines
-   NOT skippable by developers
-   Multiple policies apply in sequence
-   All applicable policies execute

### Key Pattern Insight

Policy enforcement should be **mutation-based** (inject/modify) rather than **advisory** (warn/fail softly). Developers should not be able to bypass critical policies.

---

## 2. COMPOSABLE RULES SYSTEMS

### OPA/Rego Pattern

**Logical AND** (implicit):

```rego
allow {
    input.user.admin == true
    input.request.method == "DELETE"
}
```

**Logical OR** (multiple definitions):

```rego
allow { input.user.admin == true }
allow { input.user.role == "moderator" }
```

**Negation**:

```rego
allow {
    not input.user.blocked
    input.authenticated == true
}
```

### Dynamic Composition

-   Load policies based on request attributes (team, environment)
-   Evaluate all applicable rules
-   Combine results via denial-of-service logic

---

## 3. CONFLICT RESOLUTION STRATEGIES

### Recommended: Deny-Wins Semantics

```
Allow + Allow = Allow
Allow + Deny = Deny ← Most conservative
Deny + Deny = Deny
```

### Implementation

1.  Validate policies don't have conflicting intent (pre-commit)
2.  Test policy combinations (integration tests)
3.  Use explicit priority when needed
4.  Document all override exceptions

---

## 4. WORKFLOW CONSTRAINTS

### WIP (Work-In-Progress) Limits

```yaml
wip_limits:
  in_progress: 3
  code_review: 2
  blocked:
    max_time_hours: 24
    auto_escalation: true
```

Enforcement via pre-commit hook:

```bash
active_prs=$(gh pr list --state open --json number | jq length)
if [ $active_prs -ge $WIP_LIMIT ]; then
  echo "Error: WIP limit ($WIP_LIMIT) exceeded"
  exit 1
fi
```

### Clean Worktree Verification

```bash
if ! git diff --quiet --exit-code; then
  echo "ERROR: Uncommitted changes detected"
  git diff --name-only
  exit 1
fi
```

**Critical**: Never auto-clean. Always list issues and require developer action.

### Branch Naming Validation

Production regex pattern:

```regex
^(?!.*--)(
  (main|master|develop|staging|release)$
  |^(feature|bugfix|hotfix)/[a-z0-9-]+(/[a-z0-9-]+)?$
  |^release/v?(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$
)$
```

Prevents:

-   Double hyphens
-   Hyphens at start/end
-   Arbitrary branch names

### Test Gating Integration

```yaml
test_gates:
  coverage_minimum: 80%
  branches:
    main: 90%
    develop: 80%
  fail_on_coverage_decrease: true
  required_tests:
    - unit_tests: 100%
    - integration_tests: 90%
```

ALL required tests must pass. Configuration per branch.

---

## 5. POLICY CONFIGURATION

### YAML vs TOML

**YAML Benefits**:

-   Human-readable
-   Comments allowed
-   Nested structures
-   Wide tool support

**TOML Benefits**:

-   No whitespace significance
-   Simpler grammar
-   Better validation
-   Type-safe by default

### Configuration Example (YAML)

```yaml
policies:
  - name: branch_naming
    file: policy/branches.rego
    scope: [push, pull_request]
    enforcement: mandatory
    
  - name: require_coverage
    file: policy/coverage.rego
    scope: [pull_request]
    enforcement: mandatory
    
  - name: commit_messages
    file: policy/commits.rego
    scope: [commit]
    enforcement: advisory
```

### Runtime vs Compile-Time

**Runtime** (dynamic reload):

-   Pros: Update without redeployment, A/B testing
-   Cons: Performance overhead, harder debugging

**Compile-Time** (build-time):

-   Pros: Consistency, type checking, optimized
-   Cons: Requires rebuild, slower iteration

**Recommended**: Hybrid approach

-   Critical security policies → compile-time
-   Feature flags/allowlists → runtime with 30s reload

---

## 6. TESTING & DEBUGGING

### Unit Test Pattern (OPA)

```rego
package policy.test

test_valid_feature_branch {
    allow with input as {"branch": "feature/auth"}
}

test_invalid_branch_denied {
    deny with input as {"branch": "my_random_branch"}
}
```

### Integration Test Pattern

```yaml
tests:
  - name: "High coverage passes"
    input: { coverage: 95 }
    expect: allow
    
  - name: "Low coverage fails"
    input: { coverage: 60 }
    expect: deny
    reason: "below 80% threshold"
```

### Debugging Infrastructure

1.  **Verbose output**:

```bash
opa eval -d policy/ -i input.json -f pretty --explain full
```

1.  **Policy dry-run**:

```bash
git commit --dry-run  # Test before actual commit
```

1.  **Rich error messages**:

```json
{
  "status": "policy_violation",
  "policies": [{
    "name": "branch_naming",
    "reason": "Branch must match pattern: feature/.*",
    "suggestion": "Rename to: feature/my-feature"
  }]
}
```

---

## 7. REAL-WORLD SYSTEMS COMPARISON

| System | Strength | Weakness | Composition |
|--------|----------|----------|-------------|
| **OPA** | Composable rules, Rego powerful | Learning curve | Module imports + rule defs |
| **Sentinel** | IaC focused, merge strategies | Closed-source | Policy packs |
| **GitLab** | Native CI/CD, mutation-based | Ultimate only, less flexible | Hierarchical inheritance |
| **GitHub** | Native integration, branch protection | Limited complexity | Rulesets per pattern |
| **Cerbos** | Authorization focus, embedded | Newer, smaller community | Policy bundles |

---

## 8. CRITICAL GAPS IN ADR-036

### Missing or Incomplete

1.  **Explicit Conflict Resolution Rule**

-   Currently undefined
-   Should specify deny-wins or alternative
-   Document all overrides

1.  **Plugin Architecture**

-   File discovery not defined
-   Loading order not specified
-   Module import mechanism unclear
-   Hot-reload behavior undefined

1.  **Testing Framework**

-   No unit test specification
-   Integration test pattern missing
-   Regression test requirements unclear

1.  **Error Messaging**

-   No structured error format
-   Remediation guidance missing
-   Debug information levels undefined

1.  **Configuration Validation**

-   Schema enforcement not specified
-   Pre-commit validation of policies missing
-   CI gate for policy changes not defined

1.  **Policy Lifecycle**

-   No versioning strategy
-   Deprecation process undefined
-   Migration path missing
-   Rollback procedures not specified

---

## 9. RECOMMENDED ADDITIONS TO ADR-036

### 1. Conflict Resolution (REQUIRED)

```yaml
enforcement:
  conflict_resolution: deny_wins  # Default safe
  precedence_model: explicit_priority
  validation:
    prevent_conflicting_definitions: true
    test_policy_combinations: true
```

### 2. Plugin Architecture

```yaml
policy_engine:
  plugin_discovery:
    directories: [".policies", "policies/"]
    file_pattern: "*.rego"
    recursive: true
  loading:
    order: dependency_order
    hot_reload: true
    reload_interval_seconds: 30
  module_system:
    imports: standard  # OPA-style imports
    namespace_separation: true
```

### 3. Configuration Format Standard

```yaml
policy_spec:
  version: "1.0"
  schema:
    strict: true
  metadata:
    name: string
    version: semver
    owner: team_id
    deprecated: boolean
    alternatives: [string]
  enforcement:
    mode: blocking | advisory | warning
    on_violation: deny | warn | audit
  conditions:
    apply_if:
      branch: pattern
      environment: [dev, staging, prod]
      team: [team_ids]
```

### 4. Error Messaging Framework

```yaml
policy:
  errors:
    format: structured_json
    include_fields:
      - violation_type
      - affected_resource
      - reason
      - suggestion
      - remediation_steps
      - debug_info_level: [basic, detailed, trace]
```

### 5. Testing Standard

```yaml
testing:
  unit_test_framework: opa_test_v1
  integration_test_format: yaml_test_suite
  required_coverage: 80%
  dry_run_support: true
  regression_test_catalog: required
```

### 6. Policy Lifecycle

```yaml
versioning:
  strategy: semver
  coexistence: allow_multiple_versions
  deprecation_notice_period_days: 30
  migration_scripts: required
  rollback_capability: required
```

---

## 10. IMPLEMENTATION ROADMAP

### Phase 1: Foundation

-   [ ] Define conflict resolution semantics
-   [ ] Document plugin architecture
-   [ ] Create configuration schema

### Phase 2: Tooling

-   [ ] Implement testing framework
-   [ ] Add error messaging system
-   [ ] Create dry-run mode

### Phase 3: Lifecycle

-   [ ] Add versioning support
-   [ ] Implement deprecation process
-   [ ] Create rollback mechanisms

### Phase 4: Integration

-   [ ] Pre-commit hook enforcement
-   [ ] CI gate integration
-   [ ] Git workflow enforcement

---

## 11. CONCRETE EXAMPLE: Complete Policy Stack

```yaml
# .beads/policies.yml
version: "1.0"

policies:
  branch_naming:
    file: "policies/branch_naming.rego"
    enforcement: mandatory
    scope: [push]
    priority: 100
    
  clean_worktree:
    file: "policies/clean_worktree.rego"
    enforcement: mandatory
    scope: [commit]
    priority: 90
    
  coverage_gates:
    file: "policies/coverage.rego"
    enforcement: mandatory
    scope: [pull_request]
    priority: 80
    
  wip_limits:
    file: "policies/wip_limits.rego"
    enforcement: advisory
    scope: [pull_request_create]
    priority: 50

conflict_resolution:
  model: deny_wins
  validator: enabled
  
error_handling:
  format: json
  include_debug: verbose
  
testing:
  unit_tests: "policies/tests/*.rego"
  integration_tests: "tests/policy_integration.yml"
```

---

## 12. CONCLUSION

Modern policy enforcement requires:

1.  **Clear semantics** (deny-wins conflict resolution)
2.  **Composable architecture** (module-based, hierarchical)
3.  **Robust testing** (unit + integration + regression)
4.  **Developer experience** (rich errors, remediation guidance)
5.  **Lifecycle management** (versioning, deprecation, migration)

ADR-036 should adopt these patterns from proven systems (OPA, GitLab, Sentinel) while adding the missing pieces (testing, lifecycle, debugging) to create a complete enforcement framework.
