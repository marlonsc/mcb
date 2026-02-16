<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# snapshot Module

**Note**: Snapshot functionality is defined as a port trait in v0.2.1.

**Trait**: `crates/mcb-domain/src/ports/infrastructure/snapshot.rs`
**Adapter**: No adapter implementation yet (port trait only)

## Overview

Snapshot management for incremental codebase tracking. Tracks file changes using SHA256 hashing for efficient incremental sync. Avoids reprocessing unchanged files during codebase indexing.

### Components

### SnapshotProvider Trait (`mcb-domain`)

Port definition for snapshot operations:

```rust
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    async fn capture(&self, path: &Path) -> Result<CodebaseSnapshot>;
    async fn compare(&self, old: &CodebaseSnapshot, new: &CodebaseSnapshot) -> Result<SnapshotChanges>;
    async fn store(&self, snapshot: &CodebaseSnapshot) -> Result<()>;
    async fn load(&self, id: &str) -> Result<Option<CodebaseSnapshot>>;
}
```

**Status**: Port traits defined, no adapter implementations yet.

## File Structure

```text
crates/mcb-domain/src/ports/infrastructure/
└── snapshot.rs              # SnapshotProvider trait, StateStoreProvider trait
```

## Domain Types

Related types in `mcb-domain`:

- `CodebaseSnapshot` - Point-in-time codebase state
- `FileSnapshot` - Individual file state with hash
- `SnapshotChanges` - Delta between snapshots

## Key Exports

```rust
// Traits (from mcb-domain)
pub use ports::infrastructure::snapshot::{SnapshotProvider, StateStoreProvider};
```

## Cross-References

- **Domain**: [domain.md](./domain.md) (trait definition)
- **Sync**: [sync.md](./sync.md) (uses snapshots)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
