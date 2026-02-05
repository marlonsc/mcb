# ADR-041: Error Recovery with VCS Gates Decision

**Status:** ACCEPTED ✅  
**Date:** 2026-02-05  
**Decision Maker:** Marlon (Technical Lead)  
**Approval:** VOTED & APPROVED WITH MODIFICATION

---

## Decision

**Implement Full Compensation Logic** for error recovery + automatic **VCS Gate for major/minor version changes**.

Two-part strategy:

### Part A: Full Compensation Logic (Error Recovery)

When workflow transition fails:

```
Task 1: ✅ Update package.json version
Task 2: ✅ Update CHANGELOG.md
Task 3: ❌ FAILS! → Security scan finds vulnerability

Recovery:
  Undo Task 2: Restore CHANGELOG.md
  Undo Task 1: Restore package.json
  → Session returns to `RolledBack` state
  → Log: "Rolled back due to security scan failure"
```

**Full compensation includes:**
- Database transaction rollback (SQLite ROLLBACK)
- File system restoration (git reset --hard)
- External API call reversal (if applicable)
- Memory state consistency

### Part B: VCS Gate for major/minor Versions

All changes to `major.minor` versions (e.g., 0.2 → 0.3) require:

```
Change Type: Version Bump (0.2.0 → 0.3.0)
  ↓
Automatic Gate Activation:
  1. MUST be on `main` or `release/*` branch
  2. MUST have PR review (ADR-040: RequireCodeReview policy)
  3. MUST pass all policies (tests, coverage, security, docs, architecture)
  4. MUST NOT be direct commit to main (PR-only)
  5. MUST be merged via `Squash and Merge` or `Create a Merge Commit` (not Fast-Forward)
  6. Automated: Code Review Gateway
  7. Automated: Architecture Validation Gateway
  8. Automated: Security Scan Gateway
  ↓
After Merge to main:
  - Create annotated git tag (vX.Y.Z)
  - Generate release notes
  - Publish to crates.io
  - Notification to team
```

---

## Rationale

### Part A: Why Full Compensation?

#### Against Simple Rollback

**Simple Rollback (initial recommendation):**
- Restore previous workflow state
- ✅ Faster to implement (2-3 days)
- ❌ Leaves side effects orphaned
  
**Example failure:**
```
Task 1: ✅ Publish to crates.io (external API call)
Task 2: ❌ Update release notes (fails)
→ Simple Rollback: restore session state only
→ Result: Package published, docs missing → inconsistency!
```

**Full Compensation (this decision):**
- Undo external API calls
- Revert database mutations
- Reset filesystem to pre-transition state
- ✅ Prevents orphaned side effects
- ✅ Ensures consistency across all systems
- ❌ More complex (but worth it)

#### Production Precedent

| Tool | Approach | Method |
|------|----------|--------|
| **Kubernetes** | Full compensation | Rollback deployments, cleanup resources |
| **Terraform** | Full compensation | Destroy created resources on failure |
| **GitLab CI/CD** | Full compensation | Cleanup artifacts, revert deployments |
| **GitHub Actions** | Simple rollback | Artifacts kept, job state reset |
| **Database Migrations** | Full compensation | Rollback schema + data to pre-migration |

**Observation**: Production systems handling state mutations use full compensation.

### Part B: Why VCS Gates for Version Changes?

#### The Problem

Version bump is a **critical operation**:
- Signals feature completeness to users
- Triggers automatic releases (crates.io, Docker, etc.)
- May be used by dependent projects immediately
- **Wrong version** = cascading impact

#### What Can Go Wrong Without Gates

```
Developer on branch feature/add-new-policy:
  - Modifies src/policies/mod.rs
  - Changes Cargo.toml: 0.2.0 → 0.3.0  (accidental!)
  - No tests run locally
  - Commits directly to main (pushes without PR)
  ↓
Automatic Publish:
  - cargo publish (crates.io)
  - Docker build published
  - GitHub release created
  - 2 hours later: discovers tests fail in v0.3.0
  - Can't unpublish crates.io (immutable)
  - Users download broken version
```

#### Gates Prevent This

1. **PR Requirement**: Change reviewed before merge
2. **Policy Gates**: Tests + security + coverage all pass
3. **Architecture Gate**: `make validate` passes
4. **Code Review**: Human eyes on version change
5. **Merge Strategy**: No direct commits to main

#### Production Pattern

All mature projects gate version changes:

| Project | Pattern |
|---------|---------|
| **Rust** (rustup) | Version must be in CHANGELOG, tag checked |
| **Node.js** | Version via release workflow (separate repo) |
| **Kubernetes** | Version in VERSION file, release PR required |
| **Docker** | Semantic versioning enforced in CI/CD |

---

## Implementation

### Part A: Compensation Framework

#### Structure

```rust
pub struct CompensationLog {
    session_id: String,
    task_id: String,
    action: String,           // "update_package_json", "publish_crate", etc.
    completed_at: Instant,
    compensation_fn: Box<dyn Fn() -> Result<()>>,  // How to undo
    status: CompensationStatus,
}

pub enum CompensationStatus {
    Pending,
    Compensated,
    Failed { reason: String },
}
```

#### Workflow Execution with Compensation

```rust
pub async fn execute_workflow_with_compensation(
    session: &mut WorkflowSession,
    tasks: Vec<WorkflowTask>,
) -> Result<(), WorkflowError> {
    let mut compensations = Vec::new();
    
    for task in tasks {
        match execute_task(&task).await {
            Ok(()) => {
                compensations.push(CompensationLog {
                    task_id: task.id.clone(),
                    action: task.action.clone(),
                    compensation_fn: task.compensation_fn.clone(),
                    status: CompensationStatus::Pending,
                    completed_at: Instant::now(),
                });
            }
            Err(e) => {
                // Failure: compensate in reverse order
                eprintln!("Task {} failed: {}", task.id, e);
                eprintln!("Starting compensation...");
                
                for compensation in compensations.iter_mut().rev() {
                    match (compensation.compensation_fn)() {
                        Ok(()) => {
                            compensation.status = CompensationStatus::Compensated;
                            eprintln!("✓ Compensated: {}", compensation.action);
                        }
                        Err(e) => {
                            compensation.status = CompensationStatus::Failed {
                                reason: e.to_string(),
                            };
                            eprintln!("✗ Failed to compensate: {} ({})", compensation.action, e);
                        }
                    }
                }
                
                // Log all compensations
                session.compensation_log = compensations;
                session.state = WorkflowState::Failed {
                    reason: e.to_string(),
                    compensated: true,
                };
                
                return Err(e);
            }
        }
    }
    
    Ok(())
}
```

#### Compensation Examples

**Task 1: Update package.json**
```rust
WorkflowTask {
    id: "task-version-bump",
    action: "update_package_json",
    action_fn: |_| {
        let new_version = "0.3.0";
        let cargo_toml = std::fs::read_to_string("Cargo.toml")?;
        let updated = cargo_toml.replace("version = \"0.2.0\"", "version = \"0.3.0\"");
        std::fs::write("Cargo.toml", updated)?;
        Ok(())
    },
    compensation_fn: |_| {
        // Restore from git
        git_reset_hard_for_file("Cargo.toml")
    },
}
```

**Task 2: Publish to crates.io**
```rust
WorkflowTask {
    id: "task-publish-crate",
    action: "publish_crate",
    action_fn: |_| {
        execute_command("cargo publish")
    },
    compensation_fn: |_| {
        // Can't unpublish crates.io (immutable)
        // Instead: mark as yanked
        execute_command("cargo yank --vers 0.3.0")
    },
}
```

### Part B: VCS Gate Implementation

#### Version Change Detection

```rust
pub async fn detect_version_change(
    git_context: &GitContext,
) -> Result<Option<VersionBump>> {
    let old_version = get_version_from_file("Cargo.toml", "0.2.0")?;
    let new_version = get_version_from_file_staged("Cargo.toml")?;
    
    if parse_version(old_version) != parse_version(new_version) {
        let bump = VersionBump {
            from: old_version,
            to: new_version,
            bump_type: detect_bump_type(old_version, new_version),
        };
        return Ok(Some(bump));
    }
    
    Ok(None)
}

pub enum BumpType {
    Patch,   // 0.2.1 → 0.2.2 (bugfix, no gate needed)
    Minor,   // 0.2.0 → 0.3.0 (feature, GATE REQUIRED)
    Major,   // 0.2.0 → 1.0.0 (breaking, GATE REQUIRED)
}
```

#### VCS Gate Policy

```rust
pub struct VersionChangeGate;

impl Policy for VersionChangeGate {
    async fn evaluate(
        &self,
        trigger: &TransitionTrigger,
        context: &ProjectContext,
    ) -> Result<PolicyResult> {
        // Only applies if version is changing
        let version_bump = detect_version_change(&context.git).await?;
        if version_bump.is_none() {
            return Ok(PolicyResult::Allow);  // No version change, skip gate
        }
        
        let bump = version_bump.unwrap();
        
        // Patch versions don't require gate
        if bump.bump_type == BumpType::Patch {
            return Ok(PolicyResult::Allow);
        }
        
        // Major/Minor require all gates
        if bump.bump_type == BumpType::Minor || bump.bump_type == BumpType::Major {
            // Check: PR review approved?
            if context.tracker.code_review_approvals < 2 {
                return Ok(PolicyResult::Reject(
                    "Version bump requires 2+ code review approvals".to_string()
                ));
            }
            
            // Check: on protected branch?
            if !["main", "release/*"].contains(&context.git.current_branch.as_str()) {
                return Ok(PolicyResult::Reject(
                    "Version bump must be on main or release/* branch".to_string()
                ));
            }
            
            // Check: architecture valid?
            let arch_result = execute_command("make validate").await?;
            if !arch_result.success {
                return Ok(PolicyResult::Reject(
                    "Architecture validation failed (make validate)".to_string()
                ));
            }
            
            // All gates pass
            return Ok(PolicyResult::Allow);
        }
        
        Ok(PolicyResult::Allow)
    }
}
```

#### Configuration

```toml
[policies.version_change_gate]
enabled = true
require_pr = true
min_approvals = 2
require_merge_commit = true  # Not fast-forward
allowed_branches = ["main", "release/*"]
gates = [
    "code_review",
    "architecture_validation",
    "security_scan",
    "tests_pass",
]
```

---

## Testing Strategy

### Part A: Compensation (30 test cases)

- Task execution success path ✓
- Single task failure with compensation ✓
- Multiple tasks with partial failure ✓
- Compensation failure (external API unreachable) ✓
- Session state consistency after compensation ✓
- Audit log captures all compensations ✓

### Part B: VCS Gate (25 test cases)

- Patch version bump (no gate) ✓
- Minor version bump (gate required) ✓
- Major version bump (gate required) ✓
- Insufficient approvals (reject) ✓
- Wrong branch (reject) ✓
- Architecture validation fails (reject) ✓
- All gates pass (allow) ✓

**Total: 55 test cases** for Part A + Part B

---

## Schema Changes

### SQLite Schema Extension

```sql
CREATE TABLE compensation_logs (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    action TEXT NOT NULL,
    completed_at DATETIME NOT NULL,
    compensation_status TEXT NOT NULL,  -- "Pending", "Compensated", "Failed"
    failure_reason TEXT,
    FOREIGN KEY (session_id) REFERENCES workflow_sessions(id)
);
```

---

## Timeline Impact

### Part A: Full Compensation
- Domain entities: 1 day
- Execution framework: 2 days
- Integration tests: 1 day
- **Subtotal: 4 days**

### Part B: VCS Gate
- Version change detection: 0.5 days
- VersionChangeGate policy: 1 day
- Integration with transition flow: 1 day
- Tests: 1.5 days
- **Subtotal: 4 days**

**Total Additional: 8 days** (previously estimated 1 week for simple rollback)

Updated timeline:
```
Original: 4-5 weeks
With full compensation + VCS gate: 4.5-5.5 weeks  (+0.5 week)
```

---

## Related Decisions

- **ADR-038**: FSM States (Failed/RolledBack states persist compensation logs)
- **ADR-040**: Policies (VersionChangeGate policy added)
- **ADR-037**: Orchestrator (invokes compensation framework)

---

## User-Facing Impact

### Developers

```
Session: Transition from Executing → Verifying

Policy Check: Security scan finds vulnerability
→ Transition DENIED
→ Session state: Failed (Compensated)
→ All side effects UNDONE
→ Git working tree CLEAN
→ Next action: Fix vulnerability, retry

Log output:
  ✓ Update package.json (compensated)
  ✓ Update CHANGELOG.md (compensated)
  ✗ Publish crate (failed - security scan)
  → Session rolled back successfully
  → Ready for retry
```

### Version Bump Workflow

```
Developer: Submits PR bumping 0.2.0 → 0.3.0
  ↓
Automatic Gates:
  ✓ Tests pass (50/50)
  ✓ Coverage maintained (92% → 91.5%)
  ✓ Security scan clean
  ✓ Architecture valid
  ✓ Documentation complete
  ✓ Code review: 2 approvals
  ✓ On main branch
  ✓ Merge commit (not fast-forward)
  ↓
Merge to main:
  → Automatic git tag: v0.3.0
  → cargo publish (crates.io)
  → Docker build published
  → GitHub release created
  → Team notification
```

---

## Approval Chain

- [ ] Technical Lead: ✅ VOTED (Marlon) - WITH MODIFICATION
  - Original: Simple Rollback only
  - Modified by user: Full Compensation + VCS Gate
- [ ] Architect: (Pending)
- [ ] Team: (Pending)

**Vote Status**: ACCEPTED by Marlon on 2026-02-05 with modification.

**User's Rationale**: 
> "Você tem que fazer full compensation, mas a ingestão do código no main e branches de major/minor versions tem que passar por tipo um PR e code gateway pelos VCS, para que isso não aconteça."
>
> (Translation: "You must do full compensation, but code ingestion into main and major/minor version branches must go through a PR and code gateway via VCS, so this doesn't happen.")

---

## Next Steps

1. Architect confirms compensation framework design is sound
2. Team confirms VCS gate aligns with deployment policy
3. Begin implementation Week 2 with both strategies
4. Test gates thoroughly before merge
