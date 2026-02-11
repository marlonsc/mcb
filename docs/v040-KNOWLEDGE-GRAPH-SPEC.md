# MCB v0.4.0 Knowledge Graph Specification

## Overview

The Knowledge Graph is the core semantic layer of the MCB Integrated Context System. It transforms raw source code into a queryable, relationship-aware network of entities, enabling high-fidelity code reasoning and hybrid search.

## 1. Knowledge Graph Schema

### Node Types

Nodes represent semantic entities extracted from the source code.

| Node Type | Description | Attributes |
|-----------|-------------|------------|
| `Module` | A file or logical grouping of code | `path`, `is_external` |
| `Function` | A callable unit of code | `name`, `signature`, `is_async` |
| `Class` / `Struct` | A data structure or object definition | `name`, `fields`, `methods` |
| `Interface` / `Trait` | A behavioral contract | `name`, `methods` |
| `Import` | An external dependency reference | `source`, `alias` |
| `Variable` | A global or significant local state | `name`, `type_ref` |

### Edge Types

Edges define directed relationships between semantic entities.

| Edge Type | Source | Target | Description |
|-----------|--------|--------|-------------|
| `CALLS` | `Function` | `Function` | Function execution flow |
| `IMPORTS` | `Module` | `Module` | Dependency relationship |
| `EXTENDS` | `Class` | `Class` | Inheritance relationship |
| `IMPLEMENTS`| `Class` | `Interface`| Contract fulfillment |
| `CONTAINS` | `Module` | `Entity` | Ownership hierarchy |
| `TYPE_REF` | `Variable` | `Class` | Data type association |

### Rust Entity Design

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: NodeId,
    pub kind: CodeNodeKind,
    pub fqn: String, // Fully Qualified Name
    pub range: TextRange,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RelationshipType {
    Calls,
    Imports,
    Extends,
    Implements,
    Contains,
}

pub struct CodeGraph {
    pub dag: petgraph::stable_graph::StableDiGraph<CodeNode, RelationshipType>,
    pub timestamp: SystemTime,
}
```

## 2. TreeSitter Semantic Extraction

MCB uses `tree-sitter` for high-performance, language-agnostic AST parsing and relationship extraction.

### Extraction Approach

1. **AST Parsing**: Generate a concrete syntax tree using language-specific tree-sitter grammars.
2. **TSG Rules**: Use TreeSitter Graph (TSG) DSL to map AST patterns to graph nodes and edges.
3. **Symbol Resolution**: Resolve local references to Fully Qualified Names (FQNs) to link nodes across modules.
4. **Incremental Updates**: Only re-extract files with changed hashes, patching the existing graph.

### Extraction Port

```rust
#[async_trait]
pub trait SemanticExtractorProvider: Send + Sync {
    async fn extract_symbols(&self, content: &str, lang: &str) -> Result<Vec<Symbol>>;
    async fn extract_relationships(&self, content: &str, lang: &str) -> Result<Vec<Relationship>>;
}
```

## 3. RRF Hybrid Search Algorithm

Hybrid search combines multiple retrieval signals to ensure both semantic relevance and structural accuracy.

### Reciprocal Rank Fusion (RRF)

RRF merges rankings from Full-Text Search (FTS), Vector Embeddings, and Graph Traversal.

**Formula**:
$$score(d) = \sum_{r \in R} \frac{1}{k + rank(d, r)}$$
*where $k$ is a constant (default 60), and $rank(d, r)$ is the rank of document $d$ in Result set $r$.*

### Search Composition

1. **FTS (BM25)**: Lexical matching on code content and documentation.
2. **Vector (Cosine)**: Semantic similarity using code embeddings.
3. **Graph (PageRank/Traversal)**: Structural importance and proximity (e.g., "find callers of X").
4. **Freshness Weighting**: Penalty applied to stale context ($score = score \times penalty$).

## 4. Context Snapshot Design

Context snapshots provide immutable points-in-time of the system state, enabling time-travel queries and consistent reasoning.

### Snapshot Structure

```rust
pub struct ContextSnapshot {
    pub id: ContextId,
    pub timestamp: SystemTime,
    pub graph: Arc<CodeGraph>,
    pub vcs_state: VcsSnapshot, // Git branch/commit
    pub freshness: ContextFreshness,
    pub version: u64,
}
```

### Snapshotting Policy

-   **Automatic**: Triggered on git commits or significant workflow state changes.
-   **Manual**: Triggered via `context_snapshot` MCP tool.
-   **Retention**: Snapshots are persisted in SQLite with a configurable TTL (Time-To-Live) to manage storage.
