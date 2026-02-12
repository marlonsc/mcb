# git2

Last updated: 2026-02-12

## Executive Summary

Git2 is MCB's VCS engine implementation for repository opening, branch/history traversal, tree walking, and diff-state analysis behind the `VcsProvider` abstraction.

## Context7 + External Research

- Context7 ID: `/websites/docs_rs-git2`
- API docs: https://docs.rs/git2/latest/git2/
- Upstream: https://github.com/rust-lang/git2-rs
- libgit2 docs: https://libgit2.org/libgit2/

## Actual MCB Usage (Current Source of Truth)

### 1) Provider implementation and repository operations

- `crates/mcb-providers/src/git/git2_provider.rs:6`
- `crates/mcb-providers/src/git/git2_provider.rs:17`
- `crates/mcb-providers/src/git/git2_provider.rs:93`

Pattern: `Git2Provider` encapsulates git2 APIs and implements domain `VcsProvider` behavior.

### 2) Tree walking and diff mapping

- `crates/mcb-providers/src/git/git2_provider.rs:243`
- `crates/mcb-providers/src/git/git2_provider.rs:340`

Pattern: tree traversal and delta conversion map git-native values to domain enums.

### 3) Submodule and blocking boundary handling

- `crates/mcb-providers/src/git/submodule.rs:6`
- `crates/mcb-providers/src/git/submodule.rs:48`

Pattern: blocking libgit2 calls are isolated from async runtime with explicit offloading.

### 4) DI registration and service exposure

- `crates/mcb-infrastructure/src/di/vcs.rs:9`
- `crates/mcb-infrastructure/src/di/vcs.rs:15`

Pattern: provider is wired into infrastructure DI and consumed via domain ports.

## ADR Alignment (Critical)

- ADR-008 (`docs/adr/008-git-aware-semantic-indexing-v0.2.0.md`): git2 selected for git-aware indexing.
- ADR-035 (`docs/adr/035-context-scout.md`): git2 is required for context discovery and should stay behind `VcsProvider`.
- ADR-038 (`docs/adr/038-multi-tier-execution-model.md`): git2 powers VCS operations in multi-tier workflows.

## GitHub Evidence (Upstream + In-Repo)

- Upstream git2-rs: https://github.com/rust-lang/git2-rs
- Cargo production usage: https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs
- In-repo anchor: `crates/mcb-providers/src/git/git2_provider.rs:254`
- In-repo anchor: `crates/mcb-providers/src/git/git2_provider.rs:343`

## Common Pitfalls

- Treating git2 objects as async/thread-safe without explicit ownership strategy.
- Repeating expensive repo scans without scoping or caching.
- Assuming branch names instead of deriving from repository HEAD/default branch.

## References

- https://docs.rs/git2/latest/git2/
- https://github.com/rust-lang/git2-rs
- https://libgit2.org/libgit2/
- `docs/adr/008-git-aware-semantic-indexing-v0.2.0.md`
- `docs/adr/035-context-scout.md`
