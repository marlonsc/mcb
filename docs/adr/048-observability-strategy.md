<!-- markdownlint-disable MD013 MD024 MD025 MD030 MD040 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->
# ADR 048: Gap-Free Observability Strategy

## Status

Accepted

## Target Version

0.2.2

## Detailed Plan

[Observability Strategy Plan](../plans/v0.2.2-observability-strategy.md)

## Context

The user requires a comprehensive observability solution for `mcb` (Metrics, Logs, Traces) that works with OpenTelemetry.
**Constraints**:

- **Transparency**: Minimal changes to business logic.
- **Completeness**: Must include "indicators" (metrics) for dashboards.
- **Configurability**: "Minimal" default to "Max" visibility.
- **Architecture**: Must cover Stdio, HTTP, and Async Event Bus.

## Decision

We will implement a **Unified Observability Layer** based on `tracing` ecosystem, leveraging "Auto-Instrumentation" pattern to derive Metrics and Traces from standard execution spans.

### 1. Component Architecture

| Signal | Source | Implementation Mechanism |
| :--- | :--- | :--- |
| **Logs** | `info!`, `error!` | `tracing-subscriber::fmt` (Existing) |
| **Traces** | `#[instrument]` | `opentelemetry-otlp` + `tracing-opentelemetry` |
| **Metrics** | Spans | `tracing-opentelemetry::MetricsLayer` (Derives RED metrics) |

**Why `MetricsLayer`?**
Instead of manually injecting `counter!("tool_calls", 1)` into every function (which violates "minimal code change"), we effectively "turn spans into metrics". The duration, count, and status (Ok/Err) of the `call_tool` span become our Golden Signals.

### 2. Configuration Strategy (`mcb.toml`)

```toml
[observability]
enabled = true
level = "minimal" # off | minimal | debug | trace | max

[observability.otlp]
enabled = false
endpoint = "http://localhost:4317"
protocol = "grpc"
```

### 3. Critical Instrumentation Points

To achieve "gap-free" visibility, we instrument the **Edges** and the **Core**:

1. **HTTP Edge (`http.rs`)**:
    - **Mechanism**: `TracingFairing`.
    - **Role**:  Extracts W3C Trace Context from HTTP headers. Ensures web requests are part of distributed traces.

2. **Stdio Edge / Core (`mcp_server.rs`)**:
    - **Mechanism**: `#[instrument]` on `call_tool`. manual context extraction from `meta`.
    - **Role**: The "Root Span" for CLI/IDE interactions. It bridges the air-gap of Stdio by reading provenance from JSON-RPC.

3. **Async Core (`events/tokio.rs`)**:
    - **Mechanism**: `#[instrument]` on `publish_event`.
    - **Role**: Traces the "fire-and-forget" background events, linking causes (tool calls) to effects (indexing updates).

### 4. Technical Specifications

#### Dependencies (Verified Compatibility)

```toml
[dependencies]
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-otlp = "0.15"
tracing-opentelemetry = "0.23"
opentelemetry-http = "0.11"
```

#### Level Behavior

| Level | Logs | Traces | Metrics |
| :--- | :--- | :--- | :--- |
| **Off** | Error Only | Disabled | Disabled |
| **Minimal** | Info | **Critical Only** (Root Spans) | **Default** (Throughput/Latency) |
| **Debug** | Debug | Sampled (10%) | Extended |
| **Trace** | Trace | 100% | Full |
| **Max** | Trace | 100% + Payload Attributes | Full |

## Consequences

### Positive

- **Zero-Code Metrics**: We get dashboard-ready metrics (RPS, Latency) just by adding one attribute to `McpServer`.
- **Full Context**: We trace across boundaries (HTTP -> Server -> EventBus).
- **Standard Compliance**: Fully OTLP compatible.

### Negative

- **Binary Size**: OTel libraries are large.
- **Build Time**: Compiling `prost` (gRPC) takes time.

## Compliance

- **Target**: v0.2.2
- **Gap Analysis**: Addressed Stdio context gap and Metrics gap.
