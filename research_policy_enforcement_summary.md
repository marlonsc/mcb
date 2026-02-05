# Policy Enforcement Research - Executive Summary

## What We Learned

### Real-World Systems Analyzed
1. **OPA (Open Policy Agent)** - CNCF declarative policy engine
2. **HashiCorp Sentinel** - Infrastructure-as-code policy framework
3. **GitLab Security Policies** - Native CI/CD policy injection
4. **GitHub Actions** - Enterprise policy enforcement
5. **Cerbos** - Authorization-focused policy system

---

## 1. CRITICAL PATTERNS DISCOVERED

### Pattern 1: Mutation-Based Enforcement (vs Advisory)
- **GitLab Model**: Policies inject jobs that CANNOT be skipped
- **GitHub Model**: Status checks MUST pass before merge
- **Lesson**: Policies should modify/block, not just warn

### Pattern 2: Deny-Wins Conflict Resolution
```
When multiple policies apply:
- Any "deny" policy blocks regardless of "allow" policies
- Safest default (prevents security bypasses)
- Must be explicitly documented
```

### Pattern 3: Three Composition Levels
1. **Hierarchical** (GitLab): Parent → Group → Project inheritance
2. **Modular** (OPA): Import-based with rule definitions
3. **Pack-Based** (Sentinel): Grouped with merge strategies

---

## 2. WORKFLOW CONSTRAINTS FINDINGS

### WIP Limits Are Underutilized
- Should be **policy-enforced**, not just advisory
- Integration point: Pre-commit hook + CI gate
- Example: Block new PRs if `active_prs >= WIP_LIMIT`

### Clean Worktree Detection
- **NEVER auto-fix** (dangerous)
- Always LIST issues and require developer action
- Pre-commit hook pattern:
  ```bash
  git diff --quiet --exit-code || exit 1
  ```

### Branch Naming Has Production Patterns
Best regex (prevents double hyphens, leading/trailing hyphens):
```regex
^(?!.*--)((main|develop|feature|bugfix|release)/[a-z0-9-]+)?$
```

### Test Gating Should Be Mandatory
- All required tests must pass (AND semantics)
- Configuration per branch (main: 90%, develop: 80%)
- Fail on coverage decrease
- Integration: GitHub Actions, GitLab CI native

---

## 3. POLICY CONFIGURATION INSIGHTS

### YAML vs TOML
| Aspect | YAML | TOML |
|--------|------|------|
| Readability | Excellent | Good |
| Whitespace Significance | Yes (risky) | No (safer) |
| Comments | Yes | Yes |
| Validation | Good | Better |
| **Recommendation** | Use for discovery metadata | Use for strict policy defs |

### Configuration Should Include
```yaml
# Minimal required fields
policy:
  name: "descriptive_name"
  version: "1.0.0"
  enforcement: mandatory|advisory|warning
  scope: [commit, push, pull_request]
  conflict_resolution: deny_wins
```

---

## 4. TESTING & DEBUGGING GAPS

### What's Missing from Most Systems
1. ❌ **Unit test framework** for policies
2. ❌ **Dry-run mode** (test without enforcing)
3. ❌ **Rich error messages** with remediation
4. ❌ **Policy regression tests**
5. ❌ **Debug tracing** through rule evaluation

### Recommended Testing Pattern
```rego
# Unit test
test_valid_branch_allowed {
    allow with input as {"branch": "feature/auth"}
}

# Integration test (YAML)
tests:
  - name: "valid branch"
    input: {branch: "feature/auth"}
    expect: allow
```

### Debugging Infrastructure Needed
- Structured error output (JSON)
- Remediation steps included
- Debug info levels (basic/detailed/trace)
- Policy evaluation path visualization

---

## 5. CRITICAL GAPS IN ADR-036

### ✅ What's Covered
- Basic policy guard patterns
- Workflow constraints conceptually
- Configuration format options

### ❌ Missing (Must Add)
| Gap | Impact | Priority |
|-----|--------|----------|
| Explicit conflict resolution | Security risk | **CRITICAL** |
| Plugin architecture details | Implementation ambiguous | **HIGH** |
| Testing framework specification | Hard to validate policies | **HIGH** |
| Error messaging standard | Poor developer experience | **MEDIUM** |
| Policy lifecycle (versioning, deprecation, rollback) | Operational risk | **HIGH** |
| Configuration schema validation | Easy to introduce bugs | **MEDIUM** |

---

## 6. ACTIONABLE RECOMMENDATIONS

### For ADR-036 Enhancement

**1. Add Conflict Resolution Section**
```yaml
enforcement:
  conflict_resolution: deny_wins
  precedence_model: explicit_priority
  validation:
    prevent_conflicting_definitions: true
    test_combinations: true
```

**2. Define Plugin Architecture**
```yaml
policy_engine:
  discovery:
    directories: [".policies", "policies/"]
    pattern: "*.rego"
  loading:
    order: dependency_order
    hot_reload: true
    reload_interval: 30s
```

**3. Mandate Testing**
- Unit tests (OPA test format)
- Integration tests (YAML suite)
- Minimum 80% policy coverage
- Regression test catalog required

**4. Error Messaging Framework**
```json
{
  "status": "policy_violation",
  "policies": [{
    "name": "branch_naming",
    "reason": "Invalid pattern",
    "suggestion": "Use: feature/description",
    "docs": "url"
  }]
}
```

**5. Lifecycle Management**
- Semantic versioning for policies
- Deprecation notice period (30 days)
- Migration scripts required
- Rollback capability mandatory

---

## 7. IMPLEMENTATION PRIORITY

### Phase 1 (Sprint 1-2): Foundation
- [ ] Conflict resolution semantics
- [ ] Configuration schema
- [ ] Basic error messages

### Phase 2 (Sprint 3-4): Developer Experience
- [ ] Testing framework
- [ ] Dry-run mode
- [ ] Rich error messages with remediation

### Phase 3 (Sprint 5-6): Production Readiness
- [ ] Versioning system
- [ ] Deprecation process
- [ ] Rollback procedures

### Phase 4 (Sprint 7+): Integration
- [ ] Pre-commit hook enforcement
- [ ] CI gate integration
- [ ] Workflow constraint enforcement (WIP, clean worktree, branch naming, tests)

---

## 8. KEY DECISION POINTS FOR ADR-036

### Decision 1: Conflict Resolution Model
**Recommendation: Adopt deny-wins semantics**
- Most conservative (prevents security bypasses)
- Used by OPA, RBAC, Sentinel
- Must be explicit in documentation

### Decision 2: Composition Style
**Recommendation: Hybrid approach**
- Hierarchical for inheritance (like GitLab)
- Modular imports for composability (like OPA)
- Pack-based grouping for organization

### Decision 3: Configuration Format
**Recommendation: YAML for discovery, TOML for policies**
- YAML: Policy metadata (name, version, scope)
- TOML: Strict policy definitions (immutable defaults)
- Both support schema validation

### Decision 4: Runtime vs Compile-Time
**Recommendation: Hybrid**
- Critical security policies → compile-time (immutable)
- Operational policies → runtime (dynamic reload 30s)
- Feature flags/allowlists → runtime

### Decision 5: Plugin Loading Strategy
**Recommendation: Dependency order with hot-reload**
- Discover from `.policies/`, `policies/` directories
- Load in dependency order (respect imports)
- Hot-reload every 30s (config configurable)
- Namespace separation for isolation

---

## 9. CODE EXAMPLES FOR ADR-036

### Example 1: Complete Policy Definition
```yaml
# .beads/policies.yml
version: "1.0"

enforcement:
  model: deny_wins
  validation: enabled

policies:
  branch_naming:
    name: "Branch Naming Convention"
    version: "1.0.0"
    file: "policies/branch_naming.rego"
    scope: [push]
    priority: 100
    enforcement: mandatory
    
  clean_worktree:
    name: "Clean Worktree Check"
    version: "1.0.0"
    file: "policies/clean_worktree.rego"
    scope: [commit]
    priority: 90
    enforcement: mandatory
```

### Example 2: Policy Testing Suite
```yaml
# tests/policy_tests.yml
tests:
  branch_naming:
    - case: "valid_feature_branch"
      input: {branch: "feature/user-auth"}
      expect: allow
      
    - case: "invalid_double_hyphen"
      input: {branch: "feature/user--auth"}
      expect: deny
      reason: "double hyphens not allowed"

  clean_worktree:
    - case: "clean_workspace"
      input: {changes: 0}
      expect: allow
      
    - case: "uncommitted_changes"
      input: {changes: 3}
      expect: deny
```

### Example 3: Error Response Format
```json
{
  "timestamp": "2025-01-20T10:30:00Z",
  "status": "policy_violation",
  "severity": "error",
  "violated_policies": [
    {
      "name": "branch_naming",
      "version": "1.0.0",
      "reason": "Branch 'feature/my_feature' doesn't match pattern",
      "pattern": "^feature/[a-z0-9-]+$",
      "suggestion": "Rename to: feature/my-feature",
      "remediation": [
        "git branch -m feature/my_feature feature/my-feature",
        "git push -u origin feature/my-feature"
      ],
      "docs_url": "https://docs.example.com/policies/branch-naming"
    }
  ]
}
```

---

## 10. NEXT STEPS

1. **Review ADR-036** against these findings
2. **Identify gaps** in your specific system
3. **Prioritize additions** based on your risk profile
4. **Prototype implementation** with OPA/Rego
5. **Establish testing framework** early
6. **Document conflict resolution** explicitly
7. **Create operator runbooks** for policy changes

---

## References

### Systems Studied
- Open Policy Agent (OPA/Rego)
- HashiCorp Sentinel
- GitLab Security Policies (Pipeline Execution)
- GitHub Actions + Rulesets
- Cerbos

### Key Concepts Validated
- Deny-wins semantics (RBAC standard)
- Mutation-based enforcement (GitLab pattern)
- Hierarchical composition (enterprise best practice)
- Policy testing (Cerbos pattern)
- Hot-reload with fallback (OPA pattern)

### Industry References
- OWASP Policy as Code
- NIST cybersecurity framework
- Cloud Native Computing Foundation (CNCF) projects
