# Integration Points Context

**Last updated:** 2026-02-02
**Source:** `README.md` (provider/vector store lists) and `.planning/STATE.md` (memory search progress)

## Overview

Memory Context Browser composes embedding providers, vector stores, SQLite FTS, and MCP services so semantic search stays performant while mirroring git history and memory data.

## Key Integrations

### Embedding provider matrix

**Used in:** `README.md` "Provider Ecosystem"

-   Providers: OpenAI, VoyageAI, Ollama, Gemini, FastEmbed, Null (7 total).
-   All providers implement `EmbeddingProvider` so swapping them only requires new `embed_batch`/`dimensions()` implementations plus a distributed-slice entry.
**When to update:** Add documentation in `docs/providers/` and this context file whenever a new provider enters the ecosystem.

### Vector store adapters

**Used in:** `README.md` (vector store list: in-memory, encrypted, filesystem, Milvus, EdgeVec, Null)
**Integration note:** Each vector store adapter must map to the `VectorStore` trait in `mcb-providers`, and configuration is centralized in `docs/CONFIGURATION.md` and environment variables documented there.

### SQLite + FTS5 hybrid search

**Used in:** `.planning/STATE.md` (Phase 6 progress, Hybrid Search plan)

-   The product relies on SQLite triggers and SHA256 deduplication to keep memory observations in sync with FTS indices.
-   `MemoryRepository` traits include deletion helpers so cleanup happens during Hybrid Search.

## Related Context

-   `docs/context/domain-concepts.md`
-   `docs/CONFIGURATION.md` (env var wiring for providers/vector stores)
