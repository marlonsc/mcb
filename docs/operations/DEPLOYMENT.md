# Deployment Guide

This guide covers the current MCB deployment paths. When this document
disagrees with executable source, trust `Cargo.toml`, `Makefile`,
`makefiles/dispatch.mk`, `scripts/lib/mcb.sh`, `config/*.yaml`, and
`AGENTS.md`, then update this guide in the same change.

## Supported Runtime Modes

MCB is a Rust 2024 Loco application exposing MCP over stdio and HTTP.

| Mode | Command | Purpose |
| ---- | ------- | ------- |
| Development | `make check WHAT=dev ACT=run` | Local development server from the workspace |
| MCP stdio | `mcb serve --stdio` | MCP client process transport |
| HTTP daemon | `mcb serve --server` | HTTP/admin runtime without stdio |
| Release install | `make ship WHAT=release ACT=install APPLY=Y` | User service install with config and MCP client updates |

Do not use the removed `config.toml` provider format. Runtime configuration is
Loco YAML under `config/`, with MCB-specific fields under `settings:`.

## Configuration Profiles

| Profile | File | Port | Persistence | Default providers |
| ------- | ---- | ---- | ----------- | ----------------- |
| Development | `config/development.yaml` | `3000` | SQLite | Ollama embeddings, Milvus vector store |
| Test | `config/test.yaml` | dynamic `0` | SQLite test DB | FastEmbed embeddings, EdgeVec vector store |
| Production | `config/production.yaml` | `8080` | SQLite | Ollama embeddings, Milvus vector store |

The public configuration index is `docs/CONFIGURATION.md`; detailed profile and
environment guidance lives under `docs/configuration/`.

## Build And Validate

Use the project Make verbs. They are the command SSOT for build, test,
validation, release, and Git operations.

```bash
make build RELEASE=1
make check WHAT=lint
make test
make check WHAT=validate QUICK=1
```

For release packaging, use:

```bash
make ship WHAT=release ACT=package APPLY=Y
```

For user-local installation, use:

```bash
make ship WHAT=release ACT=install APPLY=Y
make ship WHAT=release ACT=install-validate
```

The install flow builds the release binary, installs MCB under the user's home
directory, writes installed YAML config, updates supported MCP client configs
when present, manages the user `mcb` systemd service, and validates MCP stdio
with an initialize request.

## MCP Client Configuration

For direct stdio integration, configure the MCP client to invoke:

```json
{
  "mcpServers": {
    "mcb": {
      "command": "mcb",
      "args": ["serve", "--stdio"]
    }
  }
}
```

The public MCP contract is documented in `docs/MCP_TOOLS.md`. It currently
exposes 24 public tool names grouped into search, index, memory, session, agent,
validation, VCS, project, and entity families.

## Operational Checks

Use the smallest relevant gate first, then broaden:

```bash
make ship WHAT=release ACT=install-validate
make check WHAT=lint
make test SCOPE=startup
make check WHAT=validate QUICK=1
```

For CI and PR state, use:

```bash
make ship WHAT=pr ACT=checks PR=<number>
```

For Git state, use:

```bash
make ship WHAT=status
make ship WHAT=diff
```

## Kubernetes And GitOps

Kubernetes manifests in `k8s/` are declarative repository artifacts, not an
imperative deployment channel. Steady-state cluster changes must go through Git
and the GitOps controller, following `AGENTS.md`.

Open reconciliation work for the current `k8s/` manifests is tracked in beads,
not in this operations page. Use `bd show mcb-vy4k.5.12` before changing that
lane.

## References

- `AGENTS.md` - project rules, Make verbs, beads workflow, and Git policy.
- `README.md` - user-facing overview and quick start.
- `docs/MCP_TOOLS.md` - public MCP API.
- `docs/CONFIGURATION.md` - configuration index.
- `docs/operations/CHANGELOG.md` - release history.
