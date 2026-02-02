# Phase 06: Memory Search - Research

**Researched:** 2026-02-02
**Domain:** Search / Information Retrieval / SQLite
**Confidence:** HIGH

## Summary

Phase 6 implements "Memory Search" with two key capabilities: **Hybrid Search** (combining Semantic Vector Search with Lexical BM25 Search) and **Progressive Disclosure** (revealing information in layers to manage context window usage).

Research confirms that **SQLite FTS5** is available in our environment and is the standard solution for adding BM25 capabilities to our existing SQLite storage. We will use **Reciprocal Rank Fusion (RRF)** to combine vector and lexical results, avoiding the complex and brittle process of score normalization.

**Primary recommendation:** Implement `observations_fts` virtual table with Triggers for automatic synchronization, and update `MemoryRepository::search` to perform RRF in-memory after fetching results from both indices.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| **SQLite FTS5** | Bundled | BM25 Lexical Search | Native to SQLite, fast, low overhead |
| **mcb-providers** | Workspace | Semantic Embedding | Existing project standard for vectors |
| **Reciprocal Rank Fusion** | Custom | Result Combination | Industry standard for hybrid search (Elastic/Azure/OpenSearch) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **sqlx** | 0.8 | Database Access | Already in use, supports SQLite FTS5 raw queries |

## Architecture Patterns

### Database Schema (Hybrid Storage)

We need a dedicated Virtual Table for FTS5 that mirrors the content of the `observations` table.

```sql
-- 1. Main storage (Source of Truth) - Already exists
CREATE TABLE observations (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    ...
);

-- 2. Search Index (FTS5)
-- 'id' is UNINDEXED to allow retrieval without pollution of search space
CREATE VIRTUAL TABLE observations_fts USING fts5(content, id UNINDEXED);

-- 3. Synchronization Triggers
CREATE TRIGGER obs_ai AFTER INSERT ON observations BEGIN
  INSERT INTO observations_fts(id, content) VALUES (new.id, new.content);
END;
CREATE TRIGGER obs_ad AFTER DELETE ON observations BEGIN
  DELETE FROM observations_fts WHERE id = old.id;
END;
CREATE TRIGGER obs_au AFTER UPDATE ON observations BEGIN
  DELETE FROM observations_fts WHERE id = old.id;
  INSERT INTO observations_fts(id, content) VALUES (new.id, new.content);
END;
```

### Hybrid Search Algorithm (RRF)

**Why RRF?**
Vector search returns cosine similarity (0.0 to 1.0). FTS5 returns BM25 scores (0.0 to unbounded). Summing them (`0.8 + 15.4`) allows BM25 to dominate. Normalization is brittle. RRF works on *rank*, which is robust.

**Pattern:**
1.  **Vector Query**: Get top $N$ results from Vector Store.
2.  **Lexical Query**: Get top $N$ results from FTS5 (`SELECT id, rank FROM observations_fts ...`).
3.  **Fuse**:
    ```rust
    // score = 1.0 / (k + rank)
    // k is typically 60
    ```
4.  **Sort & Slice**: Return top $K$ combined results.

### Progressive Disclosure Workflow

To satisfy MEM-04, the system exposes memory in 3 layers:

**Layer 1: Timeline (Metadata)**
*   **Tool**: `timeline(start_date, end_date)`
*   **Returns**: List of `(id, created_at, type, tags, summary)`.
*   **Purpose**: High-level scanning of what happened. "Summary" is a truncated preview (first 100 chars) or a dedicated summary field.

**Layer 2: Search (Relevance)**
*   **Tool**: `search_memories(query)`
*   **Returns**: List of `(id, score, snippet)`.
*   **Purpose**: Finding specific items. Includes `similarity_score` to gauge relevance.

**Layer 3: Inspection (Detail)**
*   **Tool**: `get_observations(ids)`
*   **Returns**: Full `Observation` objects.
*   **Purpose**: Reading the actual content into context.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| **Result Merging** | Weighted Sum | **RRF (Rank)** | Weighted sum requires constant tuning of alpha params. RRF is "set and forget". |
| **Search Index** | Custom In-Memory Index | **SQLite FTS5** | FTS5 handles stemming, tokenization, and query syntax correctly. |
| **Syncing** | Application-side dual writes | **SQL Triggers** | Application logic *will* fail eventually. DB triggers guarantee consistency. |

## Common Pitfalls

### Pitfall 1: Ghost Rows
**What goes wrong:** Search returns IDs that don't exist in the main table.
**Why it happens:** FTS table out of sync with Main table (manual sync failed).
**How to avoid:** Use **SQL Triggers** (as defined above) so the database enforces consistency.

### Pitfall 2: The "Just Sum It" Trap
**What goes wrong:** Adding `cosine_similarity + bm25_score`.
**Why it happens:** BM25 scores are unbounded (can be 10, 50, 100). Cosine is 0-1.
**Impact:** Vector search becomes irrelevant.
**Solution:** Use Reciprocal Rank Fusion (RRF).

### Pitfall 3: N+1 Queries on Search
**What goes wrong:** Searching FTS returns IDs, then fetching full content one-by-one.
**How to avoid:**
*   **Step 1**: Search FTS -> Get IDs.
*   **Step 2**: Search Vector -> Get IDs.
*   **Step 3**: Fuse Ranks -> Get Top K IDs.
*   **Step 4**: `SELECT * FROM observations WHERE id IN (...)` (Single query).

## Code Examples

### Reciprocal Rank Fusion (Rust)

```rust
use std::collections::HashMap;

fn rrf(vector_results: Vec<String>, fts_results: Vec<String>, k: f64) -> Vec<(String, f64)> {
    let mut scores: HashMap<String, f64> = HashMap::new();

    for (rank, id) in vector_results.iter().enumerate() {
        let score = 1.0 / (k + rank as f64 + 1.0);
        *scores.entry(id.clone()).or_insert(0.0) += score;
    }

    for (rank, id) in fts_results.iter().enumerate() {
        let score = 1.0 / (k + rank as f64 + 1.0);
        *scores.entry(id.clone()).or_insert(0.0) += score;
    }

    let mut ranked: Vec<_> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}
```

### FTS5 Query (SQLx)

```rust
// Matches simple terms. For robust querying, wrap terms in double quotes 
// or sanitize input to prevent syntax errors.
let sql = "SELECT id FROM observations_fts WHERE observations_fts MATCH ? ORDER BY rank LIMIT ?";
let rows = sqlx::query(sql)
    .bind(query_text)
    .bind(limit)
    .fetch_all(&pool)
    .await?;
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| **Pure Vector** | **Hybrid (Vector + BM25)** | Fixes "keyword amnesia" where models miss exact terms (e.g., function names). |
| **Weighted Sum** | **Reciprocal Rank Fusion** | Removes need for parameter tuning; robust across different data distributions. |

## Open Questions

1.  **Snippet Generation**
    *   What we know: FTS5 has a `snippet()` function.
    *   What's unclear: Can we get snippets efficiently if we only store `content` in FTS but fetch details from Main?
    *   Recommendation: For Phase 6, just return the `content` (truncated) from Main table. FTS snippets are nice but add complexity to the fetch query.

## Sources

### Primary (HIGH confidence)
- **SQLite Official Docs**: FTS5 Extension (verified `fts5` feature is standard).
- **Elasticsearch/OpenSearch Docs**: Hybrid Search & RRF standard practices.

### Secondary (MEDIUM confidence)
- **mcb Codebase**: Confirmed `sqlx` dependency and SQLite usage. Verified FTS5 availability via test.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - SQLite FTS5 is ubiquitous.
- Architecture: HIGH - RRF is the industry standard for 2024+ hybrid search.
- Pitfalls: HIGH - Sync and scoring issues are well-documented.

**Research date:** 2026-02-02
