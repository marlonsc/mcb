# External Integrations

**Analysis Date:** 2026-01-31

## APIs & External Services

**Embedding Providers (AI/ML):**
- **Ollama** (default) - Local self-hosted embedding service
  - SDK/Client: HTTP via `reqwest`, no official Rust SDK
  - Environment: `OLLAMA_BASE_URL` (default: http://localhost:11434)
  - Model config: `OLLAMA_MODEL` (default: nomic-embed-text)
  - Implementation: `crates/mcb-providers/src/embedding/ollama.rs`
  - Health check: simple embed operation on local model

- **OpenAI** - Enterprise embedding service
  - SDK/Client: `reqwest` HTTP client (official SDK not used)
  - Environment: `OPENAI_API_KEY`, `OPENAI_BASE_URL` (optional custom endpoint)
  - Model: text-embedding-3-small (configurable)
  - Implementation: `crates/mcb-providers/src/embedding/openai.rs`
  - Auth: Bearer token in Authorization header
  - Rate limiting: respects OpenAI rate limits

- **VoyageAI** - Alternative embedding provider
  - SDK/Client: `reqwest` HTTP client
  - Environment: `VOYAGEAI_API_KEY`
  - Implementation: `crates/mcb-providers/src/embedding/voyageai.rs`
  - Auth: API key in header

- **Google Gemini** - Google's embedding service
  - SDK/Client: `reqwest` HTTP client
  - Environment: `GEMINI_API_KEY`
  - Implementation: `crates/mcb-providers/src/embedding/gemini.rs`
  - Endpoint: Google AI Studio API

- **Anthropic** - Anthropic's embedding service
  - SDK/Client: `reqwest` HTTP client
  - Environment: `ANTHROPIC_API_KEY`
  - Implementation: `crates/mcb-providers/src/embedding/anthropic.rs`
  - Auth: API key authentication

- **FastEmbed** - Local Rust embedding library (default fallback)
  - SDK/Client: fastembed crate 5.8 (pure Rust, no external service)
  - No environment variables required
  - Implementation: `crates/mcb-providers/src/embedding/fastembed.rs`
  - Benefit: Works offline, no API key needed
  - Models: ONNX-based, runs locally

## Data Storage

**Vector Databases:**
- **Milvus** - Enterprise cloud/self-hosted vector database (production recommended)
  - Connection: `MILVUS_ADDRESS` (default: localhost:19530)
  - Auth: `MILVUS_TOKEN` (optional)
  - SDK: milvus-sdk-rust 0.1.0 from git (fixes lifetime bug)
  - Implementation: `crates/mcb-providers/src/vector_store/milvus.rs`
  - Features: Collection management, vector insert/search, metadata filtering
  - Docker service: milvusdb/milvus:v2.6.9 with etcd and MinIO dependencies
  - Used for: Semantic vector storage at scale

- **EdgeVec** - High-performance in-memory vector store
  - Configuration: In-memory, no external service
  - SDK: edgevec 0.8 crate
  - Implementation: `crates/mcb-providers/src/vector_store/edgevec.rs`
  - Features: Fast HNSW indexing, on-disk persistence
  - Used for: High-speed searches, embedded deployments

- **Filesystem** - Local filesystem-based vector storage (default fallback)
  - Configuration: `address = "./data/vectors"` in config/default.toml
  - Implementation: `crates/mcb-providers/src/vector_store/filesystem/`
  - Features: No external service, works offline, easy backup
  - Used for: Development, lightweight deployments

- **Pinecone** - Serverless vector database (REST API)
  - Connection: `PINECONE_API_KEY`, `PINECONE_ENVIRONMENT`, `PINECONE_INDEX_NAME`
  - SDK: REST via `reqwest`
  - Implementation: `crates/mcb-providers/src/vector_store/pinecone.rs`
  - Auth: API key based
  - Used for: Fully managed vector search service

- **Qdrant** - Vector database with REST/gRPC
  - Connection: `QDRANT_URL`, `QDRANT_API_KEY` (optional)
  - SDK: REST via `reqwest`
  - Implementation: `crates/mcb-providers/src/vector_store/qdrant.rs`
  - Default: http://localhost:6333
  - Auth: API key in headers if configured
  - Used for: Self-hosted or cloud vector database

- **In-Memory** - Testing and development
  - No external configuration
  - Implementation: `crates/mcb-providers/src/vector_store/in_memory.rs`
  - Features: Fast, no persistence, for testing only

**Encrypted Vector Store Wrapper:**
- AES-256-GCM encryption wrapper around any vector store
- Implementation: `crates/mcb-providers/src/vector_store/encrypted.rs`
- Feature flag: `vectorstore-encrypted`

**SQL Database (Optional):**
- **PostgreSQL** - Optional relational database for metadata
  - Connection: `DATABASE_URL` (e.g., postgresql://user:password@localhost:5432/mcp)
  - Client: r2d2 connection pool (r2d2_postgres)
  - Pool config:
    - `DATABASE_MAX_CONNECTIONS` (default: 20)
    - `DATABASE_MIN_IDLE` (default: 5)
    - `DATABASE_MAX_LIFETIME_SECS` (default: 1800)
    - `DATABASE_IDLE_TIMEOUT_SECS` (default: 600)
    - `DATABASE_CONNECTION_TIMEOUT_SECS` (default: 30)
  - Fallback: Filesystem or in-memory if not configured
  - Used for: Persistence layer (optional, not required for basic operation)

**File Storage:**
- **Filesystem (Local)** - Default for vector storage and application data
  - Data directories follow XDG standard: `~/.local/share/mcb/`
  - Subdirectories: `snapshots/`, `config-history/`, `encryption/`, `circuit-breakers/`
  - Implementation: Configurable via `data.base_dir` in config
  - Used for: Vector storage, snapshots, configuration backups

**Caching:**
- **Moka** - In-memory cache (default)
  - Configuration: `cache.backend.local` in config
  - Default TTL: 3600 seconds (1 hour), configurable per namespace
  - Max entries: 10000 (configurable)
  - Features: TTL support, eviction policies, compression
  - Implementation: `crates/mcb-providers/src/cache/moka.rs`
  - Namespaces: embeddings, search_results, metadata, provider_responses, sync_batches

- **Redis** - Distributed cache for multi-instance deployments
  - Connection: `REDIS_URL` (e.g., redis://localhost:6379/0)
  - Client: redis 1.0 with tokio-comp, connection-manager
  - Pool size: configurable (default: 10)
  - Default TTL: 3600 seconds
  - Implementation: `crates/mcb-providers/src/cache/redis.rs`
  - Used for: Multi-instance caching, distributed setups
  - Features: Same namespace support as Moka

- **Null Cache** - No-op cache for testing
  - Implementation: `crates/mcb-providers/src/cache/null.rs`
  - Used for: Tests where caching should be disabled

## Authentication & Identity

**Admin Interface (Web UI):**
- JWT-based authentication for admin dashboard
- Environment credentials:
  - `ADMIN_USERNAME` - Username for admin access
  - `ADMIN_PASSWORD` - Password (minimum 8 characters)
  - `MCP_ADMIN_JWT_SECRET` - JWT signing secret (minimum 32 characters)
  - `JWT_EXPIRATION` - Token lifetime in seconds (default: 3600 = 1 hour)
- Implementation: `crates/mcb-server/src/admin/auth.rs`
- Token generation: bcrypt hashing for password verification
- Session: HTTP cookies with JWT tokens
- Web UI: Rocket-based dashboard at `/admin` endpoint

**API Authentication:**
- Optional JWT authentication for API endpoints
- Same JWT_SECRET used as admin
- Disabled by default, enabled via `auth.enabled` in config
- Bearer token in Authorization header: `Authorization: Bearer <token>`
- Implementation: `crates/mcb-server/src/auth.rs`

**No Default Auth:**
- API accessible without authentication if not configured
- Admin dashboard requires credentials if enabled
- Security: Must set credentials in production via environment variables
- Failure mode: Server refuses to start if admin enabled without credentials

## Monitoring & Observability

**Metrics & Health Checks:**
- **Prometheus** - Metrics exporter
  - Exporter: metrics-exporter-prometheus 0.18
  - Endpoint: HTTP server on METRICS_PORT (default: 3001)
  - Metrics exposed at `/metrics` endpoint
  - Dashboard: Admin UI provides built-in metrics dashboard

- **Health Checks:**
  - Endpoint: `/health` (HTTP, main port 3000)
  - System health: CPU, memory, disk usage monitoring
  - Provider health: Each provider implements `health_check()` method
  - Implementation: `crates/mcb-application/src/use_cases/` and providers
  - Monitored via: sysinfo 0.38, nix 0.31 utilities

**Logging:**
- **Structured Logging** - Tracing framework
  - Configuration: `RUST_LOG` environment variable
  - Levels: trace, debug, info, warn, error
  - Output format: JSON (via tracing-subscriber json feature)
  - Implementation: `crates/mcb-server/src/` all modules
  - Appenders: file and stdout via tracing-appender

- **Log Persistence:**
  - Application can log to files via appenders
  - Default: stderr output to console
  - Configurable: Via RUST_LOG patterns (e.g., `info,mcb_context_browser=debug`)

**Error Tracking:**
- No external error tracking service (Sentry not used)
- Error handling: Custom error types with `thiserror`
- Implementation: `crates/mcb-domain/src/error.rs`
- Propagation: Structured logging captures all errors

## CI/CD & Deployment

**Hosting Platforms (Supported):**
- **Docker** - Container deployment
  - Dockerfile: tests/docker/Dockerfile.test for testing
  - Docker Compose: Orchestrates Ollama, Milvus, OpenAI mock, test runner
  - Image: Builds from project root with custom test runner

- **Kubernetes** - Cloud-native deployment
  - K8s manifests: `k8s/` directory
  - ConfigMaps: Configuration files
  - Deployments: MCP server service definition
  - Services: Expose MCP server

- **Standalone Binary** - Direct execution
  - `cargo build --release` produces executable
  - No runtime dependencies except external services (optional)
  - Supports stdio and HTTP transports

**CI Pipeline (GitHub Actions):**
- Configuration: `.github/` directory
- Triggers: Push, PR on main branch
- Workflow: Tests, linting, code coverage

**Build Configuration:**
- Release profile: LTO enabled, single codegen unit for optimization
- Development profile: Incremental compilation for speed

## Environment Configuration

**Required Environment Variables (Production with Admin):**
```
ADMIN_USERNAME=<username>
ADMIN_PASSWORD=<min-8-chars>
MCP_ADMIN_JWT_SECRET=<min-32-chars>
EMBEDDING_PROVIDER=ollama|openai|voyageai|gemini|fastembed
VECTOR_STORE_PROVIDER=milvus|edgevec|filesystem|in-memory
```

**Optional Environment Variables:**
```
# Ollama
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=nomic-embed-text

# OpenAI
OPENAI_API_KEY=sk-...
OPENAI_BASE_URL=https://api.openai.com/v1

# VoyageAI
VOYAGEAI_API_KEY=...

# Gemini
GEMINI_API_KEY=...

# Anthropic
ANTHROPIC_API_KEY=...

# Milvus
MILVUS_ADDRESS=localhost:19530
MILVUS_TOKEN=...

# Pinecone
PINECONE_API_KEY=...
PINECONE_ENVIRONMENT=...
PINECONE_INDEX_NAME=...

# Qdrant
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=...

# PostgreSQL
DATABASE_URL=postgresql://user:password@host:5432/db

# Redis
REDIS_URL=redis://localhost:6379/0

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
METRICS_PORT=3001

# Logging
RUST_LOG=info,mcb=debug

# Rate Limiting
RATE_LIMITING_ENABLED=false
RATE_LIMIT_RPM=100
```

**Secrets Location:**
- Environment variables (recommended for production)
- `.env` file (development only, never commit)
- Configuration file: config/default.toml (development defaults only)
- Docker secrets: Kubernetes/Docker secret mounting

## Webhooks & Callbacks

**Incoming Webhooks:**
- No external webhooks configured by default
- Admin dashboard can initiate operations: index, search, clear cache
- File watching: Optional daemon monitors file changes for re-indexing
  - Configuration: `sync.debounce_ms`, `sync.interval_ms`
  - File watcher: notify 8.2 crate
  - Implementation: `crates/mcb-application/src/use_cases/` sync operations

**Outgoing Webhooks:**
- No outgoing webhooks to external services
- Event publishing: Optional NATS event bus for internal pub/sub
  - Configuration: `MCP_EVENT_BUS_TYPE=nats`
  - Client: async-nats 0.46
  - Used for: Multi-instance coordination (optional)
  - Implementation: `crates/mcb-providers/src/events/nats.rs`

**Event Bus (Internal):**
- **NATS** - Optional pub/sub for distributed events
  - Connection: `NATS_URL` environment variable
  - Implementation: `crates/mcb-providers/src/events/nats.rs`
  - Used for: Multi-instance indexing coordination, cache invalidation
  - Fallback: Tokio-based local event bus if NATS not available

- **Tokio Event Bus** - In-process pub/sub
  - Implementation: `crates/mcb-providers/src/events/tokio.rs`
  - Default: Used when NATS not configured
  - Used for: Single-instance event propagation

## Security Considerations

**API Keys Storage:**
- Configured via environment variables only
- Never committed to git
- Template: `.env.example` for documentation
- Encryption: Optional AES-256-GCM for vector store data

**Password Handling:**
- bcrypt hashing for admin password (bcrypt 0.18)
- Argon2 available as alternative (argon2 0.5)
- PBKDF2 for key derivation (pbkdf2 0.12)
- HMAC for message authentication (hmac 0.12)

**Transport Security:**
- TLS support via reqwest (feature: "__tls")
- HTTPS/TLS configurable for HTTP transport
- Stdio transport: direct process communication

**Rate Limiting:**
- Optional rate limiter configurable
- In-memory or Redis-backed rate limiting
- Configuration: `metrics.rate_limiting` in config
- Features: Request window limiting, burst allowance

## Third-Party Service Dependencies

| Service | Type | Required | Fallback |
|---------|------|----------|----------|
| Ollama | Embedding | No | FastEmbed (local) |
| OpenAI | Embedding | No | FastEmbed (local) |
| Milvus | Vector Store | No | Filesystem/In-Memory |
| PostgreSQL | Database | No | Filesystem |
| Redis | Cache | No | Moka (in-memory) |
| NATS | Events | No | Tokio (local) |

**Notes:**
- All external services are optional
- Application works offline with local defaults
- No service required to start application
- Each provider has sensible fallbacks
- Docker Compose: Includes all test services

---

*Integration audit: 2026-01-31*
