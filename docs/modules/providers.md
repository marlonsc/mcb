# providers Module

**Source**: `src/providers/`
**Files**: 20
**Lines of Code**: 5125
**Traits**: 2
**Structs**: 0
**Enums**: 0
**Functions**: 0

## Overview

Provider interfaces and implementations

## Key Exports

`embedding::NullEmbeddingProvider as MockEmbeddingProvider // Backward compatibility,vector_store::InMemoryVectorStoreProvider,`

## File Structure

```text
embedding/openai.rs
embedding/ollama.rs
embedding/null.rs
embedding/mod.rs
embedding/voyageai.rs
embedding/gemini.rs
vector_store/milvus.rs
vector_store/in_memory.rs
vector_store/null.rs
vector_store/mod.rs
vector_store/encrypted.rs
vector_store/filesystem.rs
mod.rs
routing/health.rs
routing/circuit_breaker.rs
routing/metrics.rs
routing/cost_tracker.rs
routing/failover.rs
routing/router.rs
routing/mod.rs
```

---

*Auto-generated from source code on qua 07 jan 2026 11:52:25 -03*
