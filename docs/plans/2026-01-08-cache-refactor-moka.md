# Cache Refactor Implementation Plan (Moka)

> **IMPORTANT:** Start with fresh context. Run `/clear` before `/implement`.

Created: 2026-01-08
Status: VERIFIED

## Summary
**Goal:** Replace the manual `RwLock<HashMap>` cache implementation with `moka`, a high-performance, lock-free, async caching library, to eliminate deadlocks and improve concurrency.

**Architecture:**
- **Exclusive Mode Strategy:**
  - **Local Mode (Default):** Uses `moka::future::Cache` for high-performance in-memory caching.
  - **Remote Mode (Redis):** Uses Redis exclusively if `redis_url` is configured.
- Dedicate one Moka cache instance per namespace (`embeddings`, `search_results`, `metadata`, `provider_responses`) to respect individual TTLs and limits.
- Remove all manual eviction (LRU) and background cleanup tasks, relying on Moka's internal "TinyLFU" and TTL management.

**Tech Stack:** `moka` (v0.12+), `tokio`, `serde_json`.

## Scope

### In Scope
- Refactoring `src/core/cache.rs` to use `moka`.
- Adding `moka` dependency to `Cargo.toml`.
- Mapping existing namespaces to dedicated Moka cache instances.
- Updating all public methods (`get`, `set`, `delete`, `clear_namespace`, `get_stats`) to work with Moka.
- Removing manual cache maintenance code (`background_cleanup`, `evict_*`).
- Ensuring `tests/reproduce_freeze.rs` passes.

### Out of Scope
- Changing the public API of `CacheManager` (method signatures remain the same).
- Modifying other modules (internal refactor only).

## Prerequisites
- `moka` crate must be added to dependencies.

## Feature Inventory (Refactoring)

### Files Being Replaced/Modified

| File | Features | Mapped to Task |
|------|----------|----------------|
| `src/core/cache.rs` | `CacheManager` struct | Task 2 |
| | `new` (initialization) | Task 2 |
| | `get` | Task 2 |
| | `set` | Task 2 |
| | `delete` | Task 2 |
| | `clear_namespace` | Task 2 |
| | `get_stats` | Task 2 |
| | `background_cleanup` | **REMOVED** (Handled by Moka) |
| | `cleanup_expired_entries` | **REMOVED** (Handled by Moka) |
| | `evict_entries` | **REMOVED** (Handled by Moka) |
| | `evict_namespace_entries` | **REMOVED** (Handled by Moka) |
| | `get_from_local` | Task 2 (Refactor) |
| | `set_in_local` | Task 2 (Refactor) |
| | `delete_from_local` | Task 2 (Refactor) |

### Feature Mapping Verification
- [x] All public methods mapped.
- [x] Manual maintenance methods marked for removal.
- [x] Redis methods preserved (Task 2 will keep them).

## Progress Tracking

- [x] Task 1: Add Dependencies & Setup
- [x] Task 2: Refactor CacheManager to Moka
- [x] Task 3: Verify & Cleanup

**Total Tasks:** 3

## Implementation Tasks

### Task 1: Add Dependencies & Setup
**Objective:** Add `moka` to `Cargo.toml`.

**Files:**
- Modify: `Cargo.toml`

**Implementation Steps:**
1. Add `moka = { version = "0.12", features = ["future"] }` to `[dependencies]`.
2. Run `cargo check` (or equivalent `make` command) to verify dependencies.

**Definition of Done:**
- [x] `moka` dependency added.
- [x] Project compiles.

### Task 2: Refactor CacheManager to Moka
**Objective:** Replace `RwLock<HashMap>` with `moka::future::Cache` fields in `CacheManager`.

**Files:**
- Modify: `src/core/cache.rs`

**Implementation Steps:**
1. **Update Struct:** Change `CacheManager` fields.
   - Remove `local_cache`.
   - Add fields: `embeddings_cache`, `search_results_cache`, `metadata_cache`, `provider_responses_cache` (all `moka::future::Cache<String, serde_json::Value>`).
2. **Update `new`:**
   - Initialize each Moka cache using the config from `CacheNamespacesConfig` (TTL, max_capacity).
   - Remove `background_cleanup` spawning.
   - Implement "Exclusive Mode" logic: If Redis configured, use Redis (Remote). Else use Moka (Local).
3. **Helper Methods:** Implement a helper `get_cache(&self, namespace: &str) -> &Cache<...>` to select the correct cache instance based on string.
4. **Update `get`/`set`/`delete`:**
   - Use `self.get_cache(namespace).get(&full_key).await`.
   - Use `self.get_cache(namespace).insert(key, value).await`.
   - Remove manual `CacheEntry` wrapping for local cache (store `serde_json::Value` directly).
   - *Note:* Keep `CacheEntry` struct for Redis serialization only.
5. **Update `get_stats`:**
   - Sum up stats from all 4 Moka caches (`run_pending_tasks` might be needed for accuracy, or just read `stats()`).
   - Moka stats provide `hit_count`, `miss_count`, etc.

**Definition of Done:**
- [x] `CacheManager` uses Moka.
- [x] No `RwLock<HashMap>` usage.
- [x] Compiles without errors.
- [x] Existing tests in `src/core/cache.rs` pass.

### Task 3: Verify & Cleanup
**Objective:** Run the reproduction test to confirm the fix and ensure no regressions.

**Files:**
- Run: `tests/reproduce_freeze.rs`
- Run: `tests/config_tests.rs` (and other cache-related tests)

**Implementation Steps:**
1. Run `cargo test --test reproduce_freeze`.
2. Run `cargo test`.
3. Verify no deadlocks occur.

**Definition of Done:**
- [x] Reproduction test passes consistently (no freeze).
- [x] All cache unit tests pass.

## Testing Strategy
- **Unit Tests:** Use existing `src/core/cache.rs` tests.
- **Integration Tests:** `reproduce_freeze.rs` is the key verification.
- **Manual Verification:** None needed beyond tests.

## Risks and Mitigations
- **Risk:** Moka's async overhead vs std::sync::RwLock.
  - *Mitigation:* Moka is highly optimized for concurrent async access; overhead is negligible compared to lock contention.
- **Risk:** Loss of precise "access_count" tracking if strictly required.
  - *Mitigation:* `get_stats` aggregates global stats which Moka provides. Per-entry access count was internal logic for eviction, which Moka now handles.

## Open Questions
- None.
