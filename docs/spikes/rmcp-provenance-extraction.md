# Spike: rmcp HTTP provenance extraction via Extensions

## Goal

Validate whether `rmcp::transport::http::StreamableHttpService` can expose incoming HTTP headers so MCB can pass provenance fields (`X-Machine-Id`, `X-Session-Id`, etc.) into `build_execution_context()`.

## Decision

**CAN extract headers via Extensions.**

`StreamableHttpService` inserts `http::request::Parts` into MCP request extensions, and handlers can read it through `RequestContext.extensions` (or extractor `Extension<Parts>` in macro-based handlers).

## Evidence

### 1) rmcp transport docs explicitly describe request-part injection

- `StreamableHttpService` docs include: "rest part will remain and injected into Extensions".
- Example shown by rmcp: `Extension(parts): Extension<http::request::Parts>`.

Source:

- <https://docs.rs/rmcp/0.16.0/rmcp/transport/streamable_http_server/tower/struct.StreamableHttpService.html>

### 2) rmcp source confirms insertion of `Parts`

In `src/transport/streamable_http_server/tower.rs`, POST handling does:

```rust
let (part, body) = request.into_parts();
...
req.request.extensions_mut().insert(part);
```

Source:

- <https://docs.rs/crate/rmcp/0.16.0/source/src/transport/streamable_http_server/tower.rs>

### 3) rmcp source confirms handler-side extraction from request context

`Extension<T>` extractor reads from `RequestContext.extensions`:

```rust
let extension = context
    .as_request_context()
    .extensions
    .get::<T>()
    .cloned();
```

Source:

- <https://docs.rs/crate/rmcp/0.16.0/source/src/handler/server/common.rs>

## Minimal working example

### A) HTTP request with custom provenance headers

```http
POST /mcp HTTP/1.1
Content-Type: application/json
Accept: application/json, text/event-stream
X-Session-Id: ses_123
X-Machine-Id: machine_abc
X-Operator-Id: alice

{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search","arguments":{"query":"foo"}}}
```

### B) rmcp handler access via Extensions

```rust
use http::request::Parts;
use rmcp::service::{RequestContext, RoleServer};

fn provenance_from_context(ctx: &RequestContext<RoleServer>) -> (Option<String>, Option<String>) {
    let Some(parts) = ctx.extensions.get::<Parts>() else {
        return (None, None);
    };

    let session_id = parts
        .headers
        .get("x-session-id")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    let machine_id = parts
        .headers
        .get("x-machine-id")
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned);

    (session_id, machine_id)
}
```

### C) Applying this to MCB `build_execution_context()`

Current MCB logic reads `request.meta` and `context.meta` in `crates/mcb-server/src/mcp_server.rs`.

To include transport headers, add a fallback from `context.extensions`:

```rust
use http::request::Parts;

let http_parts = context.extensions.get::<Parts>();
let header_value = |name: &str| {
    http_parts
        .and_then(|parts| parts.headers.get(name))
        .and_then(|v| v.to_str().ok())
        .map(ToOwned::to_owned)
};

let session_id = value(&["session_id", "sessionId", "x-session-id", "x_session_id"])
    .or_else(|| header_value("x-session-id"));

let machine_id = value_or_env(
    &["machine_id", "machineId", "x-machine-id", "x_machine_id"],
    "HOSTNAME",
)
.or_else(|| header_value("x-machine-id"));
```

## Notes and caveats

- Header names are case-insensitive; `HeaderMap::get("x-session-id")` works.
- This works only on HTTP transport path where `StreamableHttpService` is used.
- Keep current meta-based extraction first, then use header fallback to preserve compatibility with non-HTTP transports (stdio, child process).
