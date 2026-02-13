<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
# v0.3.0 Implementation Plan - Phase 1 Stabilization

## Status

**Phase**: 1 (Analysis & Stabilization)
**Date**: 2026-02-08
**Focus**: Unblocking `context_search` and stabilizing tests.

## 1. Blocking Issues Analysis

The primary blocker for v0.2.0 stabilization is the missing `context_search`
implementation in the MCP layer. This blocks 20+ beads issues related to
context-aware features (mcb-cxn).

### Diagnosis

1. **Missing Enum Variant**: `SearchResource` in `mcb-server/src/args/consolidated.rs`
   only supports `Code` and `Memory`. It is missing `Context`.
2. **Missing Handler Logic**: `SearchHandler` in `mcb-server/src/handlers/search.rs`
   does not handle `SearchResource::Context` requests.
3. **Service Injection**: `SearchHandler` needs access to `ContextServiceInterface`
   (defined in `mcb-domain/src/ports/services/context.rs`) but currently only
   has `SearchServiceInterface` and `MemoryServiceInterface`.

## 2. Context Search Implementation Plan

To unblock the queue, we must implement the `context` resource in the search handler.

### Step 1: Update Domain Arguments

**File**: `crates/mcb-server/src/args/consolidated.rs`

- Add `Context` to `SearchResource` enum.

```rust
pub enum SearchResource {
    Code,
    Memory,
    Context, // Add this
}
```

### Step 2: Extend Search Handler

**File**: `crates/mcb-server/src/handlers/search.rs`

- Inject `ContextServiceInterface`.
- Implement `Context` case in `handle` method.

```rust
pub struct SearchHandler {
    search_service: Arc<dyn SearchServiceInterface>,
    memory_service: Arc<dyn MemoryServiceInterface>,
    context_service: Arc<dyn ContextServiceInterface>, // Add this
}

// In handle():
match args.resource {
    SearchResource::Code => { ... },
    SearchResource::Memory => { ... },
    SearchResource::Context => {
        // Implementation using self.context_service.search_similar()
    }
}
```

### Step 3: Update Factory/Registration

**File**: `crates/mcb-server/src/handlers/mod.rs` (or equivalent registry)

- Update `SearchHandler::new` calls to pass the context service.

## 3. Test Optimization Strategy

`cargo test` currently exceeds 120s timeout. The following E2E tests are the likely
culprits and should be marked `#[ignore]` for standard CI runs, or moved to a
separate "heavy" test suite.

### Candidates for `#[ignore]`

1. `tests/golden/test_end_to_end.rs`: Full system flow is expensive.
2. `tests/golden/test_git_awareness_e2e.rs`: Git operations are slow.
3. `tests/golden/test_memory_operations_e2e.rs`: Database/Memory interactions
   are slow.

**Action**: Add `#[ignore]` attribute to these tests. Run them explicitly via
`cargo test -- --ignored` in a separate CI stage.

## 4. Next Steps (Phase 1 Execution)

1. Create feature branch `fix/context-search`.
2. Apply changes to `consolidated.rs` and `search.rs`.
3. Verify compilation and unit tests.
4. Apply `#[ignore]` to slow tests.
5. Run `cargo test` to verify speed improvement (< 30s target).
6. Commit and push to unblock beads issues.
