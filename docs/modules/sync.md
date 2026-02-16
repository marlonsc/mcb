<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# sync Module

**Note**: Sync functionality is defined as a port trait in v0.2.1.

**Trait**: `crates/mcb-domain/src/ports/infrastructure/sync.rs`
**Null Adapter**: `crates/mcb-infrastructure/src/adapters/infrastructure/sync.rs`

## Overview

File synchronization coordination for incremental indexing. Manages file change detection and coordinates re-indexing of modified files.

### Components

### SyncProvider Trait (`mcb-domain`)

Port definition for sync operations:

```rust
#[async_trait]
pub trait SyncProvider: Send + Sync {
    async fn sync(&self, path: &Path) -> Result<SyncResult>;
    async fn get_changes(&self, path: &Path) -> Result<Vec<FileChange>>;
    fn is_file_changed(&self, path: &Path, hash: &str) -> bool;
}
```

### LockProvider Trait (`mcb-domain`)

Distributed locking for concurrent sync:

```rust
#[async_trait]
pub trait LockProvider: Send + Sync {
    async fn acquire(&self, key: &str) -> Result<Lock>;
    async fn release(&self, lock: Lock) -> Result<()>;
}
```

**Status**: Port traits defined, no adapter implementations yet.

## File Structure

```text
crates/mcb-domain/src/ports/infrastructure/
└── sync.rs                  # SyncProvider, LockProvider traits
```

## Key Exports

```rust
// Traits (from mcb-domain)
pub use ports::infrastructure::sync::{SyncProvider, LockProvider};
```

## Cross-References

- **Domain**: [domain.md](./domain.md) (trait definition)
- **Snapshot**: [snapshot.md](./snapshot.md) (change detection)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
