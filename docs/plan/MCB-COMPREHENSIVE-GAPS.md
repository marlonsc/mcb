# MCB Comprehensive Gap Analysis

**Date**: 2026-02-08  
**Analyzed By**: Sisyphus + 3 Explore Agents  
**Scope**: Domain Layer, MCP Handlers, Providers vs v0.3.0/v0.4.0 Roadmap

---

## Executive Summary

MCB v0.2.0 provides functional code search and basic MCP tools, but **significant gaps exist** blocking v0.3.0 (Workflow System) and v0.4.0 (Integrated Context). This analysis identifies **18 critical gaps** across 3 layers.

---

## Gap Matrix

| Layer | Total Gaps | Critical (P0) | High (P1) | Medium (P2) |
|-------|------------|---------------|-----------|-------------|
| Domain | 7 | 3 | 3 | 1 |
| Handlers | 4 | 2 | 1 | 1 |
| Providers | 5 | 2 | 2 | 1 |
| **Total** | **16** | **7** | **6** | **3** |

---

## Layer 1: Domain Entities & Ports

### GAP-D1: WorkflowState Outdated (P1)
**File**: `crates/mcb-domain/src/entities/workflow.rs`  
**ADR**: ADR-034 (FSM)  
**Status**: EXISTS but OUTDATED

**Issue**: Current implementation has 8-state model. ADR-034 (updated 2026-02-06) mandates 12-state model.

**Missing States**:
- `Suspended` - Workflow paused by policy
- `Timeout` - Workflow exceeded time limit
- `Cancelled` - Workflow cancelled by user
- `Abandoned` - Workflow abandoned (no resume)

---

### GAP-D2: CodeGraph Entity Missing (P0)
**File**: `crates/mcb-domain/src/entities/code_graph.rs` — **MISSING**  
**ADR**: ADR-042 (Knowledge Graph)

**Required**:
```rust
pub struct CodeGraph {
    graph: DiGraph<CodeNode, CodeEdge>,
    index: HashMap<NodeId, NodeIndex>,
}

pub enum CodeNode {
    Function { name: String, file: PathBuf, line: u32 },
    Class { name: String, file: PathBuf, line: u32 },
    Module { name: String, path: PathBuf },
}

pub enum CodeEdge {
    Calls,
    Imports,
    Extends,
    Implements,
    Contains,
}
```

---

### GAP-D3: ContextSnapshot Entity Missing (P0)
**File**: `crates/mcb-domain/src/entities/context_snapshot.rs` — **MISSING**  
**ADR**: ADR-045 (Context Versioning)

**Required**: Immutable state capture for time-travel queries.

---

### GAP-D4: WorkflowEngine Port Missing (P0)
**File**: `crates/mcb-domain/src/ports/providers/workflow.rs` — **MISSING**  
**ADR**: ADR-034

**Required**: Trait for workflow state machine operations.

---

### GAP-D5: SemanticExtractorProvider Missing (P1)
**File**: `crates/mcb-domain/src/ports/providers/semantic_extractor.rs` — **MISSING**  
**ADR**: ADR-042

**Required**: Trait for AST relationship extraction.

---

### GAP-D6: ContextScout Service Missing (P1)
**File**: `crates/mcb-domain/src/ports/services/context_scout.rs` — **MISSING**  
**ADR**: ADR-035

**Required**: Service for context gathering and search.

---

### GAP-D7: PolicyEngine Service Missing (P2)
**File**: `crates/mcb-domain/src/ports/services/policy_engine.rs` — **MISSING**  
**ADR**: ADR-036

**Required**: Service for workflow validation and enforcement.

---

## Layer 2: MCP Handlers

### GAP-H1: Project Handler Stubbed (P0)
**File**: `crates/mcb-server/src/handlers/project.rs`  
**Status**: ENTIRELY STUBBED

**Current**: Returns `"Project workflow not yet implemented"`  
**Required**: Connect to ProjectService, implement CRUD operations.

---

### GAP-H2: Memory ErrorPattern Not Implemented (P0)
**File**: `crates/mcb-server/src/handlers/memory/handler.rs` (lines 60, 78)  
**Status**: PARTIAL

**Current**: Returns `"Error pattern memory is not implemented yet"`  
**Required**: Implement ErrorPattern storage and retrieval.

---

### GAP-H3: VCS Repository Discovery Broken (P1)
**File**: `crates/mcb-server/src/handlers/vcs/list_repos.rs`  
**Status**: PARTIAL

**Current**: Returns Milvus collection names, not actual git repositories.  
**Required**: Use `VcsProvider::list_repositories` for true discovery.

---

### GAP-H4: Context Search Handler Missing (P2)
**File**: `crates/mcb-server/src/handlers/search.rs`  
**Status**: SearchResource enum missing Context variant

**Required**: Add `Context` to SearchResource, inject ContextServiceInterface.

---

## Layer 3: Providers

### GAP-P1: TantivyBM25 Not Implemented (P0)
**File**: `crates/mcb-providers/src/hybrid_search/bm25.rs`  
**Status**: Custom HashMap-based scorer, NO INVERTED INDEX

**Current**: In-memory tokenization and scoring  
**Required**: Tantivy-backed inverted index for production-scale search

**Impact**: Current BM25 won't scale beyond ~10k files.

---

### GAP-P2: TreeSitterExtractor Not Centralized (P0)
**File**: Various language processors  
**Status**: Extraction logic scattered

**Current**: Each language processor has its own extraction rules  
**Required**: Centralized `TreeSitterExtractor` service

**Impact**: Duplicated code, inconsistent extraction across languages.

---

### GAP-P3: Milvus Stats Incomplete (P1)
**File**: `crates/mcb-providers/src/vector_store/milvus.rs` (lines 91-115)  
**Status**: PARTIAL

**Current**: Hardcodes `status: "active"`, only parses `row_count`  
**Required**: Full health metrics from Milvus

---

### GAP-P4: Pinecone Listing Workaround (P1)
**File**: `crates/mcb-providers/src/vector_store/pinecone.rs` (lines 358-368)  
**Status**: WORKAROUND

**Current**: Uses zero-vector search to list vectors  
**Required**: Proper list API implementation

---

### GAP-P5: Generic Error Handling (P2)
**File**: Multiple providers  
**Status**: SUBOPTIMAL

**Current**: Wraps errors into `Error::internal` or `Error::vector_db`  
**Required**: Preserve specific error context from underlying drivers

---

## Priority Implementation Order

### Phase 1: Unblock OpenCode Integration (Week 1)
1. **GAP-H2**: Fix Memory ErrorPattern → Enables observation storage
2. **GAP-H1**: Implement Project Handler → Enables project-scoped context

### Phase 2: Complete v0.3.0 Core (Week 2-3)
3. **GAP-D1**: Update WorkflowState to 12-state model
4. **GAP-D4**: Implement WorkflowEngine port
5. **GAP-D6**: Implement ContextScout service
6. **GAP-H3**: Fix VCS repository discovery

### Phase 3: Enable v0.4.0 Features (Week 4-6)
7. **GAP-D2**: Implement CodeGraph entity
8. **GAP-D3**: Implement ContextSnapshot entity
9. **GAP-P1**: Implement TantivyBM25
10. **GAP-P2**: Centralize TreeSitterExtractor
11. **GAP-D5**: Implement SemanticExtractorProvider

### Phase 4: Polish (Week 7+)
12. **GAP-D7**: Implement PolicyEngine service
13. **GAP-H4**: Add Context search handler
14. **GAP-P3**: Complete Milvus stats
15. **GAP-P4**: Fix Pinecone listing
16. **GAP-P5**: Improve error handling

---

## Beads Issues to Create

```bash
cd ~/mcb

# P0 - Critical
bd create --title "GAP-D2: Implement CodeGraph entity (ADR-042)" --type feature --priority 0
bd create --title "GAP-D3: Implement ContextSnapshot entity (ADR-045)" --type feature --priority 0
bd create --title "GAP-D4: Implement WorkflowEngine port (ADR-034)" --type feature --priority 0
bd create --title "GAP-P1: Implement TantivyBM25 for production-scale search" --type feature --priority 0
bd create --title "GAP-P2: Centralize TreeSitterExtractor service" --type feature --priority 0

# P1 - High
bd create --title "GAP-D1: Update WorkflowState to 12-state model" --type feature --priority 1
bd create --title "GAP-D5: Implement SemanticExtractorProvider port" --type feature --priority 1
bd create --title "GAP-D6: Implement ContextScout service (ADR-035)" --type feature --priority 1
bd create --title "GAP-H3: Fix VCS list_repositories to use VcsProvider" --type bug --priority 1
bd create --title "GAP-P3: Complete Milvus health metrics in get_stats" --type feature --priority 1
bd create --title "GAP-P4: Fix Pinecone vector listing workaround" --type bug --priority 1

# P2 - Medium
bd create --title "GAP-D7: Implement PolicyEngine service (ADR-036)" --type feature --priority 2
bd create --title "GAP-H4: Add Context variant to SearchResource enum" --type feature --priority 2
bd create --title "GAP-P5: Improve error handling - preserve driver context" --type feature --priority 2
```

---

## Cross-References

- **Initial Gaps Report**: `docs/plan/MCB-GAPS-REPORT.md`
- **Integration Plan**: `docs/plan/OPENCODE-INTEGRATION-PLAN.md`
- **v0.3.0 Implementation Spec**: `docs/v030-IMPLEMENTATION.md`
- **Roadmap**: `docs/developer/ROADMAP.md`
- **ADRs**: `docs/adr/034-046`
