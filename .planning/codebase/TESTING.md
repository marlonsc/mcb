# Testing Patterns

**Analysis Date:** 2025-01-31

## Test Framework

**Runner:**
- Rust built-in test framework (no external test runner needed)
- Run via `cargo test` command
- Parallel execution by default (use `TEST_THREADS=1` to serialize)

**Assertion Library:**
- Rust standard `assert!()`, `assert_eq!()`, `assert_ne!()`
- No external assertion crate needed
- Tests use pattern matching for complex assertions

**Run Commands:**
```bash
make test                           # Run all tests (~950+ tests)
make test QUICK=1                   # Run fast tests only (skip integration)
cargo test -p mcb-domain           # Single crate tests
cargo test -p mcb-infrastructure --test unit      # Unit tests only
cargo test -p mcb-infrastructure --test integration # Integration tests only
cargo test test_crypto_service -- --nocapture     # Single test with output
```

**Coverage:**
```bash
make coverage                       # Generate coverage report (tarpaulin)
# Coverage excludes integration tests that need external services
# Target: 80%+ coverage enforced
```

## Test File Organization

**Location:**
- Two patterns used in codebase:
  1. **External tests**: `tests/` directory at crate root (preferred for integration)
  2. **Inline tests**: `#[cfg(test)] mod tests { }` in source files (for unit/examples)

**File Naming:**
- External unit tests: `tests/unit/<name>_tests.rs` or `tests/unit/<name>_test.rs`
- External integration tests: `tests/integration/<name>_integration.rs`
- Test organization files: `tests/unit.rs` (module aggregator), `tests/integration.rs`

**Structure by Crate:**

```
mcb-domain/
├── src/
│   └── mod.rs (inline #[cfg(test)] modules in main files)
└── tests/
    ├── unit/
    │   ├── code_chunk_tests.rs
    │   ├── config_tests.rs
    │   └── types_tests.rs
    └── lib.rs (shared test utils)

mcb-infrastructure/
├── src/
│   └── (no inline tests)
└── tests/
    ├── unit.rs (aggregator file)
    ├── unit/
    │   ├── crypto_tests.rs
    │   ├── health_tests.rs
    │   ├── di_tests.rs
    │   └── ... (32+ unit test files)
    ├── integration.rs
    ├── integration/
    │   ├── cache/
    │   ├── config/
    │   ├── di/
    │   └── utils/
    ├── test_utils/ (shared fixtures)
    └── lib.rs

mcb-server/
└── tests/
    ├── unit.rs (aggregator)
    ├── unit/
    │   ├── formatter_tests.rs
    │   ├── builder_tests.rs
    │   └── ...
    ├── integration.rs
    ├── integration/
    │   ├── browse_api_integration.rs
    │   └── operating_modes_integration.rs
    └── test_utils/ (fixtures)
```

## Test Structure

**Suite Organization:**

Inline test module (domain layer):
```rust
// In src/entities/code_chunk.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_chunk_creation() {
        let chunk = CodeChunk { /* ... */ };
        assert_eq!(chunk.id, "test-chunk-001");
    }
}
```

External test file (integration tests):
```rust
// In tests/integration/operating_modes_integration.rs
use mcb_infrastructure::config::types::{AppConfig, ModeConfig};

#[test]
fn test_mode_config_defaults_to_standalone() {
    let config = ModeConfig::default();
    assert_eq!(config.mode_type, OperatingMode::Standalone);
}

#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

**Test Organization:**
- Simple unit tests: Single `#[test]` function per behavior
- Complex test suites: Nested `mod tests { }` with helper functions
- Async tests: Use `#[tokio::test]` attribute
- Integration tests: Full end-to-end test of system components

**Patterns Used:**

1. **Arrange-Act-Assert (AAA):**
```rust
#[test]
fn test_encrypt_decrypt_roundtrip() {
    // Arrange
    let master_key = CryptoService::generate_master_key();
    let service = CryptoService::new(master_key).unwrap();
    let plaintext = b"Hello, World!";

    // Act
    let encrypted = service.encrypt(plaintext).unwrap();
    let decrypted = service.decrypt(&encrypted).unwrap();

    // Assert
    assert_eq!(plaintext.to_vec(), decrypted);
}
```

2. **Builder Pattern for Test Setup:**
```rust
pub fn with_collections(mut self, collections: Vec<CollectionInfo>) -> Self {
    self.collections = collections;
    self
}

// Usage in test:
let mock = MockVectorStoreBrowser::new()
    .with_collections(vec![...])
    .with_files(vec![...])
    .with_chunks(vec![...]);
```

3. **Helper Functions for Common Setup:**
```rust
// tests/test_utils/test_fixtures.rs
pub fn create_test_search_result(id: &str) -> SearchResult { /* ... */ }
pub fn create_test_search_results(count: usize) -> Vec<SearchResult> { /* ... */ }

// Usage:
let results = create_test_search_results(3);
```

## Mocking

**Framework:** Custom trait implementations (no external mocking library)

**Strategy:**
- Mock structs implement the trait interface
- Provides canned responses for testing
- Used for external dependencies (providers, browsers)

**Pattern (from `browse_api_integration.rs`):**
```rust
pub struct MockVectorStoreBrowser {
    collections: Vec<CollectionInfo>,
    files: Vec<FileInfo>,
    chunks: Vec<SearchResult>,
}

#[async_trait]
impl VectorStoreBrowser for MockVectorStoreBrowser {
    async fn list_collections(&self) -> Result<Vec<CollectionInfo>> {
        Ok(self.collections.clone())
    }
}
```

**What to Mock:**
- External service providers (embedding, vector store, cache)
- HTTP clients and external APIs
- File system operations (for integration tests)
- Trait implementations that would require external setup

**What NOT to Mock:**
- Core domain logic (entities, value objects)
- Business rules and calculations
- In-memory operations that are fast
- Error paths that should be tested as-is

**Real Providers in Tests:**
- Some integration tests use real provider implementations
- Example: `mcb-infrastructure/tests/test_utils/real_providers.rs`
- These test actual provider behavior in isolation
- Useful for testing config loading and provider initialization

## Fixtures and Factories

**Test Data (from `test_utils/test_fixtures.rs`):**
```rust
pub fn create_test_search_result(id: &str) -> SearchResult {
    SearchResult {
        file_path: format!("src/test_{}.rs", id),
        content: "fn test() {}".to_string(),
        start_line: 1,
        end_line: 1,
        language: "rust".to_string(),
        // ... other fields
    }
}

pub fn create_test_search_results(count: usize) -> Vec<SearchResult> {
    (0..count)
        .map(|i| create_test_search_result(&format!("chunk_{}", i)))
        .collect()
}
```

**Location:**
- `tests/test_utils/test_fixtures.rs` - Reusable test data builders
- `tests/test_utils/mod.rs` - Module declaration
- `tests/lib.rs` - Shared test library setup
- Helper functions in test files for simple cases

**Builder Pattern Usage:**
```rust
let config = EmbeddingProviderConfig::new("test")
    .with_model("model-1")
    .with_api_key("secret")
    .with_dimensions(384);

assert_eq!(config.provider, "test");
```

## Coverage

**Requirements:**
- Target: 80%+ code coverage across crates
- Tool: Tarpaulin (coverage measurement)
- Config: `.tarpaulin.toml` (excludes integration tests needing external services)

**View Coverage:**
```bash
make coverage          # Generate coverage report
cargo tarpaulin -o Html --out coverage/  # HTML report
```

**Excluded from Coverage:**
- Integration test files (take too long, need external services)
- Generated code (migrations, protobuf)
- Configuration-only modules (simple getters)

**Coverage in CI:**
- Enforced on all pull requests
- Caches coverage reports between runs
- Fails if coverage drops below target

## Test Types

**Unit Tests:**
- Test single functions or small components in isolation
- Fast: < 1ms per test
- Use mocks for external dependencies
- Located in `tests/unit/` or inline `#[cfg(test)]`
- Examples: `test_crypto_service_encrypt_decrypt()`, `test_code_chunk_creation()`

**Integration Tests:**
- Test multiple components working together
- May use real providers (null, in-memory) to avoid external services
- Test configuration loading, DI container, service initialization
- Located in `tests/integration/`
- Examples: `test_mode_config_defaults_to_standalone()`, `test_cache_provider_initialization()`

**E2E Tests:**
- Not present in this codebase
- Would test complete workflows through HTTP API
- Located in `tests/integration/` with names like `*_integration.rs`
- Examples: `operating_modes_integration.rs`, `browse_api_integration.rs`

## Common Patterns

**Async Testing:**
```rust
#[tokio::test]
async fn test_indexing_operation() {
    let service = create_test_service().await;
    let result = service.index_codebase("./test_data").await;
    assert!(result.is_ok());
}
```

**Error Testing:**
```rust
#[test]
fn test_resolve_unknown_provider_fails() {
    let config = EmbeddingProviderConfig::new("nonexistent");
    let result = resolve_embedding_provider(&config);
    assert!(result.is_err(), "Should fail for unknown provider");
}

#[test]
fn test_crypto_service_invalid_key_size() {
    let invalid_key = vec![0u8; 16]; // Wrong size
    assert!(CryptoService::new(invalid_key).is_err());
}
```

**Testing Complex Logic (Formatter):**
```rust
#[test]
fn test_format_search_response_with_results() {
    let results = create_test_search_results(3);
    let duration = Duration::from_millis(150);

    let response = ResponseFormatter::format_search_response(
        "test query",
        &results,
        duration,
        10
    );

    assert!(response.is_ok());
    let result = response.expect("Expected successful response");
    assert!(!result.is_error.unwrap_or(false));
}
```

**Testing Configuration:**
```rust
#[test]
fn test_mode_config_toml_deserialization() {
    let toml = r#"
        type = "client"
        server_url = "http://localhost:9999"
        session_prefix = "claude"
        timeout_secs = 60
    "#;

    let config: ModeConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.mode_type, OperatingMode::Client);
}
```

## Test Count and Organization

**Total Test Count:** 709+ unit/integration tests across all crates

**Distribution:**
- `mcb-infrastructure`: ~300+ tests (DI, config, crypto, health checks)
- `mcb-server`: ~200+ tests (API handlers, formatting, admin routes)
- `mcb-application`: ~100+ tests (registry, services)
- `mcb-domain`: ~50+ tests (entities, value objects)
- `mcb-validate`: Embedded doctests and inline tests
- `mcb-providers`: Inline provider tests

**Test Target Groups:**
- Unit tests: Fast suite, no external services
- Integration tests: May need docker services (Milvus, Redis, Ollama)
- CI tests: SKIP integration tests that need external services

## Clippy Test Exceptions

**Allowed in Tests (via `clippy.toml`):**
```toml
allow-unwrap-in-tests = true      # OK to use .unwrap() in tests
allow-expect-in-tests = true      # OK to use .expect() in tests
allow-print-in-tests = true       # OK to use println! in tests
```

**Test Code Quality:**
- Tests use `unwrap()` for setup that should not fail
- `expect()` with descriptive messages for failure points
- `println!()` allowed for debug output (use `cargo test -- --nocapture`)

## Running Tests

**Common Commands:**
```bash
make test                          # Run all tests
make test SCOPE=mcb-server         # Single crate
cargo test -p mcb-server           # Alternative
cargo test -p mcb-server --lib     # Library tests only
cargo test -p mcb-server --test '*' # Integration tests only
cargo test test_name -- --nocapture # Single test with output
cargo test -- --test-threads=1     # Serial execution
```

**Debug Test Output:**
```bash
RUST_LOG=debug cargo test -- --nocapture
cargo test -- --nocapture --test-threads=1
```

## Test Naming Convention

**Required Pattern:** `test_<function>_<scenario>_<expected>`

**Examples (from codebase):**
- `test_crypto_service_encrypt_decrypt` - function, scenario, expected result
- `test_code_chunk_creation` - what's being tested
- `test_code_chunk_with_empty_metadata` - function with specific scenario
- `test_format_search_response_with_results` - component, scenario, expectation
- `test_format_search_response_no_results` - same component, different scenario
- `test_resolve_unknown_provider_fails` - function, scenario, expected failure

**Self-Documenting Names:**
Names must be clear enough to understand test purpose without reading code. Examples:
- ✅ `test_password_service_hash_verify_wrong_password_returns_false`
- ❌ `test_password_hash`
- ✅ `test_config_builder_with_all_parameters`
- ❌ `test_builder`

---

*Testing analysis: 2025-01-31*
