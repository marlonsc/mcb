<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 15
title: Workspace Structure for Shared Libraries
status: ACCEPTED
created:
updated: 2026-02-05
related: [13, 14]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 015: Workspace Structure for Shared Libraries

## Status

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.


**Accepted** (v0.1.1 - Foundation, v0.3.0 - Full Implementation)
**Date**: 2026-01-14
**Version**: v0.1.1 Update

## Context

Future integration of PMAT code requires shared libraries for:

- Tree-sitter AST parsing (used by search + analysis)
- Code metrics algorithms (complexity, debt scoring)
- Analysis orchestration (parallel processing, caching)

**Question**: Where to place shared code within the seven-crate architecture?

## Decision

Extend the Cargo workspace with shared library crates alongside the seven core crates:

```toml
[workspace]
members = [
    "crates/mcb",                     # Facade crate
    "crates/mcb-domain",              # Domain layer
    "crates/mcb-application",         # Application layer
    "crates/mcb-providers",           # Provider implementations
    "crates/mcb-infrastructure",      # Cross-cutting concerns
    "crates/mcb-server",              # MCP protocol server
    "crates/mcb-validate",            # Architecture validation
    "libs/tree-sitter-analysis",      # AST parsing (v0.3.0)
    "libs/code-metrics",              # Metrics (v0.3.0)
    "libs/analysis-core",             # Orchestration (v0.3.0)
]

[workspace.dependencies]
tokio = { version = "1.49", features = ["full"] }
rayon = "1.8"  # CPU-bound parallelism
tree-sitter = "0.26"

# ... shared version definitions
```

## Library Purposes

### 1. `tree-sitter-analysis` (v0.3.0)

**Purpose**: Unified AST parsing for chunking + analysis

API:

```rust
pub trait LanguageProcessor: Send + Sync {
    // Chunking (existing MCB capability in mcb-providers)
    fn chunk_code(&self, source: &str) -> Result<Vec<CodeChunk>>;

    // Analysis (future capability - v0.3.0+)
    fn validate (action=analyze)(&self, source: &str) -> Result<ComplexityMetrics>;
    fn extract_functions(&self, source: &str) -> Result<Vec<FunctionInfo>>;
}
```

v0.1.1 Status:

- Chunking code lives in `crates/mcb-providers/src/language/`
- 12 language processors implemented
- Will be extracted to this library in v0.3.0

#### 2. `code-metrics` (v0.3.0)

**Purpose**: Complexity/debt algorithms from PMAT

**API** (defined in v0.1.1, implemented in v0.3.0):

```rust
pub trait MetricsCalculator: Send + Sync {
    fn calculate_complexity(&self, ast: &ParsedCode) -> ComplexityMetrics;
    fn calculate_tdg_score(&self, metrics: &AggregateMetrics) -> TdgScore;
}
```

#### 3. `analysis-core` (v0.3.0)

**Purpose**: Orchestration utilities for parallel analysis

## Consequences

Positive:

- Code reuse between domains
- Independent versioning possible
- Easier to extract as separate crates later
- Clear API boundaries

Negative:

- Workspace compilation overhead
- Dependency management complexity

Mitigation:

- Use `workspace = true` for shared deps
- Keep libraries focused and small

## Implementation Plan

### v0.1.1 (Current - Foundation)

- Seven-crate workspace structure implemented
- Language chunking in `crates/mcb-providers/src/language/`
- Workspace dependencies defined

### v0.3.0 (Future - Full Implementation)

1. Create `libs/tree-sitter-analysis/`
2. Create `libs/code-metrics/`
3. Create `libs/analysis-core/`
4. Extract existing chunking code to library
5. Port PMAT algorithms

### v0.5.0 (Future)

1. Consider extracting git utilities to `libs/git-analysis/`

## Related ADRs

- [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Seven-crate foundation
- [ADR-014: Multi-Domain Architecture](014-multi-domain-architecture.md) - Domain organization

---

Updated 2026-02-15 - Reflects v0.2.1 seven-crate workspace
