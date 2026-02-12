# External Context Notes

Last updated: 2026-02-12

Purpose: keep compact, high-signal references for external libraries used by this repository.

Guidelines:

- One file per library or integration domain.
- Focus on project-specific usage patterns first, then best practices.
- Include version-sensitive caveats and migration notes.
- Prefer official docs and high-quality OSS references.

Current library guides:

- `context/external/tokio.md`
- `context/external/serde.md`
- `context/external/sqlx.md`
- `context/external/thiserror.md`
- `context/external/async-trait.md`
- `context/external/tracing.md`
- `context/external/figment.md`
- `context/external/linkme.md`
- `context/external/dill.md`
- `context/external/git2.md`
- `context/external/tree-sitter.md`
- `context/external/rocket.md`
- `context/external/handlebars.md`
- `context/external/rmcp.md`
- `context/external/clap.md`
- `context/external/moka.md`
- `context/external/mcb-main-libraries-reference.md`

Memory-integration references:

- `context/external/openai-agents-memory.md`
- `context/external/langgraph-memory.md`

## Authoring standard (high-rigor)

Each external document should include all of the following sections:

1. **Usage in MCB**
   - direct crate/module references
   - why this library is used in this architecture
2. **Key APIs in use**
   - concrete APIs/types/macros currently used in code
3. **Project-specific best practices**
   - patterns already followed in this repository
   - patterns we want to enforce going forward
4. **Failure and risk modes**
   - operational failures, performance pitfalls, migration risk
5. **Verification checklist**
   - how to test correctness for that library usage here
6. **References**
   - official docs first, then high-quality OSS examples

## Expected evidence level

Avoid generic library summaries. Every recommendation should be traceable to:

- a real code path in `crates/*`, or
- a documented architecture/plan file in `docs/*`, or
- an official library source/documentation reference.

## Current coverage map

| Library guide | Coverage status | Notes |
|---|---|---|
| `tokio.md` | medium | good runtime notes; can expand cancellation/shutdown patterns |
| `serde.md` | medium | good attrs overview; can add schema-compat migration playbook |
| `sqlx.md` | medium | good basics; can add transaction discipline and query audit patterns |
| `thiserror.md` | medium | strong error taxonomy guidance needed by boundary |
| `async-trait.md` | low/medium | should add trait-object vs generic tradeoff matrix |
| `tracing.md` | medium | should add telemetry cardinality and sensitive-field policy |
| `figment.md` | low/medium | should add deterministic merge precedence and validation gates |
| `linkme.md` | low/medium | should add registration debugging and linker pitfalls |
| `dill.md` | low/medium | should add runtime wiring and test catalog strategy |
| `git2.md` | low/medium | should add spawn_blocking constraints and large-repo handling |
| `tree-sitter.md` | low/medium | should add parser lifecycle, offsets, and query versioning |
| `rocket.md` | medium | should add state mutability strategy and request-guard policy |
| `handlebars.md` | low/medium | should add helper safety and template registration fail-fast policy |
| `rmcp.md` | medium | should add tool schema compatibility and protocol error mapping |
| `clap.md` | medium | should add CLI/config precedence and UX consistency rules |
| `moka.md` | medium | should add cache invalidation and capacity sizing guidance |

## Review cadence

- Revalidate these files when:
  - a dependency major/minor version changes,
  - architecture ADRs affecting library usage are accepted,
  - reliability incidents reveal undocumented pitfalls.

Recommended cadence: every 30 days or on release-branch freeze.

## Quality gates for external docs

Before merging changes to `context/external/*`:

- all text is in English,
- examples reference real repository files,
- no contradictory guidance across documents,
- no stale claims against current `Cargo.toml`,
- links resolve and point to authoritative sources.

## How to use this folder during execution

1. Read `mcb-main-libraries-reference.md` first for global map.
2. Read the specific library guide(s) for the component being changed.
3. Validate current code follows documented pattern.
4. If code diverges, either:
   - update code to match the guide, or
   - update guide with explicit rationale for new pattern.

## Non-goals

- This folder is not an introductory Rust tutorial.
- This folder is not a duplicate of upstream docs.
- This folder should not contain speculative guidance without code evidence.
