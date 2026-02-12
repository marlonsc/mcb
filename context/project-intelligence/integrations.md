# Integrations

Last updated: 2026-02-11 (America/Sao_Paulo)

## Internal Integrations

- MCP tool surface exposes indexing, search, memory, validation, VCS,
  sessions, agent logging, and project workflows.
- Providers layer integrates embedding models and vector stores behind traits.
- Documentation and validation pipelines are integrated through Make targets.

## Data/Context Integrations

- Vector search + metadata + git context combine for retrieval quality.
- Freshness and snapshot versioning are integrated design constraints in ADR-041/045.

## Tooling Integrations

- Beads issue tracking syncs with git workflow (`bd sync`).
- Docs tooling includes lint/sync/validate scripts for consistency checks.

## Sources

- `README.md`
- `docs/context/integrations.md`
- `docs/adr/041-integrated-context-system-architecture.md`
- `docs/adr/045-context-versioning-freshness.md`
- `docs/BEADS_QUICK_REFERENCE.md`

## Update Notes

- 2026-02-11: Integrated repository and ADR signals into a compact integration map.
