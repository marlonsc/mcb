# Lock-Free Async Refactoring Plan

> **IMPORTANT:** Start with fresh context. Run `/clear` before `/implement`.

Created: 2026-01-08
Status: COMPLETE

## Summary
**Goal:** Eliminate locks entirely by adopting Actor Pattern and Concurrent Data Structures, while ensuring non-blocking I/O.

**Architecture:**
- **Actor Model:** Use `tokio::mpsc` channels to manage mutable state (HybridSearch), effectively serializing access without locks.
- **Concurrent Maps:** Replace `RwLock<HashMap>` with `DashMap` for high-performance concurrent access.
- **Pure Async I/O:** Convert blocking I/O to `tokio::fs`.
- **Job Queues:** Use channels to batch and throttle heavy indexing jobs.

**Tech Stack:** Rust, Tokio (Channels, Tasks, FS), DashMap

## Scope

### In Scope
- `src/sync/lockfile.rs`: Convert to async I/O
- `src/snapshot/mod.rs` & `src/core/merkle.rs`: Offload blocking recursion
- `src/providers/vector_store/filesystem.rs`: Rewrite to use `DashMap` & `tokio::fs`
- `src/services/context.rs`: Refactor to use Actor pattern for Hybrid Search
- `src/services/indexing.rs`: Offload parsing

### Out of Scope
- Changing underlying database engines
- Rewriting tree-sitter

## Feature Inventory (Refactoring)

### Files Being Modified

| File | Functions/structs | Mapped to Task |
|------|-------------------|----------------|
| `src/sync/lockfile.rs` | `acquire_lock`, `cleanup_stale_locks` | Task 1 |
| `src/server/handlers/get_indexing_status.rs` | `get_memory_usage` | Task 1 |
| `src/snapshot/mod.rs` | `create_snapshot`, `walk_directory` | Task 2 |
| `src/core/merkle.rs` | `from_directory` | Task 2 |
| `src/chunking/engine.rs` | `chunk_code` | Task 2 |
| `src/services/indexing.rs` | `process_file` | Task 2 |
| `src/providers/vector_store/filesystem.rs` | `FilesystemVectorStore` | Task 3 |
| `src/services/context.rs` | `ContextService`, `HybridSearchActor` | Task 4 |

## Progress Tracking

- [x] Task 1: Fix Simple Blocking I/O
- [x] Task 2: Offload Blocking Operations
- [x] Task 3: Lock-Free Vector Store (DashMap)
- [x] Task 4: Hybrid Search Actor (Channel-based)

**Total Tasks:** 4 | **Completed:** 4 | **Remaining:** 0

## Implementation Tasks

### Task 1: Fix Simple Blocking I/O
**Objective:** Replace simple `std::fs` calls with `tokio::fs` in `lockfile.rs` and `get_indexing_status.rs`.

**Implementation Steps:**
1. Update `src/sync/lockfile.rs`:
   - Use `tokio::fs` for all file operations.
   - Use `spawn_blocking` only for the specific OS lock call if needed.
2. Update `src/server/handlers/get_indexing_status.rs`:
   - Use `tokio::fs::read_to_string`.
   - Make `get_memory_usage` async.

**Definition of Done:**
- [x] No `std::fs` usage in these files (except wrapped).
- [x] Compiles and passes tests.

### Task 2: Offload Blocking Operations
**Objective:** Isolate heavy recursive FS walks and CPU parsing in blocking threads.

**Implementation Steps:**
1. `src/snapshot/mod.rs`: Wrap `walk_directory` in `tokio::task::spawn_blocking`.
2. `src/core/merkle.rs`: Create `from_directory_async` that wraps `from_directory` in `spawn_blocking`.
3. `src/chunking/engine.rs`: Create `chunk_code_async` wrapping parser logic in `spawn_blocking`.
4. Update `src/services/indexing.rs` to await these new async methods.

**Definition of Done:**
- [ ] No heavy blocking operations on the async thread.

### Task 3: Lock-Free Vector Store (DashMap)
**Objective:** Replace `RwLock<HashMap>` with `DashMap` in `FilesystemVectorStore`.

**Implementation Steps:**
1. Modify `src/providers/vector_store/filesystem.rs`:
   - Replace `index_cache: Arc<RwLock<HashMap<...>>>` with `Arc<DashMap<...>>`.
   - Replace `shard_cache: Arc<RwLock<HashMap<...>>>` with `Arc<DashMap<...>>`.
   - Replace `next_shard_id: Arc<RwLock<u32>>` with `Arc<std::sync::atomic::AtomicU32>`.
   - Remove `current_collection` lock - pass collection name to methods or use `DashMap` keyed by collection.
   - Refactor logic to use `DashMap` API (no explicit locking).
   - Ensure `tokio::fs` is used for I/O.

**Definition of Done:**
- [ ] `RwLock` removed from `FilesystemVectorStore`.
- [ ] All I/O is async.

### Task 4: Hybrid Search Actor (Channel-based)
**Objective:** Move `HybridSearchEngine` state into an actor to eliminate `RwLock` and enable batching.

**Implementation Steps:**
1. Define `HybridSearchMessage` enum:
   - `Index { collection: String, chunks: Vec<CodeChunk> }`
   - `Search { query: String, ... , respond_to: Sender<Result<Vec<...>>> }`
   - `Clear { collection: String }`
2. Create `HybridSearchActor` struct that owns `HybridSearchEngine`.
3. Implement `run(mut receiver)` loop that processes messages.
   - For `Index`, accumulate/batch updates if needed, or just process sequentially (safe without locks).
4. Update `ContextService`:
   - Hold `sender: mpsc::Sender<HybridSearchMessage>` instead of `Arc<RwLock<HybridSearchEngine>>`.
   - Implement `search_similar` by sending a message and awaiting the response (oneshot).

**Definition of Done:**
- [ ] `RwLock` removed from `ContextService`.
- [ ] Hybrid Search uses message passing.

## Risks and Mitigations
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Actor bottleneck | Low | Medium | Hybrid Search is CPU bound; Actor naturally throttles it. If too slow, can use multiple actors (sharding). |
| DashMap memory usage | Low | Low | DashMap overhead is minimal compared to storing vectors. |
