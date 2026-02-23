# loco-rmcp-spike

Throwaway spike that runs a Loco HTTP server and an rmcp stdio MCP server in the same Tokio runtime.

## What this validates

- `Hooks::after_routes()` is a workable place to spawn an MCP stdio server task.
- Loco handlers and MCP tool handlers can share one `DatabaseConnection`.
- HTTP writes are visible to MCP reads immediately (same SQLite file + same runtime).

## Run

```bash
cargo build -p loco-rmcp-spike
SPIKE_HTTP_PORT=3017 SPIKE_DB_PATH=./spike.sqlite cargo run -p loco-rmcp-spike
```

HTTP endpoints:

- `GET /health` -> `200 {"ok":true}`
- `POST /notes` -> inserts one row (`http-write`) and returns `note_count`

MCP tool:

- `index` (no args): returns JSON text payload with `note_count`

## Lifecycle notes

1. `main()` builds Loco config and starts app in `StartMode::ServerOnly`.
2. During boot, `after_routes()` clones `ctx.db` and `tokio::spawn`s MCP stdio server.
3. Loco HTTP server runs in foreground; MCP server waits on stdio in background task.
4. Shutdown currently follows Loco server lifecycle. MCP task is not explicitly joined; it stops when process exits.

## Shutdown ordering (current spike behavior)

- Primary shutdown driver is Loco's HTTP graceful shutdown.
- MCP task is detached and exits when process exits or stdio closes.
- For production integration, use a shared cancellation token + `JoinHandle` tracking to explicitly stop MCP before process exit.

## Config merging approach

This spike uses code-first config (no YAML files):

- `SPIKE_HTTP_PORT` env var overrides port.
- `SPIKE_DB_PATH` env var overrides SQLite file path.
- Remaining Loco config values are hardcoded minimal defaults for local validation.

Recommended production approach:

- Keep Loco YAML/env loading as source of truth.
- Add MCP-specific section (`mcp.enabled`, transport, limits).
- Merge env overrides last for both HTTP and MCP.

## Important version caveat

Loco `0.16.4` currently depends on SeaORM `1.1.x`. This spike therefore compiles with SeaORM `1.1.19` to keep the DB connection type shared across Loco and custom code.

If your target branch requires SeaORM `=2.0.0-rc.34`, options are:

1. Move to a Loco version that supports SeaORM 2.x.
2. Keep this integration pattern but run the spike in an isolated crate/workspace and pin versions per executable.
