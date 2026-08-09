# Agent Patterns

Last updated: 2026-02-15 (America/Sao_Paulo)

## Purpose

MCB-specific orchestration patterns for autonomous work in this Rust workspace.

## Exploration Strategy

1. **First**: `glob` / `grep` / `ast-grep` for known patterns.
2. **Second**: `mcb_search` (semantic) for fuzzy concept discovery.
3. **Third**: `explore` agent for cross-layer pattern analysis.
4. **Fourth**: `librarian` agent for external docs (crate APIs, best practices).

## Implementation Patterns

### Adding a New Provider

1. Define port trait in `mcb-domain/src/ports/providers/` (or reuse existing).
2. Implement adapter in `mcb-providers/src/{category}/{name}.rs`.
3. Register via `#[distributed_slice(EMBEDDING_PROVIDERS)]` (or relevant slice).
4. DI wiring auto-discovers via `linkme` — no manual catalog edits needed.
5. Add config section in `config/default.toml` under `[providers.{category}]`.

### Adding a New MCP Tool

1. Define handler in `mcb-server/src/handlers/{tool_name}.rs`.
2. Register in `mcb-server/src/tools/registry.rs` using `ToolRegistry::register()`.
3. Handler receives `Arc<dyn ServicePort>` via DI — never construct services directly.
4. Return `McpResult<JsonValue>` — errors propagate via `McpError`.

### Adding a New Service

1. Define use case in `mcb-application/src/services/`.
2. Expose via port trait in `mcb-domain/src/ports/`.
3. Register in DI catalog: `mcb-infrastructure/src/di/mod.rs`.
4. Inject into handlers via `Catalog::get::<dyn PortTrait>()`.

## Safety Rules

- **Never** suppress type errors (`#[allow(...)]` for warnings only, never for errors).
- **Never** leave partial broken changes after failed fix attempts.
- **Never** import `mcb_infrastructure` or `mcb_server` from domain/application layers.
- **Always** verify Clean Architecture boundaries: deps flow inward only.
- **Always** run `make lint` before reporting completion.

## Delegation Triggers

| Signal | Action |
|--------|--------|
| Unknown crate API | Fire `librarian` in background |
| Cross-crate refactor (3+ crates) | Fire `explore` first, then plan |
| Architecture boundary question | Consult `oracle` |
| CI failure on boundary check | Check `make validate` output |
| New provider integration | Read existing provider in same category first |

## Verification Checklist

- [ ] `cargo check --workspace` passes
- [ ] `make lint` clean (clippy + fmt)
- [ ] `make test` passes (or pre-existing failures documented)
- [ ] No layer violations (domain imports no infra crates)
- [ ] Context files updated if architecture changed

## Sources

- `crates/mcb-server/src/tools/registry.rs` (tool registration)
- `crates/mcb-infrastructure/src/di/mod.rs` (DI catalog)
- `crates/mcb-providers/src/` (provider adapters)
- `crates/mcb-domain/src/ports/` (port traits)

## Update Notes

- 2026-02-15: Full rewrite with MCB-specific DI, provider, tool, and service patterns.
- 2026-02-11: Initial generic baseline.
