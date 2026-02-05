# Documentation Update Implementation Plan (v0.4.0)

**Status**: Phase A (Planning) → Phase B-E (Execution)  
**Scope**: 5 CRITICAL + 6 MEDIUM + 15+ LOWER priority files  
**Timeline**: 1-2 days (4-6 hours execution)  
**Owner**: Documentation Task Force

---

## Phase A: Map Exact Changes (COMPLETED)

### CRITICAL Files (Update FIRST)

**1. docs/developer/ROADMAP.md**

-   Current: v0.4.0 = "Enterprise Features" (multi-tenancy, RBAC, SSO)
-   Change: Rewrite to "Integrated Context System" (knowledge graph, hybrid search, versioning, policies)
-   Add: ADR-041-046 references, Phase 8-9 breakdown, technology choices
-   Lines: ~50 new lines, restructure ~30 existing
-   Effort: 1 hour

**2. docs/ADR/README.md**

-   Current: Index only lists ADR-001-031 (31 total)
-   Change: Add new section "Phase 8-9: v0.3→v0.4.0 (Workflow + Context)" with:
    -   Phase 8: ADR-034 (FSM), ADR-035 (freshness), ADR-036 (policies), ADR-037 (orchestrator)
    -   Phase 9: ADR-041 (architecture), ADR-042 (graph), ADR-043 (search), ADR-044 (models), ADR-045 (versioning), ADR-046 (integration)
-   Update: ADR count from 31 to 46 (or current count)
-   Effort: 30 minutes

**3. docs/README.md** (root level)

-   Current: Version "0.1.4", ADR count "31"
-   Change: Update to current version + "46 ADRs total"
-   Add: v0.4.0 section in features list ("Integrated Context System, Freshness Tracking, Time-Travel Queries")
-   Add: Link to "What's New in v0.4.0?" guide
-   Effort: 20 minutes

**4. docs/architecture/ARCHITECTURE.md**

-   Current: Describes v0.1.4 architecture (no v0.4.0 info)
-   Change: Add v0.4.0 section describing:
    -   5-layer integrated context system (graph, hybrid search, versioning)
    -   Relationship to ADR-034-037 (FSM gates, policy enforcement, compensation)
    -   Key components: CodeGraph (petgraph), HybridSearchEngine, ContextSnapshot
    -   Diagram: 5 layers + context system overlay
-   Add: Links to ADR-041-046
-   Effort: 1.5 hours

**5. docs/context/project-state.md**

-   Current: References Phase 6 "Memory Search"
-   Change: Update to Phase 8/9 status
-   Add: v0.4.0 context system section, timeline
-   Effort: 30 minutes

**CRITICAL SUBTOTAL**: ~3.5 hours

---

### NEW FILES (Create FIRST - No Dependencies)

**6. docs/guides/features/integrated-context.md**

-   Content: Overview of freshness, time-travel, compensation + example workflows
-   Links: ADR-034-037 (design), ADR-041-046 (implementation)
-   Effort: 1 hour

**7. docs/migration/v0.3-to-v0.4.md**

-   Content: New config sections, MCP tools, breaking changes, ADR references
-   Links: ADR-034-046
-   Effort: 45 minutes

**8. docs/architecture/clean-architecture.md**

-   Content: Why 6 layers, how they interact, extension patterns
-   Effort: 45 minutes

**9. docs/implementation/phase-9-roadmap.md**

-   Content: Week-by-week Phase 9 breakdown (4 weeks, 70+ tests)
-   Links: Beads issues, ADR-041-046
-   Effort: 1 hour

**10. docs/ADR/phase-9/README.md**

-   Content: ADR-041-046 overview, cross-references
-   Effort: 20 minutes

**NEW FILES SUBTOTAL**: ~4 hours

---

### MEDIUM Priority Files (Update AFTER CRITICAL)

**11. docs/README.md** (root)

-   Add: Version info, v0.4.0 badge, "What's New" link
-   Effort: 15 minutes

**12. docs/operations/CHANGELOG.md**

-   Add: v0.4.0 entry "Integrated Context System" (knowledge graph, hybrid search, versioning, FSM, policies)
-   Effort: 20 minutes

**13. docs/context/domain-concepts.md**

-   Update: Phase 6 → Phase 8-9, add v0.4.0 context system as core domain
-   Effort: 30 minutes

**14. docs/context/technical-patterns.md**

-   Add: Patterns for v0.4.0 (graph traversal, RRF fusion, versioning, FSM gating)
-   Effort: 45 minutes

**15. docs/context/integrations.md**

-   Add: v0.4.0 patterns (FSM gates context, policies scope, compensation rollback, events)
-   Effort: 45 minutes

**16. docs/context/conventions.md**

-   Add: v0.4.0 conventions (graph naming, traversal, staleness signals, snapshot naming)
-   Effort: 30 minutes

**MEDIUM SUBTOTAL**: ~3 hours

---

## Phase B: Create Folder Structure + New Files (PARALLEL)

```
Actions:
1. mkdir -p docs/guides/features/                  [if not exists]
2. mkdir -p docs/guides/workflows/                 [if not exists]
3. mkdir -p docs/migration/                        [if not exists]
4. mkdir -p docs/adr/phase-9/                      [if not exists]
5. mkdir -p docs/implementation/                   [if not exists]
6. Create 5 new files (items 6-10 above)
```

**Effort**: 30 minutes (mkdir + 5 new files via Write tool in parallel)

---

## Phase C: Update CRITICAL Files

**Order** (dependency order):

1.  docs/ADR/README.md (foundation for other ADR links)
2.  docs/architecture/ARCHITECTURE.md (defines v0.4.0 shape)
3.  docs/developer/ROADMAP.md (shows what's coming)
4.  docs/context/project-state.md (current state)
5.  docs/README.md (user-facing, links to above)

**Effort**: 3.5 hours (Edit tool, sequential to maintain consistency)

---

## Phase D: Cross-Link All Docs

**For each ADR (034-046)**:

-   Add footer with "Implementation References" section
-   Link to mcb-application, mcb-domain files
-   Link to Beads issues

**For each feature guide**:

-   Add footer "For Deep Dives" with ADR links

**For each architecture doc**:

-   Add "See also" links to relevant ADRs

**Effort**: 1.5 hours (Sed-like replacements or manual edits)

---

## Phase E: Validate + Commit

**Validation**:

```bash
make docs-lint              # Markdown syntax
make docs-validate          # Link checking (if available)
grep -r "ADR-0[0-9]" docs   # Verify ADR references
grep -r "v0.4" docs         # Verify version mentions
```

**Commit**:

```bash
git add docs/
git commit -m "docs: Update for v0.4.0 Integrated Context System

New files:
- docs/guides/features/integrated-context.md (freshness, time-travel, compensation)
- docs/migration/v0.3-to-v0.4.md (upgrade guide)
- docs/architecture/clean-architecture.md (design walkthrough)
- docs/implementation/phase-9-roadmap.md (4-week execution plan)
- docs/adr/phase-9/ (ADR-041-046 organization)

Updated critical files:
- docs/adr/README.md (added Phase 8-9 sections + ADR-034-046)
- docs/architecture/ARCHITECTURE.md (added v0.4.0 section with diagrams)
- docs/developer/ROADMAP.md (rewrote v0.4.0 as Integrated Context System)
- docs/context/project-state.md (updated to Phase 8-9)
- docs/README.md (version + ADR count updates)

Updated medium-priority files:
- CHANGELOG.md, domain-concepts.md, technical-patterns.md, integrations.md, conventions.md

Cross-linked all docs (ADRs ↔ guides ↔ implementation)

Related: ADR-041-046, Phase 9 execution plan, ARCHITECTURE_ALIGNMENT_ADR-041-046.md"
```

**Effort**: 20 minutes

---

## TOTAL EFFORT ESTIMATE

| Phase | Task | Hours |
|-------|------|-------|
| A | Map changes (completed) | 0.5 |
| B | Create folders + new files | 0.5 |
| C | Update CRITICAL files (5) | 3.5 |
| D | Cross-link all docs | 1.5 |
| E | Validate + commit | 0.5 |
| **TOTAL** | | **6.5 hours** |

---

## Execution Readiness Checklist

-   [ ] Phase A complete (this document)
-   [ ] All file locations identified
-   [ ] ADR-041-046 available for linking
-   [ ] Current version number confirmed
-   [ ] Team agrees on structure (Oracle recommendation)
-   [ ] Ready to execute Phase B-E

---

**Next**: Execute Phase B-E using Sisyphus (parallel implementation) + validation
