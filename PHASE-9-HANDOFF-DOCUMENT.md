# Phase 9 v0.4.0 Handoff Document

**Date**: 2026-02-05  
**Status**: ✅ READY FOR EXECUTION  
**Target Start**: February 17, 2026  
**Duration**: 4 weeks (Feb 17 - Mar 16)

---

## 📋 What You're Inheriting

### 1. Complete Architecture Foundation

✅ **ADR-041-046** (6 new ADRs, committed)
- v0.4.0 Integrated Context System specification
- 5-layer architecture: VCS → Indexing → Memory → Graph → Search → Policies
- 14,000+ lines of detailed design + code examples
- Aligned with MCB's Clean Architecture patterns

✅ **ARCHITECTURE_ALIGNMENT_ADR-041-046.md** (Binding document)
- MCB architecture patterns documented
- 9 corrections identified + applied
- Corrected file structure (exact file paths)
- Trait signatures matching MCB idioms
- DI wiring guidance (dill Catalog pattern)
- Risk assessment + success criteria

✅ **RESEARCH_SYNTHESIS_V0.4.0.md** (Evidence)
- 4 parallel research agents findings
- 20+ academic papers (RAG, knowledge graphs, freshness)
- 25+ production Rust crates validated
- Tech stack justification

### 2. Executable Beads Issues (35 total)

**Priority P0 Blocker (1 issue)**:
- `mcb-ii2`: Lock ADR-035 dependency (prerequisite for Week 1)

**Priority P1 Week 1 Issues (34 tasks)**:
- **9 Corrections**: Architecture alignment fixes
- **5 Domain+Ports**: Entities + trait definitions
- **5 Providers**: Implementation of all 4 new provider types
- **3 Services**: Context search, routing, versioning
- **3 DI+Infrastructure**: Catalog wiring, config, compensation
- **4 MCP Tools**: search, snapshot, timeline, validate
- **5 Testing**: Unit, integration, validation, performance

**All dependencies linked** in Beads (visualizable + automated)

### 3. Success Criteria (Locked)

**Architecture Compliance**:
- ✅ Zero CA (Clean Architecture) violations
- ✅ Strict dependency direction (inward only)
- ✅ All ports in mcb-domain
- ✅ All implementations in correct layers

**Code Quality**:
- ✅ 70+ tests, 85%+ coverage on domain
- ✅ `make fmt`, `make lint`, `make test` all passing
- ✅ `make validate` zero violations
- ✅ No `unwrap()` or `expect()` in production

**Performance**:
- ✅ Graph extraction <1ms per file
- ✅ Hybrid search <500ms per query
- ✅ Snapshots <10ms creation
- ✅ Memory <100MB for 24h history

**Integration**:
- ✅ FSM ↔ Context ↔ Policies fully integrated
- ✅ Compensation rollback working
- ✅ All 4 layers working together

---

## 🚀 Execution Plan

### Phase 9 Week 1: Corrections + Graph Infrastructure (Feb 17-22)

**Daily Breakdown**:

**Day 1-2 (Mon-Tue): Corrections (Issues 2-10)**
- Update ADR-041-046 docs (remove dual roles, clarify layers)
- Clarify ADR-035, BeadsTask, event bus patterns
- Decision: ContextService concrete vs trait
- Decision: BeadsTask import contract

**Day 2-3 (Tue-Wed): Domain Layer (Issues 11-15)**
- Create mcb-domain/src/entities/context.rs
- Create mcb-domain/src/ports/infrastructure/context.rs (ContextRepository, ContextGraphTraversal)
- Create mcb-domain/src/ports/providers/semantic_extractor.rs
- Create mcb-domain/src/ports/providers/full_text_search.rs
- Acceptance: All 4 files compile, basic tests pass

**Day 3-4 (Wed-Thu): Providers (Issues 16-20)**
- SqliteContextRepository impl (with tests)
- TreeSitterSemanticExtractor impl (caching via Moka)
- CodeGraph entity + petgraph DAG builder
- TantivyFullTextSearchProvider impl
- linkme provider registration
- Acceptance: All providers compile + 4+ integration tests

**Day 5 (Fri): Services + DI (Issues 21-26)**
- ContextSearchService (mcb-application, not trait)
- TaskRouterService (Stage 1: AST-based)
- VersionedContextService wrapper
- dill Catalog wiring (add all providers + services)
- Figment config ([context] section)
- CompensationHandler (mcb-infrastructure)
- Acceptance: Services compose correctly, DI resolves

**Day 6 (Fri-Mon): Tools + Validation (Issues 27-35)**
- 4 MCP tool handlers (search, snapshot, timeline, validate)
- 15+ unit tests (entities, repos, services)
- 10+ integration tests (DI, composition, E2E)
- `make validate` clean, `make test` green
- Performance benchmarks
- Acceptance: All tests passing, metrics documented

### Phase 9 Weeks 2-4: Search Engine, Versioning, Integration

(Follow same pattern: design → port traits → implement → test → integrate → verify)

**Week 2: Hybrid Search Engine (ADR-043)**
- tantivy FTS integration
- vecstore + RRF fusion
- Freshness weighting
- Graph expansion
- Target: <500ms per query

**Week 3: Versioning + Snapshots (ADR-045)**
- Immutable snapshots (im::Vector)
- TTL garbage collection
- Time-travel queries
- Staleness tracking (time + signals)
- Target: <20ms time-travel, <100MB memory

**Week 4: FSM + Policy Integration (ADR-046)**
- FSM state gates context freshness
- Policies enforce scope boundaries
- Compensation + rollback
- Event publishing
- Full E2E testing

---

## 🔗 Key Dependencies & Blockers

**CRITICAL**: Must complete BEFORE Week 1 starts

- [ ] **ADR-035 locked** (interface stable, no changes)
- [ ] **Corrections approved** (9 changes vetted by team)
- [ ] **BeadsTask contract clarified** (import or entity?)
- [ ] **Phase 8 completion confirmed** (ADR-034-037 stable)

If ANY of these are not complete by Feb 14, Phase 9 **cannot start Feb 17**.

---

## 📚 Reference Documents

**In This Repository**:
1. `ARCHITECTURE_ALIGNMENT_ADR-041-046.md` — Primary reference for Week 1
2. `RESEARCH_SYNTHESIS_V0.4.0.md` — Evidence for tech stack choices
3. `docs/adr/041-*.md` through `docs/adr/046-*.md` — Full specifications
4. `CLAUDE.md` — MCB development guide (make commands, quality gates)
5. `docs/architecture/ARCHITECTURE.md` — MCB architecture overview

**Beads Issues**:
- 35 issues created with full descriptions
- Dependencies visualizable in Beads
- All acceptance criteria documented
- Estimated effort: 18-25 hours / 5-7 days

**In Your Session Memory**:
- `v0.4.0 research complete` — Research findings
- `workflow-mode-analysis` — Architecture alignment findings
- `adr-041-046` — Complete implementation guidance

---

## ⚠️ Known Risks & Mitigations

| Risk | Probability | Mitigation |
|------|---|---|
| **ADR-035 not locked** | HIGH | Verify in checkpoint meeting (Feb 14) |
| **tree-sitter-graph immaturity** | MEDIUM | Fallback to AST walking if needed (Day 3) |
| **RRF weights suboptimal** | LOW | A/B test + tune in Week 4 |
| **Snapshot memory overhead** | LOW | TTL GC configured (24h + archive) |
| **Phase 8 slip** | MEDIUM | Have Phase 9 Week 1-2 independent path |

---

## ✅ Pre-Execution Checklist (Do This Feb 10-14)

**Architecture Review**:
- [ ] Team reviews ARCHITECTURE_ALIGNMENT_ADR-041-046.md
- [ ] 9 corrections approved
- [ ] ADR-035 locked (no changes after Feb 14)
- [ ] File structure approved
- [ ] Trait signatures finalized

**Beads Setup**:
- [ ] All 35 issues visible in `bd ready`
- [ ] Dependencies correctly linked
- [ ] P0 blocker (ADR-035 lock) marked
- [ ] Estimate verified (18-25 hours)

**Environment**:
- [ ] `make quality` passes (baseline clean)
- [ ] `make validate` clean (baseline 0 violations)
- [ ] MCB 0.2.0 stable on release/v0.2.0
- [ ] Team members assigned to Week 1 tasks

---

## 🎯 Success Looks Like (Feb 23, End of Week 1)

```
✅ make fmt --all -- --check
✅ cargo clippy --all-targets -- -D warnings
✅ make validate (0 CA violations)
✅ make test (70+ tests passing, 85%+ coverage)
✅ Performance benchmarks documented
✅ All 35 Week 1 issues closed/verified
✅ Commit history clean + messages clear
✅ Code review approved + merged
✅ v0.4.0 Week 1 checkpoint validated
```

---

## 📞 Support

**For Questions During Execution**:
1. Reference `ARCHITECTURE_ALIGNMENT_ADR-041-046.md` (Parts 1-5 guide every decision)
2. Check Beads issue descriptions (acceptance criteria explicit)
3. Follow MCB patterns from `CLAUDE.md` + existing crates
4. Consult `docs/adr/` for architectural decisions
5. Use `make validate` to catch CA violations early

**For Scope Creep / Changes**:
1. Update the relevant Beads issue
2. Document rationale in commit message
3. Flag in weekly checkpoint
4. Do NOT change trait signatures without approval

---

## 🎉 Final Status

**Workflow Mode Analysis**: ✅ COMPLETE  
**ADRs 041-046**: ✅ COMMITTED  
**Architecture Alignment**: ✅ VALIDATED (92% confidence)  
**Beads Issues**: ✅ CREATED (35 issues, dependencies linked)  
**Execution Path**: ✅ CLEAR (Week 1: 6 days, 18-25 hours)  

**READY FOR PHASE 9 EXECUTION**

Start date: **February 17, 2026**  
Team: [Assign in checkpoint meeting]  
Expected completion: **March 16, 2026**

---

**Document Author**: Sisyphus (Workflow Mode)  
**Confidence**: 92% (architecture + execution plan)  
**Next Owner**: Phase 9 Execution Team
