<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# Metrics Module

**Source**: `crates/mcb-infrastructure/src/infrastructure/admin.rs` and `crates/mcb-server/src/admin/`
**Crates**: `mcb-infrastructure`, `mcb-server`

System monitoring, performance tracking, and HTTP metrics API.

## Overview

The metrics functionality is distributed across crates in v0.2.1:

- **mcb-infrastructure**: `AtomicPerformanceMetrics`, `DefaultIndexingOperations` - Performance tracking
- **mcb-server**: Admin endpoints for metrics exposure

### Components

### AtomicPerformanceMetrics (`mcb-infrastructure`)

Thread-safe performance metrics collection:

- Query latency (P50, P95, P99)
- Cache hit/miss rates
- Request throughput
- Error rates

### Metrics Endpoints (`mcb-server`)

HTTP API for metrics access via admin router.

### Endpoints

| Endpoint | Method | Purpose |
| ---------- | -------- | --------- |
| `/alive` | GET | Process liveness without dependency checks |
| `/ready` | GET | Database, migration, embedding, and vector-store readiness |
| `/metrics` | GET | Prometheus text exposition |

## File Structure

```text
crates/mcb-infrastructure/src/infrastructure/
└── admin.rs                 # AtomicPerformanceMetrics, DefaultIndexingOperations

crates/mcb-server/src/admin/
├── handlers.rs              # Metrics endpoint handlers
└── models.rs                # MetricsResponse types
```

## Key Exports

```rust
// From mcb-infrastructure
pub use mcb_infrastructure::infrastructure::{AtomicPerformanceMetrics, DefaultIndexingOperations};

// From mcb-server
pub use admin::{metrics_handler, MetricsResponse};
```

## Configuration

The three operational endpoints are intentionally unauthenticated so Kubernetes
probes and Prometheus can use them without application credentials. They expose
only process state, dependency names/status, request labels, and aggregate job
counts. Administrative and MCP routes retain their existing authentication.

## Cross-References

- **Admin**: [admin.md](./admin.md) (metrics endpoints)
- **Server**: [server.md](./server.md) (HTTP server)
- **Providers**: [providers.md](./providers.md) (metrics implementation)
- **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

---

### Updated 2026-02-12 - Reflects modular crate architecture (v0.2.1)
