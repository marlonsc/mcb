# Core Module

**Source**: `src/domain/` (types, ports) and `src/infrastructure/` (utilities)

Foundational types, traits, and utilities used throughout the system.

## Overview

The core module functionality is now split across Clean Architecture layers:
- **Domain types** (`src/domain/types.rs`): Embedding, CodeChunk, SearchResult, Language
- **Port traits** (`src/domain/ports/`): EmbeddingProvider, VectorStoreProvider, etc.
- **Infrastructure utilities** (`src/infrastructure/`): auth, cache, crypto, rate_limit

## Submodules

### Types (`types.rs`)

Core data structures for code intelligence.

\1-   `Embedding` - Vector representation of text/code
\1-   `CodeChunk` - Parsed code segment with metadata
\1-   `SearchResult` - Ranked search item with score
\1-   `Language` - Supported programming languages

### Error Handling (`error.rs`)

Comprehensive error types with `thiserror`.

\1-   `Error` - Main error enum with variants
\1-   `Result<T>` - Type alias for `Result<T, Error>`

### Authentication (`auth.rs`)

JWT-based identity and access management.

\1-   `AuthService` - Token validation and generation
\1-   `Claims` - JWT payload structure
\1-   `Permission` - Authorization controls

### Caching (`cache.rs`)

Multi-level caching with TTL and size limits.

\1-   `CacheManager` - Main cache interface
\1-   Configurable TTL and eviction policies

### Rate Limiting (`rate_limit.rs`)

Request throttling with multiple strategies.

\1-   `RateLimiter` - Token bucket implementation
\1-   Configurable limits per endpoint/user

### Hybrid Search (`hybrid_search.rs`)

Combined BM25 + semantic search.

\1-   `HybridSearchEngine` - Orchestrates dual ranking
\1-   `BM25Scorer` - Term frequency ranking
\1-   Configurable weighting between methods

### Other Utilities

\1-   `crypto.rs` - Encryption utilities (AES-GCM)
\1-   `database.rs` - Connection pooling
\1-   `http_client.rs` - HTTP client with retry
\1-   `limits.rs` - Resource quotas
\1-   `merkle.rs` - Data integrity verification

## Key Exports

```rust
// Domain types
pub use types::{Embedding, CodeChunk, SearchResult, Language};
pub use error::{Error, Result};

// Security
pub use auth::{AuthService, Permission, Claims};
pub use crypto::*;

// Infrastructure
pub use cache::CacheManager;
pub use rate_limit::RateLimiter;
pub use hybrid_search::HybridSearchEngine;
```

## File Structure (Clean Architecture)

```text
src/domain/
├── types.rs         # Domain types (Embedding, CodeChunk, etc.)
├── error.rs         # Domain error types
├── validation.rs    # Input validation rules
└── ports/           # Port traits (interfaces)

src/infrastructure/
├── auth/            # JWT authentication
├── cache.rs         # Multi-level caching
├── crypto/          # Encryption utilities
├── rate_limit.rs    # Request throttling
└── ...              # Other infrastructure
```

## Testing

Core types have 18 dedicated tests. See [tests/core_types.rs](../../tests/core_types.rs).

## Cross-References

\1-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
\1-  **Services**: [services.md](./services.md) (uses core types)
\1-  **Providers**: [providers.md](./providers.md) (implements traits)
\1-  **Server**: [server.md](./server.md) (uses auth/rate limiting)
