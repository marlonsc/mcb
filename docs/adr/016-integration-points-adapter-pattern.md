---
adr: 16
title: Integration Points and Adapter Pattern
status: ACCEPTED
created:
updated: 2026-02-05
related: [13, 15, 19]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 016: Integration Points and Adapter Pattern

## Status

**Accepted** (v0.2.0 - Preparation)
**Date**: 2026-01-14
**Version**: v0.2.0 (Preparation)

## Context

Integrating PMAT code (proven algorithms, extensive tests) while maintaining MCB's clean architecture.

**Challenge**: PMAT and MCB have different type systems, error handling, and patterns.

## Decision

Use**Adapter Pattern** with thin conversion layer:

```text
PMAT Algorithm (reused 100%)
    ↓
Adapter (thin wrapper, ~10-50 LOC)
    ↓
MCB Service (uses adapter)
    ↓
MCB Handler (MCP protocol)
```

### Example: Complexity Analyzer

**PMAT Original** (100% reused):

```rust
// libs/code-metrics/src/complexity/analyzer.rs
// DIRECT COPY from PMAT

pub struct ComplexityAnalyzer {
    // PMAT's implementation unchanged
}

impl ComplexityAnalyzer {
    pub fn analyze_file(&self, path: &Path) -> anyhow::Result<FunctionComplexity> {
        // PMAT's algorithm - ZERO CHANGES
    }
}
```

**MCB Adapter** (thin wrapper):

```rust
// crates/mcb-providers/src/analyzers/complexity_adapter.rs

use code_metrics::complexity::ComplexityAnalyzer;

pub struct ComplexityAnalyzerAdapter {
    inner: ComplexityAnalyzer,  // Wrap PMAT code
}

impl ComplexityAnalyzerAdapter {
    pub async fn analyze(&self, path: &Path) -> Result<ComplexityReport> {
        // 1. Call PMAT analyzer (blocking, CPU-bound)
        let pmat_result = tokio::task::spawn_blocking(move || {
            self.inner.analyze_file(path)
        }).await??;

        // 2. Convert PMAT type → MCB type
        Ok(ComplexityReport::from_pmat(pmat_result))
    }
}
```

**MCB Service** (uses adapter):

```rust
// crates/mcb-application/src/use_cases/analysis/complexity.rs

#[derive(Component)]
#[shaku(interface = ComplexityAnalysisInterface)]
pub struct ComplexityAnalysisService {
    adapter: Arc<ComplexityAnalyzerAdapter>,

    #[shaku(inject)]
    cache: Arc<dyn CacheProvider>,

    #[shaku(inject)]
    event_bus: Arc<dyn EventBusProvider>,
}

impl ComplexityAnalysisInterface for ComplexityAnalysisService {
    async fn analyze_file(&self, path: &Path) -> Result<ComplexityReport> {
        // Check cache
        if let Some(cached) = self.cache.get(path).await? {
            return Ok(cached);
        }

        // Use adapter
        let report = self.adapter.analyze(path).await?;

        // Cache + publish event
        self.cache.set(path, &report).await?;
        self.event_bus.publish(SystemEvent::ComplexityAnalyzed { path }).await?;

        Ok(report)
    }
}
```

### Adapter Responsibilities

1. **Type Conversion**: PMAT types ↔ MCB types
2. **Error Translation**: anyhow::Error → MCB's typed errors
3. **Async Bridging**: Wrap blocking PMAT code in `spawn_blocking`
4. **Minimal Logic**: <50 LOC, pure translation

### Integration Point Definition

**v0.2.0** (This release):

- Define adapter interfaces
- Create empty adapter directory structure
- Document expected PMAT → MCB conversions

```rust
// crates/mcb-providers/src/analyzers/mod.rs (v0.2.0)

// Adapter interface for future analysis tools
pub trait AnalysisAdapter: Send + Sync {
    type PmatType;   // PMAT's output type
    type McbType;    // MCB's domain type

    async fn execute(&self, input: &Path) -> Result<Self::McbType>;
}

// Future adapters will implement this
// pub struct ComplexityAnalyzerAdapter;  // v0.3.0
// pub struct TdgScorerAdapter;           // v0.3.0
// pub struct SatdDetectorAdapter;        // v0.3.0
```

**v0.3.0** (Implementation):

- Implement adapters for complexity, TDG, SATD
- Port PMAT code to `libs/code-metrics/`

## Consequences

Positive:

- 100% PMAT algorithm reuse (no reimplementation risk)
- Clean architecture preserved
- Type safety via adapter contracts
- Easy to add new adapters

Negative:

- Indirection overhead (~1-2ms per call)
- Two type systems to maintain

Mitigation:

- Keep adapters thin (target <50 LOC)
- Use inline conversions where possible
- Benchmark to ensure <1% overhead

## Implementation Checklist (v0.2.0)

- [ ] Create `crates/mcb-providers/src/analyzers/` directory
- [ ] Define `AnalysisAdapter` trait
- [ ] Document conversion patterns
- [ ] Create adapter templates for v0.3.0

## Related ADRs

- [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate organization
- [ADR-015: Workspace Shared Libraries](015-workspace-shared-libraries.md) - PMAT code location
- [ADR-019: Error Handling Strategy](019-error-handling-strategy.md) - Error conversion patterns

---

Updated 2026-01-17 - Reflects v0.1.2 crate paths
