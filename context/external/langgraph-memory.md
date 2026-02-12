# LangGraph Memory: Patterns and MCB Relevance

Last updated: 2026-02-12  
Scope: memory architecture patterns from LangChain/LangGraph with applicability to MCB's observation/context system.  
Cross-reference: `context/external/openai-agents-memory.md`, `context/external/mcb-main-libraries-reference.md`.

---

## 1. Source and Evidence Basis

Primary source: LangChain/LangGraph documentation (`add-memory`, `persistence`, `long-term-memory`).

This document captures transferable design patterns, not a direct dependency integration.

---

## 2. Core Memory Patterns from LangGraph

### 2.1 Short-term vs long-term separation

Short-term memory uses thread state and checkpointers (conversation-scoped). Long-term memory uses a persistent store that survives across sessions.

### 2.2 Namespace-based partitioning

Durable memory is scoped by namespace tuples, for example `(user_id, "memories")`, enabling clean multi-tenant isolation.

### 2.3 Typed JSON records with unique keys

Memory entries are stored as JSON-like records with unique IDs, making them individually addressable and updatable.

### 2.4 Semantic retrieval

`store.search(namespace, query, limit)` enables relevance-based memory retrieval rather than brute-force replay.

### 2.5 Selective injection

Only top-ranked relevant memories are injected into prompts. This controls context size and preserves signal quality.

---

## 3. Transferable Guidance for MCB

### 3.1 Memory scoping model

MCB's observation system aligns with namespace-based partitioning:

- `project_id` as primary namespace
- `session_id` and `tags` for secondary scoping
- stored in `crates/mcb-providers/src/database/sqlite/memory_repository.rs`

### 3.2 Hybrid retrieval strategy

MCB already implements a pattern similar to LangGraph's semantic search:

- FTS for keyword-level recall
- vector search for semantic similarity
- `tokio::join!` for parallel execution of both channels

See `crates/mcb-application/src/use_cases/memory_service.rs`.

### 3.3 Concise record discipline

Keep long-term memory records compact and typed. Avoid storing large unstructured blobs that degrade retrieval precision and increase storage cost.

### 3.4 Injection budget control

When injecting memory into tool context or prompts, select small high-signal subsets rather than bulk injection. MCB's `max_tokens` and `limit` parameters in search serve this purpose.

---

## 4. Anti-Patterns to Avoid

- Storing raw conversation turns as long-term memory (noise, unbounded growth).
- Retrieving memory without namespace/tenant scoping (cross-contamination risk).
- Injecting all matching memories without ranking or budget control.

---

## 5. Cross-Document Map

- Complementary memory architecture patterns: `context/external/openai-agents-memory.md`
- MCB memory service implementation: `crates/mcb-application/src/use_cases/memory_service.rs`
- MCB observation entity: `crates/mcb-domain/src/entities/observation.rs`
- MCB hybrid search: `crates/mcb-providers/src/hybrid_search/engine.rs`

---

## 6. References

- LangChain/LangGraph documentation (memory, persistence, long-term-memory)
- MCB memory architecture in `crates/mcb-domain/src/ports/repositories/memory_repository.rs`
- MCB search infrastructure in `crates/mcb-application/src/use_cases/memory_service.rs`
