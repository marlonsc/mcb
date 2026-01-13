# snapshot Module

**Source**: `src/infrastructure/snapshot/`
**Files**: 2
**Lines of Code**: 325
**Traits**: 0
**Structs**: 1
**Enums**: 0
**Functions**: 0

## Overview

Snapshot management for incremental codebase tracking
//!
Tracks file changes using SHA256 hashing for efficient incremental sync.
Avoids reprocessing unchanged files during codebase indexing.

## Key Exports

`manager::SnapshotManager,crate::domain::types::{CodebaseSnapshot, FileSnapshot, SnapshotChanges},`

## File Structure

```text
manager.rs
mod.rs
```

---

*Auto-generated from source code on seg 12 jan 2026 11:25:13 -03*
