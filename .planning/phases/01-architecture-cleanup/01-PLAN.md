# Phase 1: Architecture Cleanup - Execution Plan

**Created:** 2026-01-31
**Status:** PENDING
**Estimated LOC:** ~1500

## Objective

Break dependency cycles and create shared infrastructure crates. This phase reorganizes internal architecture without adding user-facing features.

## Prerequisites

- [x] v0.1.5 release completed
- [x] All tests passing (1670+)
- [x] 0 architecture violations
- [x] Context gathered (01-CONTEXT.md)

## Tasks

### Plan 01: Create mcb-language-support crate

**Goal:** Unified language abstraction built on Mozilla RCA fork

**Tasks:**
- [ ] Create `crates/mcb-language-support/` directory structure
- [ ] Add to workspace `Cargo.toml` members
- [ ] Add `rust-code-analysis` as dependency (Mozilla fork)
- [ ] Create `LanguageId` enum (13 supported languages)
- [ ] Create `LanguageRegistry` for language lookup
- [ ] Create `LanguageDetector` (move from mcb-validate/src/filters/language_detector.rs)
- [ ] Create `ParsedFile` struct for AST results
- [ ] Create async `Parser` trait with RCA implementation
- [ ] Create `ChunkingStrategy` trait for language-specific chunking
- [ ] Add comprehensive tests for all languages
- [ ] Verify `make test` passes

**Files to create:**
- `crates/mcb-language-support/Cargo.toml`
- `crates/mcb-language-support/src/lib.rs`
- `crates/mcb-language-support/src/language.rs` (LanguageId, LanguageRegistry)
- `crates/mcb-language-support/src/detection.rs` (LanguageDetector)
- `crates/mcb-language-support/src/parser.rs` (Parser trait, RcaParser)
- `crates/mcb-language-support/src/chunking.rs` (ChunkingStrategy)
- `crates/mcb-language-support/src/error.rs`

**Exit criteria:** New crate compiles, tests pass, `LanguageDetector` moved from mcb-validate

---

### Plan 02: Create mcb-ast-utils crate

**Goal:** AST traversal and analysis utilities

**Tasks:**
- [ ] Create `crates/mcb-ast-utils/` directory structure
- [ ] Add to workspace `Cargo.toml` members
- [ ] Add dependency on mcb-language-support
- [ ] Create `TreeWalker` for AST traversal
- [ ] Create `NodeVisitor` trait for visitor pattern
- [ ] Create `CursorUtils` for tree-sitter cursor operations
- [ ] Create `SymbolExtractor` for extracting function/class names
- [ ] Move complexity metrics from mcb-validate (src/metrics/)
- [ ] Add tests for traversal utilities
- [ ] Verify `make test` passes

**Files to create:**
- `crates/mcb-ast-utils/Cargo.toml`
- `crates/mcb-ast-utils/src/lib.rs`
- `crates/mcb-ast-utils/src/walker.rs` (TreeWalker)
- `crates/mcb-ast-utils/src/visitor.rs` (NodeVisitor)
- `crates/mcb-ast-utils/src/cursor.rs` (CursorUtils)
- `crates/mcb-ast-utils/src/symbols.rs` (SymbolExtractor)
- `crates/mcb-ast-utils/src/complexity.rs` (moved from mcb-validate)
- `crates/mcb-ast-utils/src/error.rs`

**Exit criteria:** New crate compiles, tests pass, complexity metrics moved

---

### Plan 03: Update mcb-validate to use shared crates

**Goal:** Replace inline language/AST code with shared crate dependencies

**Tasks:**
- [ ] Add dependencies on mcb-language-support and mcb-ast-utils
- [ ] Replace `LanguageDetector` usage with mcb-language-support
- [ ] Replace complexity metrics with mcb-ast-utils
- [ ] Update `src/filters/language_detector.rs` to re-export from mcb-language-support
- [ ] Update `src/metrics/` to use mcb-ast-utils
- [ ] Remove duplicated code
- [ ] Update tests to use new imports
- [ ] Verify `make test` passes
- [ ] Verify `make validate` passes

**Files to modify:**
- `crates/mcb-validate/Cargo.toml`
- `crates/mcb-validate/src/filters/language_detector.rs`
- `crates/mcb-validate/src/metrics/mod.rs`
- `crates/mcb-validate/src/metrics/rca_analyzer.rs`

**Exit criteria:** mcb-validate uses shared crates, no duplicate code, all tests pass

---

### Plan 04: Define MetricsProvider port

**Goal:** Create generic metrics port for Prometheus/OpenTelemetry integration

**Tasks:**
- [ ] Create `MetricsProvider` trait in mcb-domain/src/ports/providers/
- [ ] Define generic primitives: `increment()`, `gauge()`, `histogram()`
- [ ] Define domain convenience: `record_index_time()`, `record_search_latency()`
- [ ] Support labels/tags per Prometheus patterns
- [ ] Create `NullMetricsProvider` implementation
- [ ] Add trait to `mod.rs` exports
- [ ] Verify `make test` passes

**Files to create/modify:**
- `crates/mcb-domain/src/ports/providers/metrics_provider.rs` (new)
- `crates/mcb-domain/src/ports/providers/mod.rs` (add export)

**Exit criteria:** MetricsProvider trait defined, NullMetricsProvider works, tests pass

---

### Plan 05: Integration and validation

**Goal:** Ensure all changes integrate correctly

**Tasks:**
- [ ] Run `make test` - all 1670+ tests pass
- [ ] Run `make lint` - no warnings
- [ ] Run `make validate` - 0 violations
- [ ] Update dependency graph documentation
- [ ] Verify no mcb-infrastructure → mcb-validate dependency
- [ ] Create feature branch and commit

**Exit criteria:** All quality gates pass, dependency cycle broken, ready for Phase 2

---

## Progress Tracking

- [ ] Plan 01: Create mcb-language-support crate
- [ ] Plan 02: Create mcb-ast-utils crate
- [ ] Plan 03: Update mcb-validate to use shared crates
- [ ] Plan 04: Define MetricsProvider port
- [ ] Plan 05: Integration and validation

**Total Tasks:** 5 | **Completed:** 0 | **Remaining:** 5

## Success Criteria

1. mcb-validate uses mcb-language-support for language detection
2. mcb-validate uses mcb-ast-utils for complexity analysis
3. No duplicate language/AST code across crates
4. MetricsProvider trait defined with Prometheus-compatible API
5. All 1670+ tests pass
6. `make validate` shows 0 violations
7. Dependency cycle broken (mcb-infrastructure does not depend on mcb-validate)

## Notes

- **Migration strategy:** Big bang - all code moves in single commit per plan
- **Breaking changes:** Internal APIs can break freely (no backwards compatibility needed)
- **RCA dependency:** Using Mozilla's rust-code-analysis from crates.io
- **Existing ValidationProvider:** Already exists in mcb-domain/src/ports/providers/validation.rs - no changes needed

---
*Phase: 01-architecture-cleanup*
*Plan created: 2026-01-31*
