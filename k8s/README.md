# Kubernetes Manifests

There are no active Kubernetes manifests in this directory right now.

The previous `k8s/` contents were archived under
`docs/archive/k8s/legacy-manifests.bak/` because they mixed stale image tags,
old TOML configuration, placeholder/plaintext secret guidance, mismatched
ports/metrics, and an imperative `deploy.sh` path forbidden by `AGENTS.md`.

Rebuild Kubernetes support only from current source of truth:

- version/image: `Cargo.toml` and the release pipeline;
- runtime config: `config/*.yaml`;
- commands and GitOps policy: `AGENTS.md`, `Makefile`, and `makefiles/`;
- public MCP/API contract: `docs/MCP_TOOLS.md`;
- secrets: a project-approved GitOps secret backend, not plaintext Kubernetes
  Secret placeholders.

Current rebuild work must be tracked in beads. Start with:

```bash
bd show mcb-wj31
```
