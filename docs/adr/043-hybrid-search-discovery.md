---
adr: 43
title: Hybrid Search & Discovery for Context
status: PROPOSED
created:
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

# ADR-043: Hybrid Search & Discovery for Context

**Status**: Proposed
**Date**: 2026-02-05
**Deciders**: MCB Architecture Team
**Related**: ADR-041 (Context), ADR-042 (Knowledge Graph), ADR-046 (Policy gating)
**Predecessor**: ADR-042 (uses graph)

## Context

ADR-042 builds a knowledge graph of code relationships. ADR-043 specifies the**search engine** that queries this graph alongside:

- Full-text search (tantivy BM25 on code content)
- Vector embeddings (semantic similarity via existing MCB vector stores)
- Graph traversal (related code discovery)
- Freshness weighting (prefer recent context)

into a unified**hybrid search** that returns ranked results with explicit provenance.

## Decision

### 1. Hybrid Search Architecture: Compose Multiple Signals

```text
User Query
    ↓
┌─────────────────┬──────────────────┬───────────────────┐
│ FTS Search      │ Semantic Search  │ Graph Traversal   │
│ (tantivy BM25)  │ (vector embedding)│ (petgraph DFS)    │
└─────────────────┴──────────────────┴───────────────────┘
    ↓ results        ↓ results           ↓ results
┌──────────────────────────────────────────────────────┐
│ Reciprocal Rank Fusion (RRF)                        │
│ Combine: BM25 + Semantic + Graph + Freshness       │
└──────────────────────────────────────────────────────┘
    ↓
Ranked Results (top-k sorted by final score)
```

### 2. Search Query & Result Models

```rust
pub struct ContextSearchQuery {
    pub text: String,                          // Natural language or code snippet
    pub embedding: Option<Vec<f32>>,           // Pre-computed if available
    pub graph_expansion: Option<GraphStrategy>, // How to explore graph
    pub scope_filter: ScopeFilter,             // Project/crate/module level
    pub freshness_floor: ContextFreshness,    // Minimum acceptable freshness
    pub k: u32,                                // Top-k results (default 20)
}

pub enum GraphStrategy {
    None,
    Callers { depth: u32 },           // Find code that calls result
    Dependencies { depth: u32 },      // Find dependencies of result
    Related { radius: u32 },          // Find related code (BFS)
}

#[derive(Clone, Debug)]
pub struct ContextSearchResult {
    pub node: CodeNode,
    pub score: f32,                    // Final normalized score (0.0-1.0)
    pub bm25_rank: f32,               // Rank from FTS (0.0-1.0)
    pub semantic_rank: f32,           // Rank from embeddings
    pub graph_rank: f32,              // Rank from graph expansion
    pub freshness: ContextFreshness,  // Stale/Fresh/etc
    pub freshness_penalty: f32,       // 0.7 for Stale, 1.0 for Fresh
    pub provenance: SearchProvenance, // Which signal(s) matched
}

pub struct SearchProvenance {
    pub matched_bm25: bool,
    pub matched_semantic: bool,
    pub matched_graph_expansion: bool,
    pub distance_in_graph: Option<u32>,
}
```

### 3. Implementation: RRF Fusion Algorithm

**Application-Layer Service** (`mcb-application/src/use_cases/context_search.rs`):

```rust
/// ContextSearchService: Application-layer service that COMPOSES multiple port traits
/// from mcb-domain to provide unified hybrid search.
///
/// This is NOT a provider trait — it's a concrete service struct that orchestrates:
/// - FullTextSearchProvider (port trait from mcb-domain)
/// - VectorStoreProvider (port trait from mcb-domain)
/// - ContextGraphTraversal (port trait from mcb-domain)
pub struct ContextSearchService {
    fts_provider: Arc<dyn FullTextSearchProvider>,
    vector_store: Arc<dyn VectorStoreProvider>,
    graph_traversal: Arc<dyn ContextGraphTraversal>,
    k: u32,
    rrf_k: f32,  // RRF constant (typically 60)
}

impl ContextSearchService {
    pub async fn search(
        &self,
        query: &ContextSearchQuery,
    ) -> Result<Vec<ContextSearchResult>> {
        // Step 1: Full-text search via FullTextSearchProvider port trait
        let fts_options = FtsOptions { k: query.k as u64, ..Default::default() };
        let fts_results = self.fts_provider.search(&query.text, fts_options).await?;
        let fts_ranked = Self::normalize_ranks(&fts_results, "bm25");

        // Step 2: Semantic search via VectorStoreProvider port trait
        let embedding = query.embedding.clone()
            .or_else(|| self.encode_query(&query.text).ok());
        let semantic_results = if let Some(emb) = embedding {
            self.vector_store.search(&emb, query.k as usize).await?
        } else {
            vec![]
        };
        let semantic_ranked = Self::normalize_ranks(&semantic_results, "semantic");

        // Step 3: Graph traversal via ContextGraphTraversal port trait
        let graph_results = match &query.graph_expansion {
            GraphStrategy::None => vec![],
            GraphStrategy::Related { radius } => {
                self.graph_traversal.related_code(&fts_results, *radius).await?
            },
            _ => vec![],
        };
        let graph_ranked = Self::normalize_ranks(&graph_results, "graph");

        // Step 4: Reciprocal Rank Fusion
        let fused = Self::rrf_fusion(vec![
            ("bm25", fts_ranked, 1.0),
            ("semantic", semantic_ranked, 0.8),
            ("graph", graph_ranked, 0.6),
        ])?;

        // Step 5: Apply freshness weighting
        let fresh_weighted = Self::apply_freshness_penalty(&fused, &self.graph);

        // Step 6: Filter by scope + freshness floor
        let filtered = fresh_weighted.iter()
            .filter(|r| {
                query.scope_filter.matches(&r.node) &&
                self.freshness_sufficient(&r.freshness, &query.freshness_floor)
            })
            .take(query.k as usize)
            .cloned()
            .collect();

        Ok(filtered)
    }

    fn rrf_fusion(
        rankings: Vec<(&str, Vec<(NodeId, f32)>, f32)>,
    ) -> Result<Vec<(NodeId, f32)>> {
        // RRF formula: score = sum(1 / (k + rank))
        const RRF_K: f32 = 60.0;
        let mut scores: HashMap<NodeId, f32> = HashMap::new();

        for (signal_name, ranked_results, weight) in rankings {
            for (rank, (node_id, _)) in ranked_results.iter().enumerate() {
                let rrf_score = (1.0 / (RRF_K + (rank as f32))) * weight;
                *scores.entry(*node_id).or_insert(0.0) += rrf_score;
            }
        }

        let mut combined: Vec<_> = scores.into_iter().collect();
        combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(combined)
    }

    fn apply_freshness_penalty(
        results: &[(NodeId, f32)],
        graph: &CodeGraph,
    ) -> Vec<ContextSearchResult> {
        results.iter().map(|(node_id, score)| {
            let node = graph.dag.node_weight(*node_id).unwrap();
            let (freshness, penalty) = match node.freshness {
                ContextFreshness::Fresh => (ContextFreshness::Fresh, 1.0),
                ContextFreshness::Acceptable => (ContextFreshness::Acceptable, 0.9),
                ContextFreshness::Stale => (ContextFreshness::Stale, 0.7),
                ContextFreshness::StaleWithRisk => (ContextFreshness::StaleWithRisk, 0.5),
            };

            ContextSearchResult {
                node: node.clone(),
                score: score * penalty,
                freshness,
                freshness_penalty: penalty,
                // ... other fields
            }
        }).collect()
    }
}

// Implementation details delegated to port trait providers:
// - FullTextSearchProvider handles tantivy indexing and BM25 ranking
// - VectorStoreProvider handles semantic search and similarity computation
// - ContextGraphTraversal handles graph traversal and distance computation
```

### 3.1 Port Traits (mcb-domain)

**FullTextSearchProvider** (`mcb-domain/src/ports/providers/full_text_search.rs`):

```rust
use async_trait::async_trait;
use thiserror::Error;
use linkme::distributed_slice;

#[derive(Debug, Clone)]
pub struct FtsOptions {
    pub k: u64,                    // Top-k results
    pub min_score: Option<f32>,    // Minimum score threshold
    pub field_weights: Option<Vec<(String, f32)>>, // Field-specific weights
}

#[derive(Debug, Clone)]
pub struct FtsResult {
    pub id: String,                // Node ID
    pub score: f32,                // BM25 score
    pub matched_fields: Vec<String>, // Which fields matched
}

#[derive(Debug, Error)]
pub enum FtsError {
    #[error("FTS search failed: {0}")]
    SearchFailed(String),
    #[error("Index error: {0}")]
    IndexError(String),
}

pub type FtsResult<T> = Result<T, FtsError>;

/// Port trait for full-text search providers
/// Implementations: tantivy (default in mcb-providers), elasticsearch, meilisearch
#[async_trait]
pub trait FullTextSearchProvider: Send + Sync {
    /// Search code content using BM25 ranking
    async fn search(&self, query: &str, options: FtsOptions) -> FtsResult<Vec<FtsResult>>;

    /// Index a document
    async fn index(&self, id: &str, content: &str) -> FtsResult<()>;

    /// Clear all indexed documents
    async fn clear(&self) -> FtsResult<()>;
}

/// Linkme distributed slice for provider registration
#[distributed_slice]
pub static FULL_TEXT_SEARCH_PROVIDERS: [&'static dyn FullTextSearchProvider] = [..];
```

### 4. Integration with Memory System

MCB v0.2.0 already has memory system with FTS5 + hybrid search. v0.4.0 extends this:

```rust
pub struct UnifiedSearchEngine {
    hybrid_search: Arc<HybridSearchEngine>,     // Code graph search
    memory_search: Arc<dyn MemorySearchProvider>, // Memory FTS + vector
}

impl UnifiedSearchEngine {
    pub async fn search_all(
        &self,
        query: &str,
        context: &ContextSnapshot,
    ) -> Result<UnifiedSearchResults> {
        // Search both code AND memory
        let code_results = self.hybrid_search.search(query).await?;
        let memory_results = self.memory_search.search(query).await?;

        // Merge and rank
        UnifiedSearchResults {
            code: code_results,
            memory: memory_results,
            hybrid_rank: Self::merge_rankings(code_results, memory_results)?,
        }
    }
}
```

## Architecture Corrections

### Correction 3 (mcb-ulf): Service Layer Placement

**Issue**: HybridSearchEngine was incorrectly shown as a provider trait.

**Fix**: Renamed to `ContextSearchService` and clarified as an**application-layer concrete service** (not a trait) that:

- Lives in `mcb-application/src/use_cases/context_search.rs`
- **COMPOSES** three port traits from `mcb-domain`:
- `FullTextSearchProvider` (BM25 full-text search)
- `VectorStoreProvider` (semantic similarity)
- `ContextGraphTraversal` (graph-based discovery)
- Implements the RRF fusion algorithm to combine signals

**Rationale**: Application services orchestrate port traits; they are not themselves ports. This maintains Clean Architecture's dependency inversion principle.

### Correction 4 (mcb-jq3): Missing Port Trait Definition

**Issue**: `FullTextSearchProvider` port trait was referenced but not defined in the ADR.

**Fix**: Added complete port trait definition in section 3.1:

- **Location**: `mcb-domain/src/ports/providers/full_text_search.rs`
- **Trait Methods**:
- `async fn search(&self, query: &str, options: FtsOptions) -> FtsResult<Vec<FtsResult>>`
- `async fn index(&self, id: &str, content: &str) -> FtsResult<()>`
- `async fn clear(&self) -> FtsResult<()>`
- **Registration**: Uses `#[linkme::distributed_slice(FULL_TEXT_SEARCH_PROVIDERS)]` for compile-time provider discovery
- **Default Implementation**: tantivy (BM25) in `mcb-providers`
- **Error Handling**: Custom `FtsError` enum with `thiserror`

**Rationale**: Port traits must be explicitly defined in the domain layer. Linkme registration enables zero-runtime-overhead provider discovery.

## Testing

- **FTS tests** (5): tantivy indexing, query parsing, rank accuracy
- **Semantic tests** (5): Vector search, similarity computation
- **Graph traversal tests** (5): BFS/DFS, distance computation
- **RRF fusion tests** (8): Weight combinations, rank preservation
- **Freshness tests** (3): Penalty application, floor filtering
- **E2E search tests** (10): Real queries, Result quality

**Target**: 36+ tests, 85%+ coverage on search engine

### Success Criteria

- ✅ Search completes in <500ms for 100k nodes
- ✅ RRF fusion balanced (no single signal dominates)
- ✅ Top-3 results highly relevant (manual review)
- ✅ Freshness penalties working (stale results demoted)
- ✅ Graph expansion discovering related code (validation query)

---

**Depends on**: ADR-041 (context), ADR-042 (graph)
**Feeds**: ADR-041 (search service), ADR-046 (policy gating)
