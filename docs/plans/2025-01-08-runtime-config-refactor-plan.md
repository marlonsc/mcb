# Runtime Configuration System Refactoring Plan (Incremental & SOLID)

> **WARNING: DO NOT use the built-in `ExitPlanMode` or `EnterPlanMode` tools.**
> This project has its own planning workflow using `/plan`, `/implement`, and `/verify` slash commands.
> The built-in Claude Code plan mode tools write to different paths and are incompatible.
> When planning is complete, simply inform the user and wait for confirmation to proceed.

Created: 2025-01-08
Status: PENDING
Priority: HIGH - Technical Debt Reduction & Foundation

## Executive Summary

This plan executes a **massive code reduction** and architectural modernization of the configuration system. By adopting the `config` crate's native features and `shaku` for dependency injection, we will eliminate ~40% of the codebase's boilerplate (manual parsing, merging logic, and wiring).

**Key Goals:**
1.  **Reduce Code Volume**: Replace 400+ lines of manual config logic with `config` library features.
2.  **SOLID Principles**: Break `src/config.rs` (1300+ lines) into focused, single-responsibility modules.
3.  **Strict Validation**: Every step includes mandatory validation to ensure zero regressions.

## Architecture Vision

### Before
- **Monolithic**: `src/config.rs` handles types, loading, merging, validation, and printing.
- **Manual**: Environment variables are manually parsed and merged.
- **Rigid**: Dependencies are manually wired in `McpServer::new`.

### After
- **Modular**: `src/config/` directory with `server.rs`, `providers.rs`, `metrics.rs`.
- **Declarative**: `config` crate handles loading/merging; `validator` handles validation.
- **Injected**: `shaku` handles dependency resolution.

## Implementation Phases

### Phase 1: Foundation Dependencies (1 Day)

**Objective:** Add necessary libraries and verify compatibility without changing logic.

**Files:**
- Modify: `Cargo.toml`
- Test: `make build`

**Implementation Steps:**
1.  Add `shaku = "0.6"` (DI).
2.  Add `validator = { version = "0.16", features = ["derive"] }` (Validation).
3.  Add `notify = "6.1"` (Hot Reload).
4.  Add `arc-swap = "1.7"` (Concurrent Config Access).
5.  **Validation**: Run `cargo check` to ensure no dependency conflicts.

**Definition of Done:**
- [ ] Dependencies added.
- [ ] `cargo check` passes.
- [ ] `make build` succeeds.

### Phase 2: Modularization Strategy (3 Days)

**Objective:** Deconstruct `src/config.rs` into focused modules.

**Files:**
- Create: `src/config/types.rs` (Common types)
- Create: `src/config/server.rs` (ServerConfig)
- Create: `src/config/metrics.rs` (MetricsConfig)
- Create: `src/config/providers/` (Provider configs)
- Modify: `src/config/mod.rs` (Re-exports)
- Modify: `src/config.rs` (Deprecate/Remove)

**Implementation Steps:**
1.  **Extract Types**: Move `ServerConfig`, `MetricsConfig` to their own files.
2.  **Extract Providers**: Move `EmbeddingProviderConfig` and `VectorStoreProviderConfig` to `src/config/providers/`.
3.  **Update References**: Fix imports in `src/lib.rs` and `src/main.rs`.
4.  **Validation**: Run `cargo check` after *each* file move.

**Definition of Done:**
- [ ] `src/config.rs` is significantly smaller or removed.
- [ ] Codebase compiles with new module structure.
- [ ] No circular dependencies.

### Phase 3: Modern Config Loading (3 Days)

**Objective:** Replace manual `ConfigManager` with `config` crate builder.

**Files:**
- Create: `src/config/loader.rs`
- Modify: `src/config/mod.rs`
- Test: `tests/config_loading.rs`

**Implementation Steps:**
1.  Implement `ConfigLoader` using `config::Config::builder()`.
    -   Source 1: Default values.
    -   Source 2: File (`~/.context/config.toml`).
    -   Source 3: Environment (`MCP_` prefix).
2.  Add `#[derive(Validate)]` to config structs.
3.  Replace manual `validate()` methods with `validator` calls.
4.  **Validation**: Create a test that loads a dummy config and verifies overrides work.

**Definition of Done:**
- [ ] Manual `merge_env_config` logic removed.
- [ ] `ConfigLoader` successfully loads and merges sources.
- [ ] Validation errors are descriptive.

### Phase 4: Dependency Injection (3 Days)

**Objective:** implementations `shaku` modules to replace manual wiring.

**Files:**
- Create: `src/di/modules.rs`
- Modify: `src/server/server.rs`

**Implementation Steps:**
1.  Define `McpModule` using `shaku::module!`.
2.  Register providers (OpenAI, Milvus, etc.) as `Component`s.
3.  Update `McpServer::new` to resolve dependencies via `McpModule`.
4.  **Validation**: Run `make test` to ensure service resolution works.

**Definition of Done:**
- [ ] `McpServer` no longer manually instantiates services.
- [ ] DI container resolves the graph.

### Phase 5: Hot Reload & Cleanup (2 Days)

**Objective:** Enable runtime updates and remove dead code.

**Files:**
- Modify: `src/main.rs`
- Modify: `src/config/loader.rs`

**Implementation Steps:**
1.  Wrap global config in `ArcSwap`.
2.  Set up `notify` watcher on config file.
3.  On change: reload -> validate -> swap.
4.  **Cleanup**: Remove any remaining legacy config code.
5.  **Validation**: Manual test: change config file while running and verify update.

**Definition of Done:**
- [ ] Config updates without restart.
- [ ] Legacy code fully removed.
- [ ] All tests pass.

## Risk Mitigation & Validation

| Risk | Mitigation |
|------|------------|
| **Breaking Changes** | Changes done in new modules first; switch over atomically. |
| **Runtime Errors** | `validator` ensures config is valid before app start. |
| **Dependency Hell** | `shaku` provides compile-time checks for missing dependencies. |

## Progress Tracking

**MANDATORY: Update this checklist as tasks complete. Change `[ ]` to `[x]`.**

- [x] Phase 1: Foundation Dependencies
- [ ] Phase 2: Modularization Strategy
- [ ] Phase 3: Modern Config Loading
- [ ] Phase 4: Dependency Injection
- [ ] Phase 5: Hot Reload & Cleanup

**Total Tasks:** 5 | **Completed:** 0 | **Remaining:** 5

---
**USER: Please review this plan. Edit any section directly, then confirm to proceed.**
