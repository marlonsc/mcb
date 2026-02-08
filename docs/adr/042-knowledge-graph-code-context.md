---
adr: 42
title: Knowledge Graph for Code Context and Relationships
status: PROPOSED
created: 
updated: 2026-02-05
related: []
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

## ADR-042: Knowledge Graph for Code Context and Relationships

**Status**: Proposed  
**Date**: 2026-02-05  
**Deciders**: MCB Architecture Team  
**Related**: ADR-041 (Context System), ADR-043 (Search)  
**Successor**: ADR-043 (Hybrid Search uses this graph)

## Context

ADR-041 defines a 5-layer context system. Layer 3 is the **Knowledge Graph** that models code structure, relationships, and dependencies. This ADR specifies:

1.  What relationships to represent
2.  How to extract them efficiently (tree-sitter-graph)
3.  How to store them (petgraph DAG + slotmap)
4.  How to query them (graph traversal for context reasoning)

## Decision

### 1. Graph Structure: Multi-Graph with Multiple Edge Types

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

**Rationale**:

-   **Multiple edge types** enable different reasoning (calls vs data flows vs imports)
-   **Directed Graph** structure for relationship modeling
-   **Explicit Index** for fast node lookup by ID
-   **Simplified Node Types** focused on primary code entities

### 2. Extraction Strategy: tree-sitter-graph + Manual Walks

```rust
pub trait SemanticExtractor: Send + Sync {
    async fn extract_graph(
        &self,
        code: &str,
        language: Language,
    ) -> Result<CodeGraph>;
}

// Implementation via tree-sitter-graph
pub struct TreeSitterGraphExtractor {
    cache: Moka<String, Arc<CodeGraph>>,  // File hash → graph
}

impl SemanticExtractor for TreeSitterGraphExtractor {
    async fn extract_graph(&self, code: &str, language: Language) -> Result<CodeGraph> {
        // 1. Parse AST via tree-sitter
        let tree = parser.parse(code, None)?;
        
        // 2. Run tree-sitter-graph DSL to extract relationships
        let query = load_query(language)?;  // Language-specific TSG rules
        let mut extractor = QueryExtractor::new(&tree);
        let edges = extractor.run_query(query)?;
        
        // 3. Build petgraph DAG
        let mut dag = petgraph::dag::Dag::new();
        let mut node_map = HashMap::new();
        
        // Add nodes (functions, structs, variables)
        for (name, range) in &function_defs {
            let id = dag.add_node(CodeNode { 
                name: name.clone(), 
                range: *range,
                // ... other fields
            });
            node_map.insert(name.clone(), id);
        }
        
        // Add edges from tree-sitter-graph extraction
        for (from, to, rel_type) in edges {
            let from_id = node_map.get(from)?;
            let to_id = node_map.get(to)?;
            dag.add_edge(*from_id, *to_id, rel_type)?;
        }
        
        Ok(CodeGraph { dag, timestamp: SystemTime::now(), ... })
    }
}
```

**Rationale**:

-   **tree-sitter-graph** is a DSL for extracting semantic relationships (maintained by GitHub)
-   **Caching** by file hash avoids re-extraction on identical files
-   **Incremental updates** on file change: re-extract changed file + 1-hop neighbors
-   **No expensive ML**: Pure AST analysis at <1ms per file

### 2.5. SemanticExtractorProvider Port Trait

The semantic extraction capability is exposed as a **port trait** for provider abstraction:

```rust
// mcb-domain/src/ports/providers/semantic_extractor.rs

use async_trait::async_trait;
use crate::entities::code::{Symbol, Relationship};
use crate::errors::ContextError;

/// Port trait for semantic code extraction.
/// 
/// Implementations extract symbols and relationships from source code.
/// Registered via linkme distributed slice for compile-time discovery.
#[async_trait]
pub trait SemanticExtractorProvider: Send + Sync {
    /// Extract symbols (functions, structs, variables, etc.) from code.
    /// 
    /// # Arguments
    /// * `content` - Source code text
    /// * `language` - Programming language (e.g., "rust", "python", "typescript")
    /// 
    /// # Returns
    /// Vector of extracted symbols with locations and metadata
    async fn extract_symbols(
        &self,
        content: &str,
        language: &str,
    ) -> Result<Vec<Symbol>, ContextError>;
    
    /// Extract relationships (calls, imports, type refs, data flows) from code.
    /// 
    /// # Arguments
    /// * `content` - Source code text
    /// * `language` - Programming language
    /// 
    /// # Returns
    /// Vector of relationships between symbols
    async fn extract_relationships(
        &self,
        content: &str,
        language: &str,
    ) -> Result<Vec<Relationship>, ContextError>;
}

/// Provider registry for semantic extractors (linkme distributed slice)
#[linkme::distributed_slice]
pub static SEMANTIC_EXTRACTOR_PROVIDERS: [&'static dyn SemanticExtractorProvider] = [..];
```

**Implementation**: Tree-sitter-based implementation in `mcb-providers/src/context/tree_sitter_semantic_extractor.rs`:

```rust
// mcb-providers/src/context/tree_sitter_semantic_extractor.rs

use mcb_domain::ports::providers::semantic_extractor::SemanticExtractorProvider;
use async_trait::async_trait;

pub struct TreeSitterSemanticExtractor {
    // Parser cache, language configs, etc.
}

#[async_trait]
impl SemanticExtractorProvider for TreeSitterSemanticExtractor {
    async fn extract_symbols(
        &self,
        content: &str,
        language: &str,
    ) -> Result<Vec<Symbol>, ContextError> {
        // 1. Parse AST via tree-sitter
        // 2. Walk AST to extract function/struct/variable definitions
        // 3. Return typed Symbol entities
        unimplemented!("tree-sitter extraction")
    }
    
    async fn extract_relationships(
        &self,
        content: &str,
        language: &str,
    ) -> Result<Vec<Relationship>, ContextError> {
        // 1. Parse AST via tree-sitter
        // 2. Run tree-sitter-graph DSL to extract relationships
        // 3. Return typed Relationship entities (calls, imports, type refs, etc.)
        unimplemented!("tree-sitter-graph extraction")
    }
}

// Register provider via linkme
#[linkme::distributed_slice(SEMANTIC_EXTRACTOR_PROVIDERS)]
static TREE_SITTER_EXTRACTOR: &dyn SemanticExtractorProvider = &TreeSitterSemanticExtractor::new();
```

**Rationale**:

-   **Port abstraction**: Enables multiple extraction backends (tree-sitter, custom rules, ML-based in v0.5.0)
-   **Linkme registration**: Compile-time provider discovery, zero runtime overhead
-   **Async-first**: Aligns with MCB's async architecture (ADR-002)
-   **Clean Architecture**: Port trait in domain, implementation in providers (ADR-013)

### 3. Storage: petgraph DAG + slotmap Arena

```rust
use petgraph::dag::Dag;
use slotmap::SlotMap;

// In-memory graph (serializable to JSON via serde)
pub struct CodeGraph {
    dag: Dag<CodeNode, RelationshipType>,
    node_arena: SlotMap<NodeId, CodeNode>,  // O(1) lookups
    timestamp: SystemTime,
    file_hash: String,
}

// Persistence: SQLite via JSON serialization
pub struct SqliteGraphStore {
    db: Connection,
}

impl GraphPersistence for SqliteGraphStore {
    async fn save(&self, graph: &CodeGraph) -> Result<GraphId> {
        let json = serde_json::to_string(graph)?;
        let id = uuid::Uuid::new_v4().to_string();
        
        self.db.execute(
            "INSERT INTO code_graphs (id, graph_json, file_hash, timestamp) VALUES (?, ?, ?, ?)",
            params![&id, &json, &graph.file_hash, &graph.timestamp],
        )?;
        
        Ok(GraphId(id))
    }
    
    async fn load(&self, id: &GraphId) -> Result<CodeGraph> {
        let json: String = self.db.query_row(
            "SELECT graph_json FROM code_graphs WHERE id = ?",
            [&id.0],
            |row| row.get(0),
        )?;
        
        Ok(serde_json::from_str(&json)?)
    }
}
```

**Rationale**:

-   **petgraph**: Mature, well-tested graph library with algorithms (DFS, shortest path, etc.)
-   **slotmap**: Generational indices prevent use-after-free bugs
-   **JSON serialization**: Human-readable, easy debugging, Serde integration
-   **SQLite storage**: Persistent, queryable, no external service

### 4. Traversal API: Graph-Aware Context Reasoning

```rust
pub trait ContextGraphTraversal: Send + Sync {
    // Get all functions that call a specific function
    async fn callers(&self, node_id: NodeId) -> Result<Vec<NodeId>>;
    
    // Get all functions called by a specific function (1-hop)
    async fn callees(&self, node_id: NodeId) -> Result<Vec<NodeId>>;
    
    // Get all reachable code (transitive closure)
    async fn reachable_from(&self, node_id: NodeId, max_depth: u32) -> Result<Vec<NodeId>>;
    
    // Get minimal set of changes affecting a module (impact analysis)
    async fn reverse_dependencies(&self, node_id: NodeId) -> Result<Vec<(NodeId, u32)>>;  // depth
    
    // Get contextual code related to a query (for search expansion)
    async fn related_code(&self, node_id: NodeId, radius: u32) -> Result<Vec<NodeId>>;
}

impl ContextGraphTraversal for PetgraphCodeGraph {
    async fn callers(&self, node_id: NodeId) -> Result<Vec<NodeId>> {
        // Use petgraph's incoming_edges iterator
        Ok(self.dag.neighbors_directed(node_id, petgraph::Direction::Incoming)
            .collect())
    }
    
    async fn reachable_from(&self, node_id: NodeId, max_depth: u32) -> Result<Vec<NodeId>> {
        // BFS with depth tracking
        let mut visited = HashSet::new();
        let mut queue = VecDeque::from([(node_id, 0)]);
        
        while let Some((current, depth)) = queue.pop_front() {
            if depth > max_depth || visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            
            for neighbor in self.dag.neighbors(current) {
                queue.push_back((neighbor, depth + 1));
            }
        }
        
        Ok(visited.into_iter().collect())
    }
}
```

**Rationale**:

-   **Traversal enables reasoning**: Search queries can expand to related code
-   **Impact analysis**: Determine what breaks when a module changes
-   **Contextual expansion**: Find similar code patterns across codebase
-   **Depth limits** prevent explosion (< 100 results for depth 2-3)

## Integration with ADR-041, ADR-043, & ADR-044

**ADR-041 (Context System)**:

-   ContextSnapshot.graph contains CodeGraph
-   Freshness propagates: stale graph → demote search results from old code

**ADR-043 (Hybrid Search)**:

-   Graph traversal enables "find related code" queries
-   Graph ranking signal: higher rank if reachable from search Result
-   Example: "Search for auth handler" → find callers + data flows + tests

**ADR-044 (Lightweight Discovery Models)**:

-   AST-based routing (Stage 1) uses CodeGraph node types and structure
-   Graph metrics (cyclomatic complexity, line count) inform task-specific scoring
-   Example: Bug fix routing prioritizes error handling nodes extracted by SemanticExtractorProvider

## Incremental Updates (Optimization)

```rust
pub struct IncrementalGraphBuilder {
    prev_graph: Arc<CodeGraph>,
    changed_files: Vec<String>,
}

impl IncrementalGraphBuilder {
    pub async fn build(&self) -> Result<CodeGraph> {
        // 1. Extract graphs for changed files only
        let changed_graphs = futures::join_all(
            self.changed_files.iter()
                .map(|f| extractor.extract_graph(f))
        ).await?;
        
        // 2. Identify affected nodes (1-hop from changed functions)
        let affected_nodes = changed_graphs.iter()
            .flat_map(|g| self.prev_graph.dag.neighbors(g.root_node))
            .collect::<HashSet<_>>();
        
        // 3. Recompute only affected subgraph
        let mut new_graph = self.prev_graph.clone();
        for node_id in affected_nodes {
            new_graph.recompute_metrics(node_id)?;
        }
        
        Ok(new_graph)
    }
}
```

## Testing

-   **Unit tests** (8): Node creation, edge addition, DAG cycle detection
-   **Extraction tests** (12): Tree-sitter-graph on 5+ languages, symbol accuracy
-   **Traversal tests** (10): Reachability, impact analysis, depth limits
-   **Incremental tests** (5): Delta computation, correctness vs full rebuild

**Target**: 35+ tests, 90%+ coverage on graph logic

## Success Criteria

-   ✅ Extract relationships from code <1ms per file
-   ✅ Support 14 languages (via tree-sitter-graph + manual rules)
-   ✅ Traversal algorithms complete in <100ms for 10k-node graphs
-   ✅ Incremental updates 10x faster than full rebuild
-   ✅ No circular dependencies in DAG (enforced at type level)

## Architecture Corrections

### Correction 1: SemanticExtractorProvider Port Trait (2026-02-06)

**Issue**: ADR-042 discussed semantic extraction but did not define the port trait interface.

**Resolution**:

-   **Added**: `SemanticExtractorProvider` trait in `mcb-domain/src/ports/providers/semantic_extractor.rs`
-   **Methods**: `extract_symbols()` and `extract_relationships()` for AST-based extraction
-   **Registration**: Linkme distributed slice for compile-time provider discovery
-   **Implementation**: Tree-sitter-based extractor in `mcb-providers/src/context/tree_sitter_semantic_extractor.rs`

**Rationale**: Port traits enable provider abstraction (ADR-013). Multiple extraction backends can be swapped without changing consumer code.

---

**Depends on**: ADR-041 (context architecture)  
**Feeds**: ADR-043 (hybrid search uses graph)
