# 🚀 Release Roadmap: v0.3.0 → v0.4.0

**Status**: Planning Phase (v0.2.0 complete ✅)  
**Coordination**: v0.3.0 execution unblocks v0.4.0 implementation  
**Timeline**: v0.3.0 (Q1 2026) → v0.4.0 (Q2 2026)

---

## 📋 CONTEXT: v0.2.0 Complete

✅ **v0.2.0 Released** - Documentation Refactoring

-   ADR consolidation & metadata standardization
-   44 ADRs with YAML frontmatter
-   Updated CHANGELOG, ROADMAP, README
-   Release tag: `v0.2.0` (pushed to remote)
-   Branch: `release/v0.2.0` (ready to merge to main)

---

## 🎯 v0.3.0: Workflow System Implementation

**Release Goal**: Complete workflow architecture (ADR-034-038) + context scouting system  
**Dependencies**: ADR-034-038 fully specified and design-reviewed  
**Unblocks**: v0.4.0 implementation (integrated context system)

### v0.3.0 Scope (Phase Breakdown)

#### Phase 1: Complete ADR-034-038 Specification

-   **ADR-034**: Workflow Core FSM (design finalization)
-   **ADR-035**: Context Scout Architecture (resolve 7 TODO markers)
-   **ADR-036**: Enforcement Policies (complete policy enforcement spec)
-   **ADR-037**: Orchestrator Pattern (finalize orchestration logic)
-   **ADR-038**: Multi-Tier Execution (complete execution tiers)

**Deliverables**:

-   All 5 ADRs marked IMPLEMENTED status
-   No TODO markers remaining
-   Cross-ADR dependencies verified
-   Implementation examples provided

#### Phase 2: Core Workflow Infrastructure

-   **FSM State Machine**: Implement workflow state transitions
-   **Context Scout**: Build context gathering module
-   **Policy Engine**: Implement enforcement policy system
-   **Orchestrator**: Build task orchestration layer
-   **Execution Tiers**: Implement execution tier management

**Deliverables**:

-   5 new crates or modules for workflow system
-   Unit tests (90%+ coverage)
-   Integration tests with existing system
-   Documentation + examples

#### Phase 3: Quality Assurance & Testing

-   Full test suite (target: 95%+ code coverage)
-   Architecture validation (Phase-9 checks)
-   Performance benchmarking
-   Documentation completeness

**Deliverables**:

-   `make quality` passes (0 errors)
-   `make validate` passes (0 violations)
-   Benchmark results documented
-   Release notes prepared

#### Phase 4: Release v0.3.0

-   Merge to `main` from `release/v0.3.0` branch
-   Create tag `v0.3.0`
-   GitHub release with detailed notes
-   Update version badges to 0.3.0

**Deliverables**:

-   Tag `v0.3.0` pushed to remote
-   Release page created on GitHub
-   README updated with v0.3.0 features
-   Beads issues closed

---

## 🔗 COORDINATION: v0.3.0 → v0.4.0

### What v0.3.0 Provides for v0.4.0

**v0.4.0 Scope** (being planned by parallel agent):

-   Integrated Context System
-   Multi-agent collaboration
-   Context merging & sharing
-   Session-based context lifecycle

**v0.3.0 Unblocks**:

1.  **Workflow Foundation**: v0.4.0 needs workflow FSM (ADR-034)
2.  **Context Scout**: v0.4.0 needs context gathering (ADR-035)
3.  **Policy System**: v0.4.0 needs policy enforcement (ADR-036)
4.  **Orchestration**: v0.4.0 needs orchestrator (ADR-037)
5.  **Execution Tiers**: v0.4.0 needs multi-tier support (ADR-038)

### Interface Contracts (v0.3.0 → v0.4.0)

**Workflow FSM Public Interface** (v0.3.0 defines, v0.4.0 uses):

```rust
// v0.3.0: Implement
pub struct WorkflowFSM { ... }
pub trait WorkflowState { ... }
pub enum WorkflowTransition { ... }

// v0.4.0: Extend with context merging
impl WorkflowFSM {
    pub fn merge_context(&mut self, other: &Context) -> Result<()> { ... }
}
```

**Context Scout Interface** (v0.3.0 defines, v0.4.0 uses):

```rust
// v0.3.0: Implement
pub trait ContextScout { ... }
pub struct SearchContext { ... }

// v0.4.0: Add multi-agent support
impl ContextScout {
    pub fn scout_with_agents(&self, agents: &[Agent]) -> Result<Context> { ... }
}
```

**Policy Enforcement** (v0.3.0 defines, v0.4.0 uses):

```rust
// v0.3.0: Implement basic policies
pub enum Policy { ... }
pub struct PolicyEngine { ... }

// v0.4.0: Add context-aware policies
impl PolicyEngine {
    pub fn enforce_with_context(&self, ctx: &Context) -> Result<()> { ... }
}
```

---

## 📊 v0.3.0 Execution Plan

### Beads Issues (Preliminary)

| Epic | Task | Type | Priority | Depends On |
|------|------|------|----------|-----------|
| **Spec Completion** | ADR-034-038 final review | feature | P1 | — |
| **ADR-034 FSM** | Implement WorkflowFSM struct | feature | P1 | Spec complete |
| **ADR-035 Scout** | Implement ContextScout trait | feature | P1 | ADR-034 |
| **ADR-036 Policies** | Implement PolicyEngine | feature | P2 | ADR-034 |
| **ADR-037 Orchestrator** | Implement TaskOrchestrator | feature | P2 | ADR-035 |
| **ADR-038 Tiers** | Implement ExecutionTiers | feature | P2 | ADR-037 |
| **Testing** | Unit + integration tests | task | P1 | All features |
| **Quality** | `make quality` passes | task | P1 | Testing |
| **Release** | Tag v0.3.0 + GitHub release | task | P0 | Quality |

**Estimated effort**: 4-8 weeks (depending on parallelization)

### Branches Strategy

```
main (stable)
  ↑
  └─ release/v0.3.0 (feature branch)
      ├─ feature/adr-034-fsm
      ├─ feature/adr-035-scout
      ├─ feature/adr-036-policies
      ├─ feature/adr-037-orchestrator
      └─ feature/adr-038-tiers (all converge to release/v0.3.0)

main (stable)
  ↑
  └─ feature/v0.4.0-context-system (parallel, starts after v0.3.0 spec done)
      ├─ feature/multi-agent-support
      ├─ feature/context-merging
      ├─ feature/session-lifecycle
      └─ feature/integrated-context (converge to release/v0.4.0)
```

---

## 🔄 Parallel Execution: v0.3.0 & v0.4.0 Planning

### Timeline

| When | v0.3.0 Activity | v0.4.0 Activity | Blocker |
|------|---|---|---|
| **Week 1-2** | ADR-034-038 spec finalization | v0.4.0 requirements gathering | — |
| **Week 3** | v0.3.0 implementation starts | Design review (v0.3.0 APIs) | ADR specs |
| **Week 4-6** | Feature development (5 ADRs) | v0.4.0 design doc creation | v0.3.0 API contracts |
| **Week 7** | Testing + QA | Implementation prep | v0.3.0 APIs finalized |
| **Week 8** | v0.3.0 release candidate | v0.4.0 implementation starts | v0.3.0 release |
| **Week 9-12** | v0.3.0 released ✅ | v0.4.0 feature development | — (parallel) |
| **Week 13** | Bugfixes + maintenance | v0.4.0 testing + QA | — |
| **Week 14+** | v0.3.1 patches | v0.4.0 release candidate | — |

### Communication Protocol (v0.3.0 ↔ v0.4.0 agents)

**Interface Lock-in Points**:

1.  **Week 3**: v0.3.0 publishes preliminary API signatures (from ADRs)
2.  **Week 4**: v0.4.0 agent reviews APIs, provides feedback
3.  **Week 6**: v0.3.0 finalizes APIs, publishes stable interface
4.  **Week 7**: v0.4.0 implementation uses finalized v0.3.0 APIs

**Sync Points** (every Monday):

-   v0.3.0 agent: Reports ADR completion % + API changes
-   v0.4.0 agent: Reports design progress + API requirements
-   Resolve conflicts/changes asynchronously

**Breaking Change Protocol**:

-   If v0.4.0 needs API change in v0.3.0:
    -   Submit issue to v0.3.0 backlog
    -   Discuss deadline vs. impact
    -   If urgent: add to v0.3.1 patch cycle
    -   If non-urgent: defer to v0.4.0 compatibility layer

---

## ✅ Success Criteria (v0.3.0)

### Code Quality

-   ✅ `make quality` passes (0 errors)
-   ✅ `make validate` passes (0 violations)
-   ✅ Test coverage ≥ 95%
-   ✅ All clippy warnings suppressed or fixed
-   ✅ No `unwrap()` or `expect()` in non-test code

### Documentation

-   ✅ ADR-034-038 all IMPLEMENTED status
-   ✅ API documentation complete (rustdoc)
-   ✅ Examples provided for each major component
-   ✅ CHANGELOG.md updated with v0.3.0 features
-   ✅ Migration guide from v0.2.0 → v0.3.0 (if breaking)

### Testing

-   ✅ Unit tests pass (all crates)
-   ✅ Integration tests pass
-   ✅ Benchmarks run successfully
-   ✅ No flaky tests

### Release

-   ✅ Tag `v0.3.0` created and pushed
-   ✅ GitHub release page created
-   ✅ README updated to v0.3.0
-   ✅ Version badges updated
-   ✅ All Beads issues closed

---

## 📞 Handoff to v0.4.0 Agent

**When v0.3.0 is released**, v0.4.0 agent receives:

1.  **Stable APIs**: Full Rust trait definitions + examples
2.  **Documentation**: ADR-034-038 (IMPLEMENTED status)
3.  **Code**: Workflow system implementation (v0.3.0)
4.  **Tests**: Integration test examples for v0.4.0 to extend
5.  **Roadmap**: v0.4.0 phase planning (Phases 1-6)

**v0.4.0 Can Then**:

-   Implement multi-agent context merging
-   Extend WorkflowFSM with context awareness
-   Add session-based lifecycle management
-   Implement integrated context system (ADR-041-046)

---

## 🎯 Next Action (v0.3.0 Planning)

### Immediate (after v0.2.0 release)

1.  Create branch `release/v0.3.0` from `main`
2.  Create feature branches for ADR-034-038 implementation
3.  Publish preliminary API signatures
4.  Notify v0.4.0 agent: "v0.3.0 API contracts published, awaiting feedback"

### Planning

1.  Generate detailed v0.3.0 Beads issues (12-15 tasks)
2.  Set up parallel development coordination
3.  Schedule sync points (weekly)
4.  Create v0.4.0 placeholder plan (for v0.4.0 agent)

### Execution

1.  Each ADR-034-038 implementation as separate feature
2.  Regular integration testing
3.  Weekly progress reports
4.  API feedback loops with v0.4.0 agent

---

## 📝 v0.4.0 Preview (Being Planned by Parallel Agent)

**v0.4.0 Scope**: Integrated Context System (ADR-041-046)

**Depends on v0.3.0**:

-   ✅ WorkflowFSM (ADR-034)
-   ✅ ContextScout (ADR-035)
-   ✅ PolicyEngine (ADR-036)
-   ✅ TaskOrchestrator (ADR-037)
-   ✅ ExecutionTiers (ADR-038)

**v0.4.0 Will Add**:

-   Multi-agent collaboration
-   Context merging & sharing
-   Session lifecycle management
-   Global memory patterns
-   Hierarchical planning

**v0.4.0 Release Timeline**: Q2 2026 (after v0.3.0 Q1)

---

## 🚀 Status Summary

| Version | Status | Timeline | Dependency |
|---------|--------|----------|-----------|
| **v0.2.0** | ✅ Released | Done | — |
| **v0.3.0** | 📋 Planning | Q1 2026 (4-8 weeks) | Main unblock |
| **v0.4.0** | 🎨 Designing | Q2 2026 | Depends on v0.3.0 |

**Coordination**: Weekly sync between v0.3.0 and v0.4.0 agents  
**Communication**: Via Beads issues + GitHub discussions  
**Status**: Ready to begin v0.3.0 implementation phase

---

**Next**: Execute v0.3.0 planning + coordinate with v0.4.0 agent
