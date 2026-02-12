# MCB Rust Style Guide

## Project Identity

MCB (Memory Context Browser) is a high-performance MCP server written in Rust (edition 2024, MSRV 1.92). It uses a Clean Architecture workspace with 9 crates. All code is reviewed against these rules.

## Architecture (Non-Negotiable)

### Crate Dependency Direction

Dependencies ALWAYS point inward. Violations are CI-blocking.

```ascii
mcb-server -> mcb-infrastructure -> mcb-application -> mcb-domain
                    |                      ^
              mcb-providers ---------------+
```

- `mcb-domain`: entities, port traits, errors. Zero MCB dependencies.
- `mcb-application`: use cases, services. Depends only on `mcb-domain`.
- `mcb-providers`: concrete implementations of port traits. Depends on `mcb-domain` + `mcb-application`.
- `mcb-infrastructure`: DI (dill), config (figment), logging, health. Depends on domain/application/providers.
- `mcb-server`: MCP protocol, HTTP transport. Depends on infrastructure. Never directly on providers.
- `mcb-validate`: dev-only architecture validation. Not a runtime dependency.

### Port/Adapter Pattern

Port traits live in `mcb-domain`. Adapters live in `mcb-providers`. Services in `mcb-application` consume ports via trait objects (`Arc<dyn Trait>`), never concrete types.

### Provider Registration

New providers MUST register via `#[linkme::distributed_slice]` with function pointer factories (not closures). This enables compile-time auto-discovery.

## Safety Rules

### Forbidden Patterns

- `unsafe` code: denied workspace-wide via `unsafe_code = "deny"` in Cargo.toml.
- `unwrap()` or `expect()` outside of tests: use `?` propagation and helper constructors like `Error::io(...)`.
- `#[allow(unused_*)]`: remove dead code instead of suppressing warnings.
- `as any` / `@ts-ignore` equivalents: never suppress type errors.
- Empty `catch` / error-swallowing patterns.
- `todo!()` or `unimplemented!()` in non-draft code.

### Error Handling (ADR-019)

- Return `Result<T, Error>` using the workspace error types.
- Use thiserror for domain errors, anyhow for application-layer orchestration.
- Create typed helper constructors: `Error::io(msg)`, `Error::config(msg)`.
- Never panic in library code.

## Formatting

- Edition: 2024
- Max line width: 100 characters
- Indent: 4 spaces (no tabs)
- Import order: `std` -> external crates -> workspace crates (`mcb_*`) -> local modules
- Run `make fmt` before committing (enforced in CI via `make lint MCB_CI=1`)

## Naming Conventions

- Types, traits, enums: `PascalCase` — `EmbeddingProvider`, `SearchResult`
- Functions, methods, variables: `snake_case` — `embed_batch`, `chunk_count`
- Constants: `UPPER_SNAKE_CASE` — `EMBEDDING_PROVIDERS`, `MAX_RETRIES`
- Crate names: `mcb-*` (kebab-case in Cargo.toml, `mcb_*` in Rust code)
- Modules: `snake_case` matching the concept they own

## Documentation

- `missing_docs = "warn"` is workspace-wide. Public items should have doc comments.
- Use `///` for public API docs. Explain what and why, not how.
- Module-level docs (`//!`) for each `lib.rs` and significant modules.
- Keep doc comments concise — if the function name and types make the purpose obvious, a one-liner suffices.

## Testing

### Test Organization

- Unit tests: `crates/*/tests/unit/` (separate files, not inline `#[cfg(test)]` modules for complex tests)
- Integration tests: `crates/*/tests/integration.rs`
- Golden acceptance tests: `tests/golden/` (end-to-end MCP scenarios)
- E2E browser tests: `tests/e2e/` (Playwright, TypeScript)

### Test Requirements

- New logic MUST include tests.
- Use `#[tokio::test]` for async tests.
- Prefer `assert_eq!` / `assert!` with descriptive messages.
- Use `tempfile` for filesystem tests; clean up resources.
- Use `serial_test` when tests share global state.

## Dependency Management

- Workspace-level dependencies in root `Cargo.toml` `[workspace.dependencies]`.
- Crates inherit via `dep.workspace = true`.
- `cargo-deny` enforces license allowlist (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016).
- No unknown registries or git sources (enforced in `deny.toml`).
- Dependabot manages updates weekly; patch/minor auto-merge, major requires manual review.

## Build System

- **Always use `make` targets** — never raw `cargo` commands in CI or docs.
- `make build` / `make build-release` — compile
- `make test` — all tests (unit + integration)
- `make lint MCB_CI=1` — rustfmt check + clippy strict
- `make validate` — architecture boundary validation (9 rules, 7 phases)
- `make audit` — security audit (cargo-audit + cargo-deny + osv-scanner)
- `make quality` — fmt + lint + test in one step
- `make ci-full` — mirrors GitHub Actions exactly

## Commit Messages

Conventional commits with scope:

```ascii
<type>(<scope>): <short description>

<body: 1-2 sentences explaining why>

Fixes #<issue-id>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`

Scope: module or crate name (e.g., `core`, `cli`, `mcb-server`, `docs`)

## Pull Request Expectations

Before opening:

- `make test` passes
- `make lint MCB_CI=1` passes
- `make validate` passes (zero architecture violations)
- Documentation updated if public API changed

PR description must include:

- What changed and why
- How to test
- Breaking changes (if any)

## Code Review Focus Areas

When reviewing PRs, prioritize in this order:

1. **Architecture boundaries**: Does this respect the crate dependency graph? No inner-to-outer imports.
2. **Safety**: No unsafe, no unwrap outside tests, proper error propagation.
3. **Correctness**: Logic errors, edge cases, race conditions.
4. **Performance**: Unnecessary allocations, O(n^2) patterns, missing async.
5. **Maintainability**: Naming, modularity, idiomatic Rust.
6. **Security**: Input validation, credential handling, injection vectors.
7. **Tests**: New code has tests. Existing tests not deleted to "pass".

## Async Patterns

- Tokio is the runtime (full features).
- Use `async fn` and `.await` — no blocking calls on the async runtime.
- Use `tokio::spawn` for concurrent work; `futures::join!` for parallel awaits.
- Channel types: `tokio::sync::mpsc` for internal messaging.

## Configuration (ADR-025)

- Figment loads config from: defaults -> `config/default.toml` -> env vars -> CLI args.
- Config types live in `mcb-infrastructure`.
- Environment variable prefix: standard env names (e.g., `EMBEDDING_PROVIDER`, `OPENAI_API_KEY`).

## DI Pattern (ADR-029)

- IoC container: `dill` crate.
- Bootstrap via `init_app(config) -> AppContext`.
- Services resolved via trait objects: `catalog.get_one::<dyn Trait>()`.
- Handles (`RwLock<Arc<dyn Trait>>`) enable runtime provider switching.
