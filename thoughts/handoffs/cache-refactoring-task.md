# Cache Module Refactoring - Handoff for Implementation Agent

**Status**: PENDING - Ready for implementation
**Priority**: HIGH - Blocking compilation
**Assignee**: Cache Refactoring Agent
**Date**: 2026-01-12

## Context

Phase 1 Security Fixes completed successfully, but cache module refactoring was started and left incomplete. Current state: **BROKEN COMPILATION** with 6 errors and 9 warnings.

### Current Errors

1. **CacheBackendConfig undefined** - `factory.rs` imports non-existent type
2. **MokaCacheProvider.config field removed** - Factory calls `config.clone()` but field no longer exists
3. **RedisCacheProvider simplified** - Only has `client` field, but factory expects `url` and `default_ttl` parameters
4. **Deprecated field warnings** - 8 deprecation notices about CacheConfig legacy fields

## Requirements

### MUST DO (Blocking)
1. **Implement CacheBackendConfig enum** in `src/infrastructure/cache/config.rs`
   - Variants: `Local { max_entries, default_ttl_seconds }`, `Redis { url, pool_size, default_ttl_seconds }`
   - Add to CacheConfig struct as `backend: CacheBackendConfig` field
   - Implement migration logic from legacy fields

2. **Restore MokaCacheProvider fields** for factory compatibility
   - Keep `config: CacheConfig` field (needed by factory for `config.clone()`)
   - Field IS used - don't remove it

3. **Restore RedisCacheProvider fields** for factory compatibility
   - Keep `url: String` (used in logging/diagnostics)
   - Keep `default_ttl: Duration` (used in TTL enforcement)
   - Fields should be prefixed with underscore if truly unused: `_url`, `_default_ttl`

4. **Verify factory.rs compiles and is actively used**
   - Currently only imports exist, no actual calls found
   - If unused: document as dead code for Phase 2 cleanup
   - If used: integrate with server initialization

5. **Remove deprecated field warnings**
   - Migrate all uses of `max_size`, `redis_url`, `default_ttl_seconds` to `backend` enum

### SHOULD DO (Code Quality)
1. Make all struct fields either:
   - Actively used (no prefix)
   - Consciously unused (prefix with `_` instead of `#[allow(dead_code)]`)
2. Convert CacheManager to implement `CacheProvider` trait
3. Update server initialization to use `SharedCacheProvider` instead of `Arc<CacheManager>` directly

## Files to Modify

- `src/infrastructure/cache/config.rs` - Add CacheBackendConfig enum
- `src/infrastructure/cache/providers/moka.rs` - Ensure config field exists
- `src/infrastructure/cache/providers/redis.rs` - Restore url and default_ttl fields
- `src/infrastructure/cache/factory.rs` - Verify it's working correctly
- `src/infrastructure/cache/manager.rs` - Update to use CacheBackendConfig

## Success Criteria

- [ ] `cargo build --lib` completes with zero errors
- [ ] `cargo clippy --all-targets` shows zero cache-related errors
- [ ] All deprecation warnings about CacheConfig fields removed
- [ ] Factory function compiles and is testable
- [ ] No `#[allow(dead_code)]` annotations - use `_prefix` instead
- [ ] All tests pass: `cargo test`

## Test Commands

```bash
# Build without errors
cargo build --lib 2>&1 | grep "error\["

# Check clippy
cargo clippy --all-targets --all-features 2>&1 | grep "error:\|warning:" | grep -v "unused variable"

# Run tests
cargo test --lib cache
```

## Notes

- Phase 1 Security work (credentials, config) is COMPLETE and working
- This is a side effect of incomplete cache refactoring from earlier session
- Fast fail approach: Build must pass, no partial solutions
- After this: Focus on non-cache components for DI patterns and dead code removal
