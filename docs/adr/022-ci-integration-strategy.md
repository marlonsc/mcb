<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 22
title: Continuous Integration Strategy
status: ACCEPTED
created:
updated: 2026-02-05
related: [13, 17, 20]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 022: Continuous Integration Strategy

## Status

**Accepted** (v0.2.0 - Implementation)
**Date**: 2026-01-14

## Context

Multi-release integration requires robust CI to catch regressions.

## Decision

**Comprehensive CI pipeline** for each release:

### Test Matrix

```yaml

# .github/workflows/ci.yml

strategy:
  matrix:
    rust: [stable, beta]
    os: [ubuntu-latest, macos-latest, windows-latest]
    features:
      -   default
      -   full
      -   search
      -   analysis  # v0.3.0+
```

## Quality Gates

Every PR must pass:

1. `cargo fmt --check` (formatting)
2. `cargo clippy -- -D warnings` (linting)
3. `cargo test --all-features` (all tests)
4. `cargo test --no-default-features` (minimal build)
5. `cargo bench` (no performance regression > 10%)
6. `cargo doc` (documentation builds)

### Benchmark Tracking

Track performance metrics:

- Search latency (target: ≤ 100ms)
- Analysis latency (target: ≤ 500ms/file)
- Memory usage (target: ≤ 300MB)
- Binary size (target: ≤ 80MB)

Alert on regression > 10%

### Version-Specific Gates

**v0.1.1** (Current):

- 308+ tests must pass
- Seven-crate workspace builds
- mcb-validate reports 0 violations

v0.2.0:

- No new features (architectural only)
- All existing tests must pass
- No performance regression

v0.3.0+:

- New feature tests must pass
- Integration tests required
- PMAT tests ported for features

## v0.1.1 CI Status

Current CI workflow in `.github/workflows/`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      -   uses: actions/checkout@v4
      -   uses: dtolnay/rust-toolchain@stable
      -   run: cargo fmt --check
      -   run: cargo clippy -- -D warnings
      -   run: cargo test --all-features
      -   run: cargo run -p mcb-validate
```

Key checks:

- Format (rustfmt)
- Lint (clippy)
- Test (308+ tests)
- Architecture validation (mcb-validate)

## Implementation

**v0.2.0** (Next):

- Update CI to add Rayon tests
- Add workspace build matrix
- Benchmark infrastructure

v0.3.0+:

- Add analysis-specific benchmarks
- Extend test matrix with new features

### Consequences

Positive:

- Catch regressions early
- Performance tracking
- Cross-platform validation

Negative:

- CI time (~10 min per build)
- Matrix explosion with features

Mitigation:

- Parallel jobs
- Caching
- Selective feature testing

## Related ADRs

- [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - What to validate
- [ADR-017: Phased Feature Integration](017-phased-feature-integration.md) - Feature timeline
- [ADR-020: Testing Strategy Integration](020-testing-strategy-integration.md) - Test organization

---

Updated 2026-01-17 - Reflects v0.1.2 CI pipeline
