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

## Best Practices in MCB

### Provider abstraction

All git2 usage is encapsulated behind `VcsProvider` (domain port) and `Git2Provider` (adapter). No crate outside `mcb-providers` should import `git2` directly. This ensures the VCS implementation remains swappable and the domain stays clean.

Cross-reference: `context/external/async-trait.md` for port/adapter pattern, `context/external/dill.md` for DI registration.

### Blocking boundary isolation

libgit2 is fundamentally synchronous. MCB isolates all git2 calls from the async runtime using `tokio::task::spawn_blocking`. This is mandatory — calling git2 directly from an async context starves Tokio worker threads.

Key isolation points:
- `crates/mcb-providers/src/git/submodule.rs:48` (explicit spawn_blocking)
- `crates/mcb-providers/src/git/git2_provider.rs` (provider methods delegate through blocking boundaries)

Cross-reference: `context/external/tokio.md` (spawn_blocking discipline).

### Repository open strategy

MCB opens repositories per-operation rather than holding long-lived `Repository` handles. This avoids lock contention and stale index issues. The `Git2Provider` constructs a fresh `Repository::open()` for each VCS operation.

### Branch resolution

Never hardcode branch names ("main", "master"). MCB resolves the default branch from `HEAD` reference or repository configuration. See `crates/mcb-providers/src/git/git2_provider.rs:93` for the resolution pattern.

## Performance and Safety Considerations

### Large repository handling

For repositories with deep history or many files:
- Limit traversal depth with explicit walk boundaries
- Use `TreeWalkMode::PreOrder` with early exit when possible
- Cache tree walk results when repeated access is needed

MCB currently walks trees for indexing in `crates/mcb-providers/src/git/git2_provider.rs:243`. For very large monorepos, consider scoping to changed paths only.

### Object lifetime and memory

git2 objects (`Tree`, `Commit`, `Blob`) are tied to the `Repository` lifetime. Do not attempt to return git2 objects from spawned blocking tasks — convert to owned domain types first.

MCB's `Git2Provider` converts all git2 results to domain value objects (`BranchInfo`, `CommitInfo`, `FileStatus`) before returning from the provider boundary.

### Thread safety

`git2::Repository` is `Send` but not `Sync`. Each `spawn_blocking` closure should own its `Repository` instance. Sharing a repository across tasks requires a mutex, which MCB avoids by opening per-operation.

## Testing and Verification Guidance

### Test repository fixtures

MCB uses actual git repositories in test fixtures for VCS operations. Tests that need git state should create temporary repositories with known commits rather than mocking the git2 API.

### Mock provider for unit tests

For handler and service tests that don't need real VCS, MCB provides `MockVcsProvider` (`crates/mcb-server/tests/test_utils/mock_services/vcs.rs`). This mock implements the domain `VcsProvider` port and can be configured to return canned responses or simulate failures.

### Integration test discipline

Integration tests for git operations should:
1. Create a temporary directory with `tempfile::tempdir()`
2. Initialize a git repository with known state
3. Run provider operations against it
4. Assert on domain-level results, not git2 internals

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Blocking git2 call on async thread | Worker thread starvation | Enforce spawn_blocking in all provider methods |
| Stale repository index | Incorrect diff/status results | Open repository per-operation, don't cache handles |
| Large tree walk without depth limit | Memory exhaustion, timeout | Scope walks to changed paths or limit depth |
| Missing libgit2 system dependency | Build failure on CI/deploy | Document libgit2 requirement in build prerequisites |
| Git lock contention | Operation failure | Per-operation repo open avoids long-held locks |

Cross-reference: `context/external/tracing.md` for instrumenting VCS operation latency.

## Migration and Version Notes

- MCB uses git2 (Rust bindings for libgit2).
- git2-rs tracks libgit2 versions; major libgit2 updates may require Cargo dependency bumps.
- ADR-008 (`docs/adr/008-git-aware-semantic-indexing-v0.2.0.md`) selected git2 for semantic indexing.
- ADR-035 (`docs/adr/035-context-scout.md`) confirms git2 remains the VCS engine behind `VcsProvider`.
- No migration planned. If gitoxide becomes stable and offers significant advantages, it would replace git2 behind the same `VcsProvider` abstraction.

## Verification Checklist

- [ ] All git2 calls wrapped in `spawn_blocking` (no direct calls from async context)
- [ ] Repository opened per-operation, not cached long-term
- [ ] Branch resolution derives from HEAD, not hardcoded names
- [ ] Domain types returned from provider (no git2 types leak through port boundary)
- [ ] Large tree walks bounded by depth or path scope
- [ ] Mock provider used for non-VCS tests (`MockVcsProvider`)
- [ ] Integration tests use temporary repositories with known state

## Common Pitfalls

- Treating git2 objects as async/thread-safe without explicit ownership strategy.
- Repeating expensive repo scans without scoping or caching.
- Assuming branch names instead of deriving from repository HEAD/default branch.
- Returning git2 types across the provider boundary (leaks implementation detail).
- Holding `Repository` handles across async `.await` points.

## References

- https://docs.rs/git2/latest/git2/
- https://github.com/rust-lang/git2-rs
- https://libgit2.org/libgit2/
- `docs/adr/008-git-aware-semantic-indexing-v0.2.0.md`
- `docs/adr/035-context-scout.md`
- `docs/adr/038-multi-tier-execution-model.md`
- `context/external/tokio.md`
- `context/external/async-trait.md`
- `context/external/dill.md`
- `context/external/tracing.md`
