# Technology Stack

**Analysis Date:** 2026-01-31

## Languages

**Primary:**
- Rust 1.92+ (nightly) - 8 crates, core application (100% of codebase)
  - Domain, application, infrastructure, server, providers, validation

**Secondary:**
- Markdown - Documentation and ADRs
- TOML - Configuration files (Cargo.toml, config/default.toml, book.toml)
- Python - Build/testing scripts, AST analysis tools

## Runtime

**Environment:**
- Rust nightly toolchain (specified in `rust-toolchain.toml`)
- Target: x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc
- Minimum Rust version: 1.89

**Package Manager:**
- Cargo (workspace: 7 member crates)
- Workspace resolver: "2"
- Lockfile: `Cargo.lock` (199KB, committed)

## Frameworks

**Core:**
- **Tokio** 1.49 - Async runtime with full feature set (core dependency)
- **Rocket** 0.5 - HTTP server framework (migrated from Axum in v0.1.2, ADR-026)
  - Provides web UI, admin dashboard, metrics endpoints
  - Handles both MCP protocol and HTTP transport

**MCP Protocol:**
- **rmcp** 0.14 - Model Context Protocol SDK
  - Features: server, client, macros, transport-io, transport-child-process
  - Supports stdio and HTTP transports, hybrid mode

**Testing:**
- **Criterion** 0.8.1 - Benchmarking framework
- **Serial Test** 3.2 - Sequential test execution
- **Tempfile** 3.19 - Temporary file handling in tests

**Build/Dev:**
- **Cargo** - Workspace management with resolver v2
- **Clippy** - Linting (Rust all + pedantic, configured in Cargo.toml)
- **Rustfmt** - Code formatting

## Key Dependencies

**Critical (Core Functionality):**
- **Tree-sitter** 0.26 - Incremental parsing library for code analysis
  - Core for language-agnostic AST parsing and code chunking
- **Serde + serde_json** 1.0 - Serialization/deserialization
- **Async-trait** 0.1 - Async trait support for provider abstraction

**Infrastructure & DI:**
- **dill** 0.15 - IoC container for dependency injection (ADR-029)
- **Figment** 0.10 - Configuration loading from TOML/environment
- **linkme** 0.3 - Plugin registration for auto-discovery of providers

**Embedding & Vector Search:**
- **fastembed** 5.8 - Local embeddings without external services
  - Default embedding provider, pure Rust implementation
  - No external AI service required for basic functionality
- **reqwest** 0.13 - HTTP client with JSON support
  - Used by OpenAI, VoyageAI, Gemini, Ollama, Anthropic providers
  - Feature: "__tls" for TLS support

**Code Analysis:**
- **rust-code-analysis** (git: marlon-costa-dc/master) - Mozilla code analysis fork
  - Custom version with tree-sitter 0.26.3 + Kotlin support
  - Cognitive complexity analysis
- **Tree-sitter Language Parsers** (v0.23-0.25):
  - Rust 0.24, Python 0.25, JavaScript 0.25, TypeScript 0.23
  - Go 0.25, Java 0.23, C 0.24, C++ 0.23, C# 0.23
  - Ruby 0.23, PHP 0.24, Swift 0.7, Kotlin 1.1

**Vector Stores:**
- **milvus-sdk-rust** 0.1.0 (git) - Milvus cloud vector database
  - From git (crates.io v0.2.0 has lifetime bug)
  - Production vector store backend
- **edgevec** 0.8 - High-performance in-memory vector store
- **Pinecone client** - Pinecone cloud vector store (REST-based)
- **Qdrant client** - Qdrant vector database (REST-based)

**Caching:**
- **moka** 0.12 - In-memory cache with TTL (default)
- **redis** 1.0 - Distributed cache for multi-instance
  - Features: tokio-comp, connection-manager

**Cryptography & Security:**
- **bcrypt** 0.18 - Password hashing
- **argon2** 0.5 - Password hashing (modern alternative)
- **aes-gcm** 0.10 - AES-256-GCM encryption for encrypted vector store
- **base64** 0.22 - Base64 encoding/decoding
- **sha2** 0.10 - SHA-256 hashing
- **pbkdf2** 0.12 - Key derivation (with "simple" feature)
- **hmac** 0.12 - HMAC message authentication

**Database:**
- **r2d2** 0.8 - Connection pooling
- **r2d2_postgres** 0.18 - PostgreSQL driver
  - Optional: only if DATABASE_URL environment variable is set
  - Fallback to filesystem/in-memory if not configured

**Monitoring & Metrics:**
- **prometheus** 0.14 - Prometheus metrics exporter
- **metrics** 0.24 - Metrics abstraction layer
- **metrics-exporter-prometheus** 0.18 - Prometheus exporter
- **health** 0.2 - Health check utilities
- **sysinfo** 0.38 - System metrics (CPU, memory, disk)
- **nix** 0.31 - POSIX utilities (fs features)

**Logging:**
- **tracing** 0.1 - Structured logging framework
- **tracing-subscriber** 0.3 - Logging subscriber with env-filter and JSON
- **tracing-appender** 0.2 - Logging appenders

**Async & Concurrency:**
- **futures** 0.3 - Async primitives
- **futures-util** 0.3 - Future utilities
- **async-stream** 0.3 - Async streams
- **rayon** 1.8 - Parallel processing
- **dashmap** 6.0 - Concurrent HashMap
- **tokio-util** 0.7 - Tokio utilities (time, rt features)

**Events & Messaging:**
- **async-nats** 0.46 - NATS pub/sub messaging (optional, for distributed events)

**Validation & Schema:**
- **validator** 0.20 - Input validation with derive macros
- **schemars** 1.0 - JSON schema generation for MCP tools
- **garde** 0.22 - Runtime validation with derive macros
- **jsonschema** 0.40 - JSON schema validation

**Text & File Processing:**
- **regex** 1.12 - Regular expressions
- **pulldown-cmark** 0.13 - Markdown processing
- **unicode-segmentation** 1.12 - Unicode grapheme segmentation
- **walkdir** 2.5 - Directory traversal
- **globset** 0.4 - Glob pattern matching
- **glob** 0.3 - File globbing
- **ignore** 0.4 - Git ignore parsing
- **shellexpand** 3.1 - Shell variable expansion

**Utilities:**
- **uuid** 1.20 - UUID generation with v4 feature
- **chrono** 0.4 - Date/time handling with serde
- **dirs** 6.0 - XDG standard directories
- **handlebars** 6.0 - Template engine (Handlebars)
- **tera** 1.20 - Template engine (Tera alternative)
- **itertools** 0.14 - Iterator utilities
- **humantime** 2.1 - Human-readable duration formatting
- **once_cell** 1.19 - Lazy static values
- **arc-swap** 1.7 - Atomic Arc swapping
- **notify** 8.2 - File system event watching
- **downcast-rs** 2.0 - Dynamic trait downcasting
- **hostname** 0.4 - Get system hostname
- **hex** 0.4 - Hex encoding/decoding

**Compression:**
- **flate2** 1.0 - DEFLATE compression
- **tar** 0.4 - TAR archive handling
- **seahash** 4.1 - Hash function

**Validation Framework (mcb-validate):**
- **cargo_metadata** 0.23 - Parse Cargo.toml metadata
- **serde_yaml** 0.9 - YAML serialization
- **rust-rule-engine** 1.18 - Rule engine
- **rusty-rules** 0.2 - Rule system
- **evalexpr** 13 - Expression evaluation DSL
- **jsonpath-rust** 1.0.4 - JSONPath navigation
- **pest** 2.8 + **pest_derive** 2.8 - PEG parser for DSL
- **complexity** 0.2 - Cognitive complexity (syn-based)

## Configuration

**Environment Variables:**
- Configuration loading via Figment from TOML + environment
- Prefix: `MCP_` or `MCB_` for config variables
- Key configs:
  - `ADMIN_USERNAME`, `ADMIN_PASSWORD`, `JWT_SECRET` - Admin/auth
  - `EMBEDDING_PROVIDER` - Active embedding provider (ollama|openai|voyageai|gemini|fastembed)
  - `VECTOR_STORE_PROVIDER` - Vector store (milvus|edgevec|filesystem|in-memory)
  - `CACHE_ENABLED` - Enable caching
  - `REDIS_URL` - Redis connection
  - `MILVUS_ADDRESS` - Milvus server address
  - `DATABASE_URL` - PostgreSQL connection
  - `RUST_LOG` - Logging level (tracing-subscriber)

**Build Configuration:**
- `Cargo.toml` - Workspace root with workspace dependencies
- `config/default.toml` - Default application configuration
- `.env.example` - Environment variables template
- `rust-toolchain.toml` - Rust version (nightly)
- `rustfmt.toml` - Code formatting rules
- `clippy.toml` - Linting configuration
- `.qlty/qlty.toml` - Code quality configuration
- `.tarpaulin.toml` - Code coverage configuration
- `.cargo/audit.toml` - Security audit configuration
- `book.toml` - mdBook documentation

**Feature Flags:**
- `embedding-all` - All embedding providers
- `vectorstore-all` - All vector store backends
- `cache-all` - All cache backends
- `lang-all` - All language support (13 languages)
- `full` - Enable everything
- `minimal` - Minimal feature set (Ollama + in-memory + Moka + Rust/Python)

## Platform Requirements

**Development:**
- Rust nightly compiler with clippy, rustfmt
- Cargo workspace manager
- Optional: Docker/Docker Compose for service dependencies
  - Ollama (embeddings)
  - Milvus v2.6.9 (vector database)
  - OpenAI mock server (MockServer)
  - Redis (optional, for caching)
  - NATS (optional, for distributed events)

**Production:**
- Rust 1.92+ runtime
- Async executor (Tokio-based)
- External services (optional, with fallbacks):
  - Embedding service: Ollama (default) | OpenAI | VoyageAI | Gemini | Anthropic
  - Vector store: Milvus (recommended) | EdgeVec | Filesystem (default) | Pinecone | Qdrant
  - Cache: Redis (recommended) | Moka in-memory (default)
  - Database: PostgreSQL (optional, for persistence)
  - Event bus: NATS (optional, for multi-instance)

**Minimum Hardware:**
- 512MB RAM (development)
- 2GB RAM (with Ollama + Milvus)
- 4GB+ (production with Milvus + full indexing)

---

*Stack analysis: 2026-01-31*
