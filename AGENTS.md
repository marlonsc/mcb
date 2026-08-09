# AGENTS.md — mcb

> **Parent workspace law** lives in [`../AGENTS.md`](../AGENTS.md) — read it first.
> Universal engineering core: `~/.agents/UNIVERSAL_CORE.md`. Composition: global skills + parent/root `AGENTS.md` + this scope delta. Do not re-embed universal law.
>
> **Standalone / independent mode:** when `../AGENTS.md` does not resolve, pin the parent raw `AGENTS.md` URL to the same branch/release as this package (never `main`).

<!-- AIHUB-AGENTS-SCOPE-LOCAL-BEGIN -->
## Commands

The whole dev cycle runs through few canonical `make` verbs backed by the single
monopoly script `scripts/lib/mcb.sh`. Pattern: `make <verb> [WHAT=phase]
[SCOPE=...] [APPLY=Y]`. Do not call `cargo`/`git` directly — use a verb. Run
`make help` for the live list.

```bash
make help                          # All verbs + their WHAT= phases
make build [RELEASE=0|1]           # Release build by default
make dev   [WHAT=run|docker-up|docker-down|docker-logs|docker-test]
make test  [SCOPE=unit|doc|golden|startup|integration|e2e|all] [THREADS=N]
make check [WHAT=fmt|lint|validate|audit|udeps|coverage|qlty|all] [QUICK=1]
make fix   [WHAT=fmt|lint|docs|all]   # Mutating auto-fix (rustfmt, clippy --fix, markdown)
make docs  [WHAT=build|serve|lint|validate|sync|rust|check|setup|adr|adr-new|diagrams] [QUICK=1] [FIX=1]
make ci                            # CI gate (check WHAT=all)
make guard                         # Banned-pattern scanner (prod unwrap/expect/panic/todo, TODO/FIXME, unjustified #[allow])
```

Read-only git / PR / submodule inspection flows through the same monopoly:

```bash
make git WHAT=status|diff|log|show|branch|tags|stash-list
make pr  WHAT=view|checks PR=<n>
make sub WHAT=status|diff
```

Single-test local debugging is allowed when it is materially faster than the
verb:

```bash
cargo test -p mcb-server --test unit -- test_name
```

Destructive verbs are DRY-RUN by default and require `APPLY=Y` to execute:

```bash
make codegen [WHAT=all|cli|db|entities|conversions|clean] APPLY=Y
make release [WHAT=package|version|install|install-validate] [BUMP=patch|minor|major] APPLY=Y
make clean   [WHAT=build|codegen|all] APPLY=Y
make git WHAT=commit MSG='...' [FILES='...'] APPLY=Y   # also push|merge|rebase
make sub WHAT=commit|push SUB=<name> [MSG='...'] APPLY=Y
make setup [WHAT=hooks|tools|adr|all]                  # hooks installs the pre-commit gate
```

`make release WHAT=install APPLY=Y` builds, installs config under the user's home
directory, updates MCP client configs when present, and manages the user `mcb`
systemd service. Run it only when the user explicitly asks for installation work.

Enforcement is mechanical, not honor-system: `make setup WHAT=hooks` installs a
no-bypass pre-commit hook (staged `guard` + `check WHAT=lint` + `check
WHAT=validate QUICK=1`); `.claude/settings.json` denies dangerous shell and
routes every Bash through `scripts/lib/mcb.sh guard-bash`; `make guard` scans the
full tree (CI/manual) while the hook's `guard --staged` blocks only NEW
violations in the commit.

## Architecture

Clean Architecture is enforced by dependency rules and `mcb-validate`.

```text
mcb                 # CLI facade binary
  -> mcb-server     # MCP protocol, Axum HTTP, handlers, admin UI
    -> mcb-infrastructure
       # DI/linkme + AppContext, Loco config, cache, logging, tracing
      -> mcb-domain # entities, value objects, port traits, errors
  -> mcb-providers  # adapters for embedding, vector store, DB, git, parsers
  -> mcb-validate   # architecture rule engine and analysis CLI
  -> mcb-utils      # shared leaf utilities
```

Dependency rules:

- `mcb-domain`: zero internal dependencies.
- `mcb-providers`: implements domain ports; depends on `mcb-domain` and
  `mcb-utils`.
- `mcb-infrastructure`: composition and runtime wiring; can use domain,
  providers, and utils.
- `mcb-server`: entrypoint and handlers; use services through DI ports.
- `mcb-utils`: leaf crate; no `mcb-*` dependencies.
- `mcb-validate`: developer tooling; keep runtime coupling deliberate and
  covered by validation config.

Do not import lower-level concrete providers directly into handlers. Add or
reuse a domain port, wire the adapter in infrastructure, and resolve through
the catalog/context.

## Runtime Configuration

MCB uses Loco YAML configuration. Loco-native sections are `logger`, `server`,
`database`, and `cache`; MCB-specific settings live under `settings:` and are
deserialized into `AppConfig`.

Profiles:

- Development: `config/development.yaml`, port `3000`, SQLite, Ollama
  embeddings, Milvus vector store.
- Test: `config/test.yaml`, dynamic port `0`, SQLite, FastEmbed embeddings,
  EdgeVec vector store, destructive test DB flags enabled.
- Production: `config/production.yaml`, port `8080`, SQLite, Ollama
  embeddings, Milvus vector store, admin API key header enabled.

Do not hardcode configuration values in code. Add fields to the typed config
model and populate every profile.

## MCP Tooling

The public MCP interface is 24 tool names grouped into 9 handler families:

- Search: `search_code`, `search_memory`
- Index: `index_repo`, `index_status`, `clear_index`
- Memory: `store_memory`, `get_memories`, `list_memories`,
  `memory_timeline`, `inject_context`
- Session: `start_session`, `get_session`, `list_sessions`,
  `summarize_session`
- Agent: `log_tool_call`, `log_delegation`
- Validation: `validate_code`, `analyze_code`, `list_rules`
- VCS: `list_repos`, `compare_branches`, `analyze_impact`
- Compound project/entity: `project`, `entity`

Handlers and schemas are split across `crates/mcb-server/src/args/`,
`crates/mcb-server/src/handlers/`, and `crates/mcb-server/src/tools/`.
Context/provenance fields are injected where the schema marks them hidden.

When changing a tool:

1. Update the args schema and validator.
2. Update the handler.
3. Update `docs/MCP_TOOLS.md` if the public contract changed.
4. Add or update focused tests for the action/resource touched.

## Implementation Rules

- Keep edits surgical and scoped to the user request.
- Prefer existing macros and patterns: `tool_action!`, `tool_schema!`,
  `tool_enum!`, `register_tool!`, `linkme` distributed slices, and the Handle
  pattern.
- Use `Error` constructors and `Result` aliases from `mcb-domain`; do not build
  raw domain errors by hand.
- Use `?` for propagation. No `unwrap()`, `expect()`, `panic!`, `todo!`, or
  `unimplemented!` in production paths.
- Keep imports ordered: `std`, external crates, `mcb_*` crates, local modules.
- Keep generated docs and reports fixed at the generator/template.
- Keep first-party source files compact; split modules before they become
  difficult to review.

## Testing And Verification

After meaningful edits, run the smallest relevant gate first, then broaden when
the change touches shared behavior:

- Rust code: `make check WHAT=lint` plus the relevant `make test SCOPE=...`.
- Architecture rules, dependencies, or crate boundaries: add
  `make check WHAT=validate QUICK=1` or `make check WHAT=validate`.
- Docs-only changes: `make docs WHAT=lint`.
- Public docs plus architecture/status changes: `make docs WHAT=validate QUICK=1`
  when practical.
- Release/install paths: `make release APPLY=Y` only when explicitly requested.

Report command, exit code, and the meaningful output. Do not claim a full gate
passed unless that exact gate was run in the current turn.

## Documentation Pointers

- `AGENTS.md`: project-canonical agent instructions.
- `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md`: thin pointers
  back to this file.
- `README.md`: user-facing overview and quick start.
- `docs/MCP_TOOLS.md`: public MCP API.
- `docs/CONFIGURATION.md`: configuration index.
- `docs/developer/ROADMAP.md`: roadmap; verify against source before relying
  on static status.
- `docs/architecture/ARCHITECTURE.md`: architecture overview and historical
  context.
<!-- AIHUB-AGENTS-SCOPE-LOCAL-END -->
