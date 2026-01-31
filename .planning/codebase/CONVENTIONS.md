# Coding Conventions

**Analysis Date:** 2025-01-31

## Naming Patterns

**Files:**
- Lowercase with underscores: `code_chunk.rs`, `embedding_provider.rs`, `mcp_server.rs`
- Test files: `*_test.rs` or `*_tests.rs` (e.g., `crypto_tests.rs`, `formatter_tests.rs`)
- Integration test files in `tests/integration/` directory
- Unit test files in `tests/unit/` directory

**Functions:**
- Lowercase with underscores: `get_indexing_status()`, `resolve_embedding_provider()`, `create_test_config()`
- Test functions: `test_<subject>_<scenario>_<expected>` pattern
  - Examples: `test_crypto_service_encrypt_decrypt()`, `test_code_chunk_creation()`
- Async functions: `async fn <name>()` with async runtime operations
- Public API functions should have doc comments describing behavior

**Types (Structs/Enums):**
- PascalCase: `AppContext`, `McpServer`, `EmbeddingProviderHandle`, `SearchResult`
- Port traits: `<Feature>Interface` or `<Feature>Provider` suffix
  - Examples: `IndexingServiceInterface`, `EmbeddingProvider`, `VectorStoreBrowser`
- Error types: PascalCase enum variants in `Error` enum
  - Example: `Error::ConfigurationError { message }`, `Error::NotFound { resource }`

**Variables:**
- Lowercase with underscores: `config`, `provider_name`, `total_files`, `is_indexing`
- Const/static: SCREAMING_SNAKE_CASE: `MAX_CHUNK_SIZE`, `DEFAULT_TIMEOUT_SECS`
- Private fields: prefix underscore when intentionally unused, e.g., `#[allow(dead_code)] embedding_resolver`

**Module Organization:**
- Files named after module concept: `mod.rs` for module entry, submodules in subdirectories
- Clean architecture layers: `domain`, `application`, `infrastructure`, `providers`, `server`
- Crate organization: `src/`, `tests/`

## Code Style

**Formatting:**
- Tool: `rustfmt` via `cargo fmt`
- Max line width: 100 characters (`rustfmt.toml`: `max_width = 100`)
- Edition: Rust 2024
- Tab spaces: 4
- Run `make fmt` to auto-format all code

**Linting:**
- Tool: `clippy` (Rust linter)
- Config: `clippy.toml` and workspace `Cargo.toml`
- Key rules enabled: `all` (warn), `pedantic` (warn)
- Allowed exceptions: `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`
- Test allowances: `allow-unwrap-in-tests = true`, `allow-expect-in-tests = true`, `allow-print-in-tests = true`
- Run `make lint` before committing

**Unsafe Code:**
- Marked with `#[warn(unsafe_code)]` - only use when absolutely necessary
- Document why unsafe is required with comments
- Prefer safe abstractions whenever possible

## Import Organization

**Order:**
1. Standard library: `use std::*`
2. External crates: `use tokio::*`, `use serde::*`, `use thiserror::*`
3. Workspace crates: `use mcb_domain::*`, `use mcb_application::*`
4. Local modules: `use crate::*`, `use super::*`

**Path Aliases:**
- Workspace uses standard path imports: `use crate::di::bootstrap`
- No custom path aliases in codebase
- Prefer full paths for clarity: `mcb_domain::Error` not abbreviated

**Derive Macros:**
- Standard derives: `#[derive(Debug, Clone)]`
- Async-aware: `#[async_trait]` for trait async methods
- Serialization: `#[derive(Serialize, Deserialize)]` from serde
- Error types: `#[derive(Error, Debug)]` from thiserror

## Error Handling

**Pattern:**
- Use `thiserror` crate for custom error types
- Define in `src/error.rs` or `src/errors.rs`
- Use `Result<T>` type alias: `type Result<T> = std::result::Result<T, Error>`
- Return errors via `?` operator, NOT `unwrap()` or `expect()`

**Error Type Structure:**
```rust
#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {message}")]
    Io { message: String, #[source] source: Option<Box<...>> },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },
}
```

**Allowed Exceptions:**
- `unwrap()` and `expect()` allowed ONLY in test code (per `clippy.toml`)
- In production code: convert to proper error returns using `?` operator
- Example in test: `service.encrypt(plaintext).unwrap()` - acceptable
- Example in source: `service.encrypt(plaintext)?` - required

**Domain Layer (`mcb-domain`):**
- 7 occurrences of `unwrap`/`expect` (very minimal, mostly in error handling paths)
- Use error types defined in `error.rs`

## Logging

**Framework:** `tracing` crate
- Structured logging with `tracing::info!()`, `tracing::warn!()`, `tracing::error!()`, `tracing::debug!()`
- Set up via `tracing-subscriber` with environment filter
- Config: `crates/mcb-infrastructure/src/logging.rs`

**Patterns:**
- Log at entry/exit of important functions: `tracing::info!("Starting indexing operation")`
- Use structured fields: `tracing::info!(file_path = ?path, status = "processing")`
- Error logging: `tracing::error!("Operation failed: {}", error)`
- Avoid print/println in production code (use logging instead)

**Test Logging:**
- Print statements allowed in tests via `allow-print-in-tests`
- Use `#[test]` with `--nocapture` to see output: `cargo test -- --nocapture`

## Comments

**When to Comment:**
- Complex algorithms or non-obvious logic
- Business rule explanations that aren't clear from code
- TODO/FIXME items marked with issue numbers when possible
- Safety-critical sections explaining why unsafe is needed
- Documentation of ports (trait definitions) with examples

**Doc Comments (///):**
- Module-level documentation on `mod.rs` files
- Public API functions with examples
- Type documentation explaining purpose and invariants
- Examples in doc comments are tested via doctests

**JSDoc/Doc Style:**
```rust
/// Returns the embedding dimensions.
///
/// # Returns
/// The number of dimensions in the embedding vector.
pub fn dimensions(&self) -> usize
```

## Function Design

**Size Guidelines:**
- Target: <100 lines for most functions
- Largest functions in codebase: validation rules (800+ lines justified by logic complexity)
- Core business logic split across modules at 300-400 lines

**Parameters:**
- Prefer passing references: `fn process(&self, path: &str)` not `&String`
- Use strong types: `&Path` for file paths, custom structs for complex options
- Limit to 5-6 parameters; use builder pattern or config structs for more

**Return Values:**
- Return `Result<T>` for fallible operations
- Use `Option<T>` for optional values (not `Result`)
- Avoid returning large tuples; use named structs instead

**Async Functions:**
- Use `async fn` with `#[async_trait]` for trait methods
- Return futures: `async fn operation() -> Result<T>`
- Tokio is the async runtime (configured in `Cargo.toml`)

## Module Design

**Exports:**
- Re-export commonly used types in module: `pub use entities::CodeChunk`
- Maintain clear public API per module
- Private implementation details stay in submodules

**Barrel Files:**
- `mod.rs` files re-export public types for convenience
- Keep barrel files small and focused
- Example: `mcb-domain/src/mod.rs` re-exports `entities::CodeChunk`

**File Structure:**
- One primary type per file: `code_chunk.rs` contains `CodeChunk` struct
- Related helpers in same file
- Test modules inline with `#[cfg(test)]`

**Async Module Pattern:**
- Service types implement trait methods with `async fn`
- DI container provides Arc<dyn Trait> for services
- All I/O operations are async (Tokio runtime)

## Architecture Patterns

**Clean Architecture Layers:**
- **Domain** (`mcb-domain`): Pure business logic, no external deps, only `std` + minimal crates
- **Application** (`mcb-application`): Use cases, ports, decorators, zero infrastructure knowledge
- **Infrastructure** (`mcb-infrastructure`): Config, DI, provider implementations, external services
- **Server** (`mcb-server`): MCP protocol, HTTP transport, handlers
- **Providers** (`mcb-providers`): Pluggable provider implementations (embedding, vector store, cache)

**Dependency Direction:**
```
server → infrastructure → application → domain
                   ↓              ↑
              providers ─────────┘
```

**Trait-Based DI:**
- Providers are traits: `pub trait EmbeddingProvider`
- Implementations are concrete types in provider layer
- Services accept `Arc<dyn Trait>`, not concrete types
- Handle pattern for runtime switching: `Arc<EmbeddingProviderHandle>`

**Provider Registration:**
- Auto-registration via `linkme` distributed slices
- Providers register into registry at compile time
- Resolvers look up providers by name and create instances
- No runtime plugin system; all providers compiled in

## Quality Standards (Enforced by CI)

**Before Commit:**
- `make fmt` - Format all code
- `make lint` - Check linting (0 errors, warnings OK in non-critical)
- `make test` - 950+ tests pass (see TESTING.md)
- `make validate` - Architecture validation passes
- `cargo clippy` - No clippy warnings in affected code

**No Production Code:**
- Panics (unwrap/expect) - FORBIDDEN except in tests
- println!/print! - Use logging instead
- Hardcoded values - Use config system
- `#[allow(dead_code)]` without documentation

**Workspace Lints:**
```toml
[workspace.lints.rust]
unsafe_code = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
```

---

*Convention analysis: 2025-01-31*
