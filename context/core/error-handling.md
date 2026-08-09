# Error Handling

Last updated: 2026-02-15 (America/Sao_Paulo)

## Libraries

- **thiserror** 2.0 — Domain error types (`#[derive(Error)]`)
- **anyhow** 1.0 — Application/infrastructure context errors

## Domain Error Model (`mcb-domain/src/error/types.rs`)

Single `Error` enum with 25+ typed variants:

| Category | Variants |
|----------|----------|
| I/O | `IoSimple`, `Io` (with context) |
| Serialization | `Json`, `Utf8`, `Base64` |
| Domain | `NotFound`, `InvalidArgument`, `InvalidRegex` |
| Provider | `VectorDb`, `Embedding`, `Cache` |
| Config | `Config`, `Configuration`, `ConfigMissing`, `ConfigInvalid` |
| Infrastructure | `Database`, `Network`, `Infrastructure`, `Internal` |
| Auth | `Authentication` |
| VCS | `Vcs`, `RepositoryNotFound`, `BranchNotFound` |
| Memory | `ObservationStorage`, `ObservationNotFound`, `DuplicateObservation` |
| Browse | `Browse`, `Highlight` |

## Constructor Pattern

```rust
Error::vcs("message")           // Simple construction
Error::vcs_with_source("msg", e) // With source error chain
Error::not_found("resource")
Error::database("message")
```

Never construct enum variants directly — always use constructor methods.

## Propagation Rules

- Use `?` operator for error propagation
- Result alias: `pub type Result<T> = std::result::Result<T, Error>;` per crate
- Domain errors wrap lower-level errors at boundaries
- MCP-safe mapping at transport boundary (`error_mapping.rs`)

## Anti-Patterns (CI-enforced)

- `unwrap()` / `expect()` in production → `unsafe_code = "deny"`
- Empty catch blocks / silent failures
- Type suppression (`as any`, `@ts-ignore`)
- Removing tests to "fix" failures

## Verification

- Re-run checks after fixes
- Note pre-existing vs newly introduced failures

## Sources

- `crates/mcb-domain/src/error/types.rs`
- `crates/mcb-server/src/error_mapping.rs`
- `docs/adr/019-error-handling-strategy.md`

## Update Notes

- 2026-02-15: Full harvest rewrite — complete error model, constructor patterns from source.
- 2026-02-11: Initial policy snapshot.
