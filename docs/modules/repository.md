<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# repository Module

**Source**: `crates/mcb-infrastructure/src/adapters/repository/`
**Traits**: `crates/mcb-domain/src/repositories/`
**Crate**: `mcb-infrastructure`
**Files**: 3
**Lines of Code**: ~400

## Overview

Repository pattern implementation for data access abstraction. Provides repository interfaces and null implementations following the Repository pattern to separate data access logic from business logic.

### Components

### Repository Traits (`mcb-domain`)

Port definitions for repositories:

- `ChunkRepository` - Code chunk persistence operations
- `SearchRepository` - Search Result retrieval operations

**Status**: Port traits defined, no adapter implementations yet.

## File Structure

```text
crates/mcb-domain/src/repositories/
├── chunk_repository.rs       # ChunkRepository trait
├── search_repository.rs      # SearchRepository trait
└── mod.rs
```

## Repository Pattern

```rust
// Port trait (in mcb-domain); DI via dill (ADR-029)
#[async_trait]
pub trait ChunkRepository: Send + Sync {
    async fn store(&self, collection: &str, chunks: &[CodeChunk]) -> Result<()>;
    async fn get(&self, collection: &str, id: &str) -> Result<Option<CodeChunk>>;
    async fn delete(&self, collection: &str, id: &str) -> Result<()>;
}
```

## Key Exports

```rust
// Traits (from mcb-domain)
pub use repositories::{ChunkRepository, SearchRepository};
```

## Cross-References

- **Domain**: [domain.md](./domain.md) (trait definitions)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
