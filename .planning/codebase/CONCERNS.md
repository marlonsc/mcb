# Codebase Concerns

**Analysis Date:** 2026-01-31

## Tech Debt

### 1. Duplicate Tree-Sitter Integration

**Issue:** tree-sitter integrated independently in two crates (mcb-validate and mcb-providers)

**Files:**
- `crates/mcb-validate/Cargo.toml` - 13 tree-sitter language dependencies
- `crates/mcb-providers/Cargo.toml` - 13 optional tree-sitter language dependencies
- `crates/mcb-validate/src/metrics/rca_analyzer.rs` - RCA-based parsing
- `crates/mcb-providers/src/language/engine.rs` - Direct tree-sitter parsing

**Impact:**
- Binary size bloat (tree-sitter compiled twice with identical version)
- Maintenance burden (new language support must update both crates)
- Increased compilation time
- No code reuse between validation and chunking systems

**Fix Approach:**
1. Create `mcb-language-support` crate with unified Language enum and detection logic
2. Consolidate tree-sitter dependency management into single location
3. Replace `RcaAnalyzer::detect_language()` with shared function
4. Replace `detection::language_from_extension()` with shared function
5. Update both crates to use shared language detection module

---

### 2. No Centralized Language Support Infrastructure

**Issue:** Language detection logic duplicated with no single source of truth

**Files:**
- `crates/mcb-validate/src/metrics/rca_analyzer.rs:86-99` - Uses LANG enum from rust-code-analysis
- `crates/mcb-providers/src/language/detection.rs:1-50` - Uses custom Language enum
- Extension matching patterns differ across implementations

**Current Duplication:**
```rust
// mcb-validate
"rs" => Some(LANG::Rust),
"py" => Some(LANG::Python),
"js" | "mjs" | "cjs" | "jsx" => Some(LANG::Mozjs),

// mcb-providers
"rs" | "rlib" => Some(Language::Rust),
"py" => Some(Language::Python),
"js" => Some(Language::Javascript),
```

**Impact:**
- Hard to extend with new languages (must update both systems)
- Different extension patterns lead to inconsistent behavior
- Future features (language-specific metrics, validation rules) cannot reuse logic
- Test coverage needed in multiple places

**Fix Approach:**
1. Create unified Language enum (superset of both LANG and Language types)
2. Centralize file extension registry with glob patterns
3. Create shared config for language-specific parameters (chunk size, complexity thresholds)
4. Export from `mcb-language-support` crate for both consumers

---

### 3. Architecture Dependency Violation: mcb-infrastructure → mcb-validate

**Issue:** Production code layer (infrastructure) depends on dev tooling (validation)

**Files:**
- `crates/mcb-infrastructure/src/validation/service.rs:1-10` - Direct imports from mcb-validate
- `crates/mcb-infrastructure/Cargo.toml` - mcb-validate as regular dependency (not dev-dependency)
- `crates/mcb-server/src/handlers/validate_file.rs` - Uses validation service

**Violates:**
- Clean architecture (infrastructure should only depend on domain)
- Port-based isolation (uses internal types directly, not via ports)
- Dev/prod separation (validation is marked as tooling, not core service)

**Impact:**
- Cannot make mcb-validate optional without refactoring infrastructure
- Tight coupling to specific validation implementation
- Future alternative validators cannot be swapped
- Test mocking more complex due to deep imports

**Fix Approach:**
1. Define `ValidationServiceInterface` port in `mcb-domain/src/ports/services.rs` (already partially done)
2. Move `ArchitectureValidator` usage into infrastructure implementation
3. Register validation as provider via linkme (like other providers)
4. Update mcb-infrastructure to use service via port abstraction only
5. Make mcb-validate an optional feature flag

---

### 4. Metrics Infrastructure Silos in Validation

**Issue:** RcaAnalyzer (metrics) only accessible through validation tooling, not as independent service

**Files:**
- `crates/mcb-validate/src/metrics/rca_analyzer.rs` - RcaAnalyzer, RcaMetrics (16 metric types)
- `crates/mcb-infrastructure/src/validation/service.rs:analyze_file_complexity()` - Only usage

**Concerns:**
- Metrics (cyclomatic complexity, cognitive complexity, Halstead, LOC, MI, NOM, NARGS, NEXITS) are valuable but only used for validation
- Cannot use metrics independently for other analysis
- No MetricsProvider port exists (unlike EmbeddingProvider, VectorStoreProvider)
- MCP `analyze_complexity` handler tightly coupled to validation service

**Impact:**
- Other systems (IDE plugins, analysis tools) cannot access metrics without pulling mcb-validate
- Coupling increases when new metric-based validators are added
- Code reuse not possible if different metrics implementation needed

**Fix Approach:**
1. Create `mcb-metrics` crate with `MetricsProvider` port trait
2. Implement `RcaMetricsProvider` in mcb-providers as optional feature
3. Register metrics provider via linkme into application registry
4. Update validation service to use MetricsProvider from DI container
5. MCP handlers request MetricsProvider independently from ValidationServiceInterface

---

### 5. Different AST Parsing Patterns (RCA vs Direct Tree-Sitter)

**Issue:** Two different AST abstraction levels make cross-system utilities impossible

**Files:**
- `crates/mcb-validate/src/ast/decoder.rs` - RCA Callback wrapper pattern
- `crates/mcb-validate/src/ast/unwrap_detector.rs` - RCA-based node traversal
- `crates/mcb-providers/src/language/common/processor.rs` - Direct tree-sitter Node handling
- `crates/mcb-providers/src/language/rust.rs` through `kotlin.rs` - Direct tree-sitter in 13 processors

**Concerns:**
- mcb-validate locked into rust-code-analysis (custom fork) interface
- mcb-providers directly exposed to tree-sitter API changes
- Cannot share AST utilities between systems
- Adding new analysis (SLOC counting, dependency extraction) requires reimplementation

**Impact:**
- Harder to add language support (must implement both RCA callbacks and language processor)
- Difficult to refactor AST traversal patterns across 13 language processors
- Cannot extract common utilities (ancestor finding, scope resolution, name resolution)
- Future backend changes (e.g., switching from tree-sitter to another parser) require dual changes

**Fix Approach:**
1. Create `mcb-ast-utils` crate with language-agnostic abstractions
2. Define NodeAdapter trait (wraps both RCA Node and tree-sitter::Node)
3. Implement traversal utilities (ancestors, descendants, siblings, filtered walk)
4. Create query builder for common patterns (function definitions, variable uses)
5. Update validators to use adapters instead of RCA directly

---

## Known Bugs

### 1. File Lock Edge Case on Non-Unix Platforms

**Files:** `crates/mcb-server/src/collection_mapping.rs:85-89`

**Issue:** File locking skipped on Windows/non-Unix, uses best-effort with no fallback guarantee

**Symptom:** Concurrent collection name mapping writes on Windows may corrupt `collection_mapping.json`

**Current Code:**
```rust
#[cfg(not(unix))]
{
    // On non-Unix platforms, we skip locking (best effort)
    // Windows could use LockFileEx if needed
}
```

**Trigger:** Running multiple indexing operations simultaneously on Windows

**Workaround:** None (best-effort only)

**Fix Approach:**
1. Implement Windows file locking via `OpenFile` + `LockFileEx` from winapi
2. Make locking mandatory (not best-effort)
3. Add tests for concurrent access scenarios
4. Add warning in docs about Windows limitations if locking unavailable

---

### 2. Milvus Collection Not Ready on Index Creation

**Files:** `crates/mcb-providers/src/vector_store/milvus.rs:197-228`

**Issue:** Retry logic with hardcoded sleep durations doesn't guarantee collection readiness

**Symptom:** Intermittent "CollectionNotExists" errors when index creation races against collection creation

**Current Approach:** 3 retry attempts with exponential backoff (0-3 seconds total)

**Concerns:**
- No way to know if collection is truly ready vs. just having eventual consistency issues
- Fixed retry count may be insufficient for slow/overloaded Milvus clusters
- Sleep timing (500ms × attempt) may not align with actual readiness

**Fix Approach:**
1. Add exponential backoff with jitter instead of linear
2. Implement readiness polling (check collection stats) instead of time-based sleep
3. Make retry count and timeouts configurable per deployment
4. Add structured logging to track retry attempts and timing

---

## Security Considerations

### 1. Unsafe Code in Cryptography

**Files:** `crates/mcb-infrastructure/src/crypto/utils.rs:51-59`

**Risk:** Using unsafe to mutate String bytes for secure erasure

**Current Code:**
```rust
unsafe {
    let bytes = s.as_bytes_mut();
    Self::secure_erase(bytes);
}
```

**Mitigation:** Comment documents exclusivity of access and UTF-8 invariant preservation

**Concerns:**
- Relies on developer's understanding of unsafe invariants
- Zeroizing doesn't prevent compiler optimizations from removing overwrites
- Should use `zeroize` crate instead of manual implementation

**Recommendation:**
1. Replace custom SecureErasure with `zeroize` crate (proven cryptographic library)
2. Remove unsafe code (zeroize handles all unsafe internally with proper guarantees)
3. Update crypto/utils.rs to use `Zeroize` trait
4. Remove manual zeroize implementations

---

### 2. Hardcoded Fallback IP in Admin API Configuration

**Files:** `crates/mcb-server/src/admin/api.rs:50-57`

**Risk:** `.expect()` on hardcoded IP parsing

**Current Code:**
```rust
let address: IpAddr = self.host.parse().unwrap_or_else(|_| {
    "127.0.0.1"
        .parse()
        .map_err(|e| {
            tracing::error!("Failed to parse fallback IP 127.0.0.1: {}", e);
            e
        })
        .expect("Hardcoded fallback IP should always parse")
});
```

**Mitigation:** Comment acknowledges hardcoded value, validation at startup

**Concerns:**
- `.expect()` can panic if Rust's IP parser changes behavior (unlikely but possible)
- Validates at startup but adds unnecessary error handling overhead

**Recommendation:**
1. Use compile-time constant instead: `const FALLBACK_IP: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);`
2. Remove parsing entirely for hardcoded value
3. Update error handling to never reach fallback (fail earlier)

---

### 3. File System Race Condition in Config Watcher

**Files:** `crates/mcb-infrastructure/src/config/watcher.rs:101-147`

**Risk:** Config file may be partially written when watch event fires

**Current Implementation:**
- File watcher triggers async reload
- No atomic read or write coordination
- Possible to read partially-written config

**Mitigation:** None currently (relies on filesystem atomicity)

**Concern:** TOCTOU (Time-of-Check-Time-of-Use) issue if config file is externally edited

**Recommendation:**
1. Use atomic file operations (write to temp, then rename)
2. Add file lock coordination with writers
3. Implement config validation retry loop (if parse fails, retry with exponential backoff)
4. Log config reload errors but don't crash (fail-safe)

---

## Performance Bottlenecks

### 1. Vector Embedding Clone Operations in Milvus Provider

**Files:** `crates/mcb-providers/src/vector_store/milvus.rs:245-350+`

**Problem:** 47+ clone() calls in vector store operations

**Symptoms:** High memory pressure during bulk indexing; CPU cycles spent on memcpy

**Bottleneck Locations:**
- `insert_vectors()` - Cloning metadata HashMap for each vector
- `search()` - Cloning search results and embeddings
- Collection info building - Repeated string allocations

**Impact:** O(n) memory overhead proportional to dataset size

**Fix Approach:**
1. Use references (Cow<str>, &str) instead of cloning strings
2. Use iterators instead of collecting into Vec then iterating
3. Lazy-build metadata only when needed
4. Pre-allocate result buffers (Vec::with_capacity)
5. Benchmark clone impact on 100K+ vectors

---

### 2. Synchronous String Formatting in Hot Paths

**Files:** `crates/mcb-providers/src/vector_store/` (multiple files)

**Problem:** 298+ uses of `format!()` and `to_string()` in provider code

**Symptoms:** Allocations in I/O-bound operations add latency

**Example Bottleneck:**
```rust
// In hot loop
let msg = format!("vector_{}", i);  // Every iteration
```

**Fix Approach:**
1. Profile actual impact (may be negligible vs. network latency)
2. Use static strings where possible
3. Reserve string buffers for repeated formats
4. Use `write!()` macro instead of `format!()` for non-string outputs

---

### 3. RcaAnalyzer Full File Parsing on Each Request

**Files:** `crates/mcb-validate/src/metrics/rca_analyzer.rs`

**Problem:** Reparses entire file for complexity analysis on every analyze_file() call

**Symptoms:** Slow analysis on large files (1000+ lines)

**Impact:** O(file_size) time per request, no caching

**Fix Approach:**
1. Add optional LRU cache for parsed files (keyed by path + mtime)
2. Invalidate cache on file changes
3. Make caching opt-in via config
4. Profile cache hit rates in typical workloads

---

## Fragile Areas

### 1. mcb-validate Organization Module (1534 lines)

**Files:** `crates/mcb-validate/src/organization.rs:1-1534`

**Why Fragile:** Extremely large validator with complex state machine for detecting violations

**Concerns:**
- Single file handles magic numbers, duplicates, decentralization, type placement, file locations
- Complex regex patterns for pattern matching
- Multiple passes over AST (inefficient)
- Hard to unit test individual checks due to coupling

**Safe Modification:**
1. Add new violation type: extend enum, add case in reporter
2. Change threshold: update const at top of file
3. Add new rule: high risk (affects pattern matching logic throughout)
4. Fix false positive: understand entire violation type flow first

**Test Coverage Gaps:**
- No tests for edge cases (4-digit magic numbers, duplicate detection edge cases)
- False positive scenarios not covered
- Integration with other validators not tested

---

### 2. Collection Name Mapping with File Locking

**Files:** `crates/mcb-server/src/collection_mapping.rs:47-199`

**Why Fragile:** File I/O + locking coordination across process boundaries

**Concerns:**
- RAII FileLockGuard assumes Unix-like semantics (not Windows)
- Atomic rename may fail if destination exists (though unlikely with .tmp extension)
- Corrupted JSON would brick collection mapping permanently
- No recovery mechanism if lock file gets stuck

**Safe Modification:**
1. Reading mapping: Always safe (no side effects)
2. Adding new collection: Safe if lock is held (atomic write guaranteed)
3. Changing lock strategy: High risk (affects all concurrent access)

**Test Coverage Gaps:**
- No tests for concurrent mapping operations
- Windows platform behavior not tested
- Lock file cleanup not verified
- Corrupted JSON recovery not tested

---

### 3. AST Unwrap Detection in mcb-validate

**Files:** `crates/mcb-validate/src/ast/unwrap_detector.rs:1-296`

**Why Fragile:** RCA Callback pattern with custom recursive AST walking

**Concerns:**
- RCA fork may have bugs in AST construction
- Test/non-test detection logic uses comment parsing (fragile)
- Recursion depth not limited (potential stack overflow on pathological input)
- Different behavior if RCA node types change

**Safe Modification:**
1. Adjusting severity thresholds: Safe
2. Adding new unwrap patterns: Medium risk (must update regex correctly)
3. Changing test detection logic: High risk (affects many tests)
4. Switching to direct tree-sitter: Very high risk (requires complete rewrite)

**Test Coverage Gaps:**
- No tests for deeply nested code (recursion limit)
- Edge cases in test detection (module-level attributes, derives)
- Different file encodings not tested

---

### 4. Milvus Dynamic Collection Schema

**Files:** `crates/mcb-providers/src/vector_store/milvus.rs:153-238`

**Why Fragile:** Hardcoded field names and types; schema mismatch causes silent failures

**Concerns:**
- Field names ("vector", "file_path", "content") hardcoded in multiple places
- VARCHAR length limits (65535 for file_path) may be insufficient
- No schema validation after collection creation
- Index type (IvfFlat) assumes reasonable collection size (doesn't scale to 100M+ vectors)

**Safe Modification:**
1. Changing VARCHAR lengths: Medium risk (affects existing collections)
2. Adding new fields: High risk (requires schema versioning)
3. Changing index strategy: High risk (IvfFlat not optimal for all dataset sizes)

**Test Coverage Gaps:**
- No tests for large VARCHAR content (edge case)
- Index performance not tested
- Schema evolution scenarios not covered

---

## Scaling Limits

### 1. In-Memory Cache Provider

**Component:** `crates/mcb-providers/src/cache/in_memory.rs`

**Current Capacity:** Limited by available RAM (no eviction strategy)

**Limit:** Cache grows unbounded until memory exhausted

**Scaling Path:**
1. Implement LRU eviction with configurable max_size
2. Add cache statistics (hit rate, eviction rate)
3. Monitor memory pressure
4. Implement spillover to disk cache (Redis) when RAM limit reached

---

### 2. In-Memory Vector Store

**Component:** `crates/mcb-providers/src/vector_store/in_memory.rs`

**Current Capacity:** All vectors in RAM (no persistence)

**Limit:** Cannot scale beyond available RAM

**Scaling Path:**
1. Add optional persistence layer (checkpoint to disk)
2. Implement pagination for large result sets
3. Add vector quantization to reduce memory footprint
4. Support partitioning across multiple in-memory stores

---

### 3. Milvus Collection Metadata Retrieval

**Component:** `crates/mcb-providers/src/vector_store/milvus.rs:get_stats()`

**Current Capacity:** Single HTTP call to get stats

**Limit:** Large collections may have slow stat retrieval

**Scaling Path:**
1. Implement cached stats with TTL
2. Add pagination for large metadata queries
3. Cache collection schema separately

---

### 4. AST Parsing with RcaAnalyzer

**Component:** `crates/mcb-validate/src/metrics/rca_analyzer.rs`

**Current Capacity:** Single-threaded file parsing

**Limit:** Analyzing large codebases (100K+ files) is slow

**Scaling Path:**
1. Implement parallel analysis with rayon
2. Add file batching (analyze multiple files in single task)
3. Implement incremental analysis (only changed files)
4. Add analysis results caching

---

## Dependencies at Risk

### 1. rust-code-analysis Fork

**Package:** Custom fork of rust-code-analysis

**Risk:** Fork maintenance burden, diverged from upstream

**Impact:**
- Cannot easily get upstream bug fixes
- Language updates only happen if fork maintainer applies them
- Tree-sitter parser updates require manual sync

**Migration Plan:**
1. Evaluate switching to direct tree-sitter APIs for metrics
2. Investigate lsp-types or rust-analyzer metrics libraries
3. If fork still needed: Set up CI to sync upstream changes regularly

---

### 2. Milvus Client Version Pinning

**Package:** `milvus` client (gRPC-based)

**Risk:** Milvus server version incompatibilities

**Current:** No server version compatibility matrix documented

**Impact:**
- Deploying new Milvus server version without client update causes failures
- No clear upgrade path documented

**Mitigation:**
1. Document supported Milvus versions in README
2. Test against multiple server versions in CI
3. Add version check in client initialization

---

### 3. Rocket Framework Migration (ADR-026)

**Package:** Rocket web framework (migrated from Axum in v0.1.2)

**Risk:** Framework selection may not scale for admin API needs

**Impact:**
- Different from rest of async ecosystem
- Smaller community than Actix/Axum
- Custom middleware needed for complex scenarios

**Mitigation:**
1. Monitor Rocket updates and breaking changes
2. Document why Rocket was chosen (easier routing for admin API)
3. Consider re-evaluation if authentication complexity grows

---

## Test Coverage Gaps

### 1. Concurrent Collection Mapping Operations

**Untested Scenario:** Multiple processes writing collection mappings simultaneously

**Files:** `crates/mcb-server/src/collection_mapping.rs`

**Risk:** Race conditions on non-Unix platforms, potential file corruption

**Test Plan:**
- Spawn multiple threads calling `map_collection_name()` concurrently
- Verify output file integrity
- Verify no duplicate mappings created
- Run on Windows to verify non-Unix behavior

---

### 2. Config Watcher File System Events

**Untested Scenario:** Config file modifications (CHMOD, DELETE, RENAME, WRITE)

**Files:** `crates/mcb-infrastructure/src/config/watcher.rs`

**Risk:** Config watcher may miss events or reload corrupted files

**Test Plan:**
- Test each event type (create, modify, delete, rename)
- Verify reload behavior on corrupted JSON
- Test rapid successive modifications
- Verify no state inconsistency during reload

---

### 3. Error Handling in Provider Initialization

**Untested Scenario:** Provider connection failures, timeouts, auth errors

**Files:**
- `crates/mcb-providers/src/embedding/*.rs` - All providers
- `crates/mcb-providers/src/vector_store/*.rs` - All providers

**Risk:** Initialization failures crash without proper error context

**Test Plan:**
- Mock provider timeouts
- Test missing credentials/invalid tokens
- Verify error messages are actionable
- Test fallback/retry behavior

---

### 4. Large File Analysis (1000+ line files)

**Untested Scenario:** RcaAnalyzer on very large files

**Files:** `crates/mcb-validate/src/metrics/rca_analyzer.rs`

**Risk:** Stack overflow on deeply nested code; slow performance

**Test Plan:**
- Generate 5000+ line procedural code file
- Verify no panics during analysis
- Measure performance (should complete in <1s)
- Test stack usage doesn't exceed limits

---

## Missing Critical Features

### 1. No Configuration Schema Validation

**Gap:** Configuration validation is minimal (type-level only)

**Files:** `crates/mcb-infrastructure/src/config/loader.rs`

**Impact:**
- Invalid config values silently use defaults instead of failing loudly
- Hard to debug configuration errors
- No JSON schema for config files

**Blocker For:**
- Self-hosted deployments (users don't know which settings matter)
- Kubernetes ConfigMap deployments (no schema validation)

**Implementation:**
1. Generate JSON schema from Figment config types
2. Add validation step in config loader
3. Add `--validate-config` CLI command

---

### 2. No Metrics Export for Monitoring

**Gap:** Performance metrics are not exposed for monitoring systems

**Files:** All provider implementations

**Impact:**
- Cannot integrate with Prometheus/Datadog/NewRelic
- No observability into provider health
- Hard to debug performance regressions

**Blocker For:**
- Production monitoring
- SLA tracking
- Capacity planning

**Implementation:**
1. Add `MetricsCollector` trait
2. Implement Prometheus exporter in infrastructure
3. Export key metrics: vector_insert_duration, search_latency, cache_hit_rate

---

### 3. No Provider Health Checks

**Gap:** Admin API has no way to check provider connectivity

**Files:** Admin API handlers

**Impact:**
- Cannot proactively detect provider failures
- Users only discover problems after requests fail
- No way to verify configuration before starting indexing

**Blocker For:**
- Deployment automation
- Health monitoring dashboards
- Graceful degradation

**Implementation:**
1. Add `Health` trait to each provider port
2. Add `/health` endpoint in Admin API
3. Periodic health checks with configurable interval
4. Report unhealthy providers in collection info

---

### 4. No Configuration Migration Guide

**Gap:** No documentation for upgrading configuration between versions

**Files:** Configuration module docs

**Impact:**
- Breaking config changes cause silent failures
- Users unclear on how to update config

**Blocker For:**
- Version upgrades (v0.1.x → v0.2.0)
- Feature adoptions

**Implementation:**
1. Document config schema evolution
2. Provide migration script template
3. Add `--migrate-config` CLI command for automated migrations

---

## Summary by Severity

### 🔴 CRITICAL (Blocks deployment)

1. Architecture dependency violation (mcb-infrastructure → mcb-validate)
2. File lock behavior undefined on Windows
3. Milvus collection readiness race condition

### 🟠 HIGH (Impacts reliability)

1. Duplicate tree-sitter integration (maintenance burden)
2. No centralized language support (extensibility)
3. Metrics infrastructure silos (ecosystem limitation)
4. Different AST patterns prevent code reuse

### 🟡 MEDIUM (Impacts usability)

1. Unsafe string erasure in crypto (use zeroize crate)
2. Hardcoded IP with expect (refactor to constant)
3. Collection mapping fragility (no concurrent testing)
4. Large file analysis not tested (potential issues)

### 🔵 LOW (Nice to have)

1. Config schema validation (deploy better)
2. Provider health checks (monitoring)
3. Metrics export integration (observability)
4. Config migration guides (upgrades)

---

*Concerns audit: 2026-01-31*
