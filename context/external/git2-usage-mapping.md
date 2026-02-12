# Git2 Library - Internal Usage Mapping

**Library**: `git2` (libgit2 bindings, v0.13.x)  
**Status**: IMPLEMENTED (v0.2.0)  
**ADR Reference**: [ADR-008: Git-Aware Semantic Indexing v0.2.0](../../docs/adr/008-git-aware-semantic-indexing-v0.2.0.md)  
**Purpose**: Repository operations, branch management, diff analysis, and submodule traversal

## Architecture Overview

Git2 is integrated as the **VCS provider** in MCB's unified provider architecture. It implements the `VcsProvider` trait from the domain layer and provides git-aware indexing capabilities for semantic search across branches and commits.

### Design Pattern
- **Pattern**: Trait-based Provider + Async Wrapper
- **Scope**: Repository operations (branches, commits, diffs, submodules)
- **Blocking**: Uses `tokio::task::spawn_blocking` for git2 (not async-safe)

---

## Core Implementation

### 1. Provider Implementation
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/git2_provider.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1 | `//! Git2-based implementation of VcsProvider` | Module documentation |
| 6 | `use git2::{BranchType, Repository, Sort};` | Core git2 imports |
| 16-17 | `pub struct Git2Provider;` | Zero-sized provider struct |
| 19-24 | `Git2Provider::new()` | Constructor |
| 26-34 | `open_repo()` | Opens repository with error handling for NotFound |
| 28 | `e.code() == git2::ErrorCode::NotFound` | Error code matching |
| 36-53 | `get_root_commit_hash()` | Retrieves first commit via revwalk |
| 37-45 | `repo.revwalk()` + `push_head()` + `set_sorting()` | Commit history traversal |
| 55-60 | `get_default_branch()` | Gets HEAD shorthand (main/master) |
| 62-66 | `get_remote_url()` | Retrieves origin remote URL |
| 68-83 | `list_branch_names()` | Enumerates local branches |
| 70 | `repo.branches(Some(BranchType::Local))` | Branch enumeration |
| 93-149 | `VcsProvider` trait impl | Async methods for repository operations |

### 2. Repository Operations
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/git2_provider.rs:150-300`

| Line | Component | Purpose |
|------|-----------|---------|
| 150-180 | `open_repository()` | Opens repo and returns VcsRepository metadata |
| 182-230 | `list_branches()` | Lists all branches with commit info |
| 232-267 | `list_files()` | Walks tree and collects file paths |
| 243 | `tree.walk(git2::TreeWalkMode::PreOrder, ...)` | Tree traversal |
| 244 | `entry.kind() == Some(git2::ObjectType::Blob)` | File filtering |
| 254 | `git2::TreeWalkResult::Ok` | Walk result handling |
| 269-310 | `get_diff()` | Compares two refs and returns file diffs |
| 340-343 | Delta mapping | Maps `git2::Delta::*` to `DiffStatus` enum |

### 3. Submodule Traversal Service
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/submodule.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1 | `//! Submodule traversal service using git2.` | Module documentation |
| 6 | `use git2::Repository;` | Repository import |
| 41 | `// Use spawn_blocking for git2 operations (not async-safe)` | Async safety note |
| (See full file) | Submodule detection and recursive indexing | Monorepo support |
| 81 | `tracing::debug!(...)` | Debug logging for submodule discovery |
| 92 | `tracing::warn!(error = %e, ...)` | Error logging for failed submodule listing |

### 4. Module Structure
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/mod.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 8 | `mod git2_provider;` | Module declaration |
| 12 | `pub use git2_provider::Git2Provider;` | Public re-export |

---

## Dependency Injection & Registration

### 5. DI Factory
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/vcs.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1-4 | Module documentation | VCS provider factory |
| 8-9 | Imports | `VcsProvider` trait and `Git2Provider` |
| 14-16 | `default_vcs_provider()` | Factory function returning `Arc<dyn VcsProvider>` |
| 15 | `Arc::new(Git2Provider::new())` | Git2 instantiation |

### 6. DI Bootstrap
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/bootstrap.rs:14,94,206`

| Line | Component | Purpose |
|------|-----------|---------|
| 14 | `use mcb_domain::ports::providers::VcsProvider;` | Trait import |
| 94 | `vcs_provider: Arc<dyn VcsProvider>,` | Provider field in AppContext |
| 206 | `pub fn vcs_provider(&self) -> Arc<dyn VcsProvider>` | Accessor method |

### 7. Domain Port Definition
**File**: `/home/marlonsc/mcb/crates/mcb-domain/src/ports/providers/vcs.rs:16`

| Line | Component | Purpose |
|------|-----------|---------|
| 16 | `pub trait VcsProvider: Send + Sync` | Trait definition |

---

## Runtime Usage & Observability

### 8. VCS Handler Integration
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/vcs/handler.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 5 | `use mcb_domain::ports::providers::VcsProvider;` | Trait import |
| 19 | `vcs_provider: Arc<dyn VcsProvider>,` | Dependency injection |
| 24 | `pub fn new(vcs_provider: Arc<dyn VcsProvider>) -> Self` | Constructor |

### 9. VCS Handler Methods
**File**: `/home/marlonsc/mcb/crates/mcb-server/src/handlers/vcs/`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `search_branch.rs` | 14 | `#[tracing::instrument(skip_all)]` | Instrumentation macro |
| `search_branch.rs` | 16 | `vcs_provider: &Arc<dyn VcsProvider>` | Parameter |
| `list_repos.rs` | 13 | `#[tracing::instrument(skip_all)]` | Instrumentation |
| `list_repos.rs` | 15 | `vcs_provider: &Arc<dyn VcsProvider>` | Parameter |
| `index_repo.rs` | 15 | `#[tracing::instrument(skip_all)]` | Instrumentation |
| `index_repo.rs` | 17 | `vcs_provider: &Arc<dyn VcsProvider>` | Parameter |
| `compare_branches.rs` | 14 | `#[tracing::instrument(skip_all)]` | Instrumentation |
| `compare_branches.rs` | 16 | `vcs_provider: &Arc<dyn VcsProvider>` | Parameter |
| `analyze_impact.rs` | 14 | `#[tracing::instrument(skip_all)]` | Instrumentation |
| `analyze_impact.rs` | 16 | `vcs_provider: &Arc<dyn VcsProvider>` | Parameter |

### 10. Observability Hooks
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/submodule.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 81 | `tracing::debug!(...)` | Debug logging for submodule discovery |
| 92 | `tracing::warn!(error = %e, ...)` | Warning for failed submodule listing |
| 106 | `tracing::warn!(...)` | Warning for submodule initialization |
| 118 | `tracing::warn!(...)` | Warning for submodule update |
| 145 | `tracing::debug!(...)` | Debug logging for recursive traversal |
| 175 | `tracing::warn!(...)` | Warning for submodule path issues |
| 187 | `tracing::info!(...)` | Info logging for successful operations |

---

## Testing & Validation

### 11. Unit Tests - Provider Construction
**File**: `/home/marlonsc/mcb/crates/mcb-providers/tests/unit/git2_provider_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 45 | `fn test_git2_provider_constructs()` | Basic construction test |
| 46 | `let provider = Git2Provider::new();` | Instantiation |
| 55 | `fn test_git2_provider_is_object_safe()` | Object safety test |
| 56 | `fn _assert_object_safe(_: &dyn VcsProvider) {}` | Trait object validation |
| 59 | `let _erased: &dyn VcsProvider = &provider;` | Trait object casting |

### 12. Integration Tests - Submodule Handling
**File**: `/home/marlonsc/mcb/crates/mcb-providers/tests/unit/submodule_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1 | `use git2::Repository;` | Git2 import |
| 11 | `let sig = git2::Signature::now(...)` | Signature creation |
| 25-35 | Submodule test data | Test fixture with tree-sitter submodule |

### 13. Server Integration Tests
**File**: `/home/marlonsc/mcb/crates/mcb-server/tests/handlers/vcs_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 7 | `use crate::test_utils::mock_services::MockVcsProvider;` | Mock provider |
| 11 | `let mock_provider = MockVcsProvider::new();` | Mock instantiation |

### 14. Mock Provider
**File**: `/home/marlonsc/mcb/crates/mcb-server/tests/test_utils/mock_services/vcs.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 11 | `use mcb_domain::ports::providers::VcsProvider;` | Trait import |
| 14 | `pub struct MockVcsProvider` | Mock implementation |
| 38 | `impl VcsProvider for MockVcsProvider` | Trait implementation |

---

## Cargo.toml Dependencies

### 15. Dependency Declaration
**File**: `/home/marlonsc/mcb/crates/mcb-providers/Cargo.toml:80`

```toml
git2 = { workspace = true }
```

**Workspace Definition**: `/home/marlonsc/Cargo.toml`
- Version: 0.13.x
- Features: Default (includes libgit2 bindings)

---

## ADR Alignment

### ADR-008: Git-Aware Semantic Indexing v0.2.0
- **Status**: IMPLEMENTED
- **Rationale**:
  - Git2 chosen for mature, battle-tested libgit2 bindings
  - Superior performance to gitoxide (still in development)
  - Stable API with comprehensive documentation
  - Supports all required operations: branches, commits, diffs, submodules
- **Key Features**:
  - Repository identification by root commit (portable)
  - Multi-branch indexing (main + HEAD + current)
  - Commit history traversal (last 50 by default)
  - Submodule detection with recursive indexing
  - Project detection in monorepos
  - Impact analysis between commits/branches
- **Trade-offs**:
  - Not async-safe (requires `spawn_blocking`)
  - External libgit2 dependency (C library)
  - Larger binary size

### ADR-003: Unified Provider Architecture
- **Alignment**: Git2Provider implements `VcsProvider` trait
- **Registration**: Direct instantiation in DI factory (not linkme)
- **Lifecycle**: Managed by `AppContext` in bootstrap

### ADR-002: Async-First Architecture
- **Pattern**: Async trait with blocking operations
- **Implementation**: `tokio::task::spawn_blocking` for git2 calls

---

## Error Handling

### Error Code Matching
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/git/git2_provider.rs`

| Line | Error Code | Handling |
|------|-----------|----------|
| 28 | `git2::ErrorCode::NotFound` | Maps to `Error::repository_not_found()` |
| 170 | `git2::ErrorCode::NotFound` | Maps to `Error::repository_not_found()` |
| 230 | `git2::ErrorCode::NotFound` | Maps to `Error::repository_not_found()` |

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Core Impl** | `/home/marlonsc/mcb/crates/mcb-providers/src/git/git2_provider.rs:1-350` |
| **Submodule Support** | `/home/marlonsc/mcb/crates/mcb-providers/src/git/submodule.rs` |
| **DI Factory** | `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/vcs.rs:14-16` |
| **Bootstrap** | `/home/marlonsc/mcb/crates/mcb-infrastructure/src/di/bootstrap.rs:94,206` |
| **Handlers** | 5 VCS handler files with `#[tracing::instrument]` |
| **Observability** | Tracing via submodule.rs, error logging in handlers |
| **Tests** | 3+ test files validating provider, submodules, and integration |
| **ADR** | ADR-008 (primary), ADR-003, ADR-002 |
| **Status** | IMPLEMENTED, production-ready |

