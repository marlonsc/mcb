<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
---
adr: 21
title: Dependency Management Strategy
status: ACCEPTED
created:
updated: 2026-02-05
related: [13, 15, 17]
supersedes: []
superseded_by: []
implementation_status: Incomplete
---

<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->

# ADR 021: Dependency Management Strategy

## Status

> **v0.3.0 Note**: `mcb-application` crate was removed. Use cases moved to `mcb-infrastructure::di::modules::use_cases`.

**Accepted** (v0.2.0 - Implementation)
**Date**: 2026-01-14

## Context

Integrating PMAT adds new dependencies:

- `rayon` - Parallel processing
- `petgraph` - Graph algorithms (DAG analysis)
- `statistical` - Statistical metrics
- `git2` - Git integration
- `cargo_metadata` - Mutation testing

**Current MCB deps**: ~120
**PMAT unique deps**: ~20
**Combined**: ~140 (15% increase)

## Decision

Workspace-level dependency management:

```toml

# Cargo.toml (workspace root)

[workspace]
members = [
    "crates/mcb",
    "crates/mcb-domain",
    "crates/mcb-application",
    "crates/mcb-providers",
    "crates/mcb-infrastructure",
    "crates/mcb-server",
    "crates/mcb-validate",
    # Future
    # "libs/tree-sitter-analysis"
    # "libs/code-metrics"
    # "libs/analysis-core"
]

[workspace.dependencies]

# === EXISTING MCB DEPENDENCIES ===
tokio = { version = "1.49", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
async-trait = "0.1"
thiserror = "1.0"
linkme = "0.3"

# ... existing deps

# === NEW FROM PMAT (added incrementally) ===
rayon = "1.8"                # v0.2.0: Infrastructure
petgraph = "0.6"             # v0.4.0: DAG analysis
statistical = "1.0"          # v0.3.0: Metrics
git2 = "0.19"                # v0.5.0: Git integration
cargo_metadata = "0.20"      # v0.6.0: Mutation testing
```

## Dependency Addition Schedule

**v0.2.0** (Next):

- Add `rayon = "1.8"` (parallel processing infrastructure)
- Add `proptest = "1.0"` (property testing)
- Total new deps: 2

**v0.3.0** (Analysis core):

- Add `statistical = "1.0"` (metrics)
- Add 3 PMAT-specific crates
- Total new deps: 4

**v0.4.0** (Extended analysis):

- Add `petgraph = "0.6"` (DAG)
- Total new deps: 1

**v0.5.0** (Git + Quality):

- Add `git2 = "0.19"` (Git)
- Total new deps: 1

**v0.6.0** (Advanced):

- Add `cargo_metadata = "0.20"` (mutation testing)
- Add `ratatui = "0.29"` (TUI, optional)
- Total new deps: 2

**Total by v1.0.0**: ~140 deps (15% increase from v0.1.0)

## Feature Flags

```toml

# crates/mcb/Cargo.toml

[features]
default = ["search"]

# Core features
search = []                    # v0.1.0: Existing search tools
analysis = []                  # v0.3.0+: Analysis tools
quality = []                   # v0.5.0+: Quality tools
git = []                       # v0.5.0+: Git tools

# Optional features
tui-dashboard = ["ratatui"]   # v0.6.0+: TUI
mutation-testing = ["cargo_metadata"]  # v0.6.0+: Mutation

# Convenience bundles
full = ["search", "analysis", "quality", "git"]
```

## v0.1.1 Workspace Dependencies

Current workspace dependencies are defined in workspace root `Cargo.toml`:

```toml
[workspace.dependencies]

# Core async runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# DI framework (ADR-050; ADR-029 superseded)
linkme = "0.3"

# ... plus ~100 more dependencies
```

## Consequences

Positive:

- Incremental dependency addition
- Feature flags reduce binary size
- Workspace deduplication

Negative:

- Larger dependency tree
- Longer compile times

Mitigation:

- Feature flags for optional deps
- Workspace caching
- CI build matrix

## Related ADRs

- [ADR-013: Clean Architecture Crate Separation](013-clean-architecture-crate-separation.md) - Crate organization
- [ADR-015: Workspace Shared Libraries](015-workspace-shared-libraries.md) - libs/ dependencies
- [ADR-017: Phased Feature Integration](017-phased-feature-integration.md) - Feature timeline

---

Updated 2026-01-17 - Reflects v0.1.2 workspace structure
