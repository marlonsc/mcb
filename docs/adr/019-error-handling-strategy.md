---
adr: 19
title: Error Handling Strategy
status: ACCEPTED
created: 
updated: 2026-02-05
related: [13, 16]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

# ADR 019: Error Handling Strategy

## Status

**Accepted** (v0.2.0 - Implementation)
**Date**: 2026-01-14

## Context

MCB uses typed errors (`thiserror`). PMAT uses `anyhow::Error`.

**Question**: How to handle errors when integrating PMAT code?

## Decision

**Layered error handling**:

1.  **Domain Layer**: Typed errors with `thiserror`
2.  **Adapter Layer**: Convert `anyhow` → typed errors
3.  **PMAT Libraries**: Keep `anyhow` (unchanged)

### Example

**PMAT Code** (unchanged):

```rust
// libs/code-metrics/src/complexity/analyzer.rs
pub fn analyze_file(&self, path: &Path) -> anyhow::Result<FunctionComplexity> {
    // PMAT's implementation with anyhow
}
```

**Adapter Layer** (conversion):

```rust
// crates/mcb-providers/src/analyzers/complexity_adapter.rs

impl ComplexityAnalyzerAdapter {
    pub async fn analyze(&self, path: &Path) -> Result<ComplexityReport> {
        self.inner.analyze_file(path)
            .map(ComplexityReport::from_pmat)
            .map_err(|e| AnalysisError::ComplexityAnalysisFailed {
                path: path.to_path_buf(),
                source: e,  // Wrap anyhow error
            })
    }
}
```

**Domain Error Types**:

```rust
// crates/mcb-domain/src/error.rs

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Complexity analysis failed for {path}: {source}")]
    ComplexityAnalysisFailed {
        path: PathBuf,
        #[source]
        source: anyhow::Error,
    },

    #[error("TDG scoring failed: {source}")]
    TdgScoringFailed {
        #[source]
        source: anyhow::Error,
    },

    // ... more variants
}
```

## v0.1.1 Error Organization

Current error types in the eight-crate structure:

| Crate | Error File | Error Type |
|-------|-----------|------------|
| mcb-domain | `src/error.rs` | `DomainError` |
| mcb-application | `src/error.rs` | `ApplicationError` |
| mcb-providers | `src/error.rs` | `ProviderError` |
| mcb-infrastructure | `src/error.rs` | `InfrastructureError` |
| mcb-server | `src/error.rs` | `ServerError` |

All use `thiserror` for type-safe error handling.

## Consequences

**Positive**:

-   PMAT code unchanged (100% reuse)
-   MCB's typed errors at boundaries
-   Error context preserved

**Negative**:

-   Two error types to maintain

**Acceptable**: Adapter layer is thin (<50 LOC per adapter)

## Related ADRs

-   [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Error location per crate
-   [ADR-016: Integration Points Adapter Pattern](016-integration-points-adapter-pattern.md) - Adapter pattern

---

*Updated 2026-01-17 - Reflects v0.1.2 crate structure*
