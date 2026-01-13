# Metrics Module

**Source**: `src/infrastructure/metrics/`

System monitoring, performance tracking, and HTTP metrics API.

## Overview

The metrics module provides comprehensive observability for the MCP Context Browser. It collects system metrics (CPU, memory, disk), tracks query performance, and exposes a REST API for monitoring dashboards.

## Components

### SystemMetricsCollector (`system.rs`)

Collects system-level metrics using `sysinfo` crate.

\1-   CPU usage and load averages
\1-   Memory utilization (used/total/available)
\1-   Disk I/O and storage capacity
\1-   Network statistics

### PerformanceMetrics (`performance.rs`)

Tracks application performance.

\1-   Query latency (P50, P95, P99)
\1-   Cache hit/miss rates
\1-   Request throughput
\1-   Error rates

### MetricsApiServer (`http_server.rs`)

HTTP API for metrics access (port 3001).

**Endpoints**:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/health` | GET | Health check |
| `/api/metrics` | GET | Prometheus-format metrics |
| `/api/context/metrics` | GET | Application metrics JSON |

### CacheMetrics

Cache performance tracking.

\1-   Hit count / miss count
\1-   Hit rate percentage
\1-   Eviction statistics

## File Structure

```text
src/infrastructure/metrics/
├── http_server.rs   # REST API server
├── mod.rs           # Module exports
├── performance.rs   # Query performance tracking
└── system.rs        # System metrics collection
```

## Key Exports

```rust
pub use http_server::{MetricsApiServer, HealthResponse};
pub use performance::{PerformanceMetrics, CacheMetrics, QueryPerformanceMetrics};
pub use performance::PERFORMANCE_METRICS;
```

## Configuration

Environment variables:

\1-   `MCP_METRICS_ENABLED=true` - Enable metrics collection
\1-   `MCP_PORT=3001` - Unified HTTP port (Admin + Metrics + MCP)

## Testing

5 metrics tests. See [tests/metrics.rs](../../tests/metrics.rs).

## Cross-References

\1-  **Architecture**: [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)
\1-  **Server**: [server.md](./server.md) (integrates metrics)
\1-  **Admin**: [admin.md](./admin.md) (metrics dashboard)
