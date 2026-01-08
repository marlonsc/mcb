# Sync System Refactor: Batch-Based Implementation Plan

> **IMPORTANT:** Start with fresh context. Run `/clear` before `/implement`.

Created: 2026-01-08
Status: COMPLETE

## Summary
**Goal:** Replace the file-based lock system with a batch-based synchronization queue using the existing `CacheManager`.

**Relationship to Cache Plan:**
This plan **builds upon and completes** the pending `docs/plans/2026-01-08-cache-refactor-moka.md`.
- Task 1 of this plan incorporates the unfinished "Task 2" from the Cache Plan (fixing Moka compilation).
- Task 2 of this plan **extends** `CacheManager` with new Queue capabilities required for Sync.

**Architecture:**
- **State:** `CacheManager` acts as the central state store (KV + Queue).
- **Queueing:** `SyncManager` pushes `SyncBatch` items to `sync_batches` namespace.
- **Failover:**
    - **Redis:** Uses atomic List operations (`RPUSH`, `LRANGE`, `LREM`) for distributed coordination.
    - **Local (Moka):** Uses `entry().and_modify()` to atomically update a `Vec<SyncBatch>` stored as `serde_json::Value::Array` within the Moka cache.

**Tech Stack:** `moka`, `redis`, `tokio`, `serde_json`.

## Scope

### In Scope
- **Finalizing `CacheManager`:** Completing the Moka refactor (Task 2 of Cache Plan) to fix current compilation errors.
- **New Namespace:** Adding `sync_batches` to `CacheConfig`.
- **Queue Methods:** Extending `CacheManager` with `enqueue_item`, `get_queue`, `remove_item` handling Redis/Local split transparently.
- **Sync Refactor:** Updating `SyncManager` to use the queue system.
- **Cleanup:** Removing `src/sync/lockfile.rs`.

## Prerequisites
- `moka` crate is already added (from Cache Plan Task 1).

## Feature Inventory (Refactoring)

### Files Being Replaced/Modified

| Old File | Features | Status | Mapped to Task |
|----------|----------|--------|----------------|
| `src/sync/lockfile.rs` | `CodebaseLockManager` | 🗑️ REMOVE | Task 4 |
| `src/core/cache.rs` | `local_cache` refs | 🔄 FIX | Task 1 |
| | `enqueue_item` | ✨ NEW | Task 2 |
| `src/sync/manager.rs` | `sync_codebase` | 🔄 MODIFY | Task 3 |

### Feature Mapping Verification
- [x] Lockfile replaced by Cache Queue.
- [x] Broken `local_cache` calls replaced by Moka calls.

## Progress Tracking

- [x] Task 1: Finalize CacheManager (Complete Moka Refactor)
- [x] Task 2: Implement Queue Methods & Namespace
- [x] Task 3: Refactor SyncManager
- [x] Task 4: Cleanup & Verify

**Total Tasks:** 4

## Implementation Tasks

### Task 1: Finalize CacheManager (Complete Moka Refactor)
**Objective:** Finish the work started in Cache Plan Task 2. Fix all compilation errors in `src/core/cache.rs`.

**Files:**
- Modify: `src/core/cache.rs`

**Implementation Steps:**
1.  **Fix Moka Migration:**
    -   Replace all calls to `self.local_cache` with `self.get_cache(namespace)`.
    -   `get_from_local`: Call `cache.get(full_key)`.
    -   `set_in_local`: Call `cache.insert(full_key, value)`.
    -   `delete_from_local`: Call `cache.invalidate(full_key)`.
    -   `clear_namespace_local`: Call `cache.invalidate_entries_if(...)`.
    -   Remove `RwLock` logic (Moka is concurrent).
    -   Fix `get_stats` to aggregate stats from all 4 Moka instances.
2.  **Verify:** Run `cargo check` to ensure the core cache system compiles.

**Definition of Done:**
- [x] `cargo check` passes.
- [x] `CacheManager` is fully migrated to Moka.

### Task 2: Implement Queue Methods & Namespace
**Objective:** Extend `CacheManager` to support the Sync Queue.

**Files:**
- Modify: `src/core/cache.rs`
- Modify: `src/core/types.rs`

**Implementation Steps:**
1.  **Add `SyncBatch` Struct:** In `src/core/types.rs`.
    -   Fields: `id` (String), `path` (String), `created_at` (u64).
2.  **Update Config:** Add `sync_batches` to `CacheNamespacesConfig`.
    -   Default TTL: 86400s (24h).
3.  **Implement Queue Methods in `CacheManager`:**
    -   `enqueue_item(namespace, key, value)`:
        -   **Redis:** `RPUSH {key} {json}`.
        -   **Local:** Use `cache.entry(key).and_modify(|v| { ... push ... }).or_insert(vec![value])` for atomic update.
    -   `get_queue(namespace, key)`:
        -   **Redis:** `LRANGE {key} 0 -1`.
        -   **Local:** `cache.get(key)`.
    -   `remove_item(namespace, key, value_id)`:
        -   **Redis:** `LREM` (if ID matches) or custom LUA script if needed (for now `LREM` by value matching).
        -   **Local:** `cache.entry(key).and_modify(|v| { ... retain ... })`.

**Definition of Done:**
- [x] `sync_batches` namespace exists.
- [x] Queue methods implemented and tested.

### Task 3: Refactor SyncManager
**Objective:** Switch SyncManager to use the Cache Queue.

**Files:**
- Modify: `src/sync/manager.rs`

**Implementation Steps:**
1.  **Remove Lockfile:** Delete `CodebaseLockManager` calls.
2.  **Update `sync_codebase`:**
    -   Create `SyncBatch`.
    -   Call `cache.enqueue_item("sync_batches", "queue", batch)`.
    -   Retrieve queue.
    -   **Logic:**
        -   If queue head is ME, proceed with sync.
        -   If queue head is NOT ME, return `Ok(false)` (Debounced/Queued).
        -   *Note:* Real batch processing would involve a worker, but for minimal changes we treat the queue as a "Waiting Line".
3.  **Completion:**
    -   After sync, call `remove_item` to pop self from queue.
4.  **Health Check:**
    -   Periodically scan queue (or on every sync attempt).
    -   Remove items older than 24h.

**Definition of Done:**
- [x] `sync_codebase` uses Cache Queue.
- [x] No file locks.
- [x] Stale batches cleaned up.

### Task 4: Cleanup & Verify
**Objective:** Remove old code and verify.

**Files:**
- Delete: `src/sync/lockfile.rs`
- Modify: `src/sync/mod.rs`
- Modify: `Cargo.toml`

**Implementation Steps:**
1.  Remove `fs2`.
2.  Delete lockfile module.
3.  Run `tests/reproduce_freeze.rs` and `cargo test`.

**Definition of Done:**
- [x] Tests pass.

## Open Questions
- None.

---
**USER: Please review this plan. Edit any section directly, then confirm to proceed.**
