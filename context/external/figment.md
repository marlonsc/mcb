# Figment

Last updated: 2026-02-12

## Executive Summary

`figment` is the configuration composition engine used by MCB for layered loading and typed extraction. In this repository, it is used as a strict precedence pipeline (defaults file -> optional override file -> env vars), followed by explicit validation.

Primary implementation:

- `crates/mcb-infrastructure/src/config/loader.rs`
- `crates/mcb-validate/src/config/file_config.rs`

Primary architecture decision:

- `docs/adr/025-figment-configuration.md`

## Context7 + Upstream Capability Notes

Context7 library resolved:

- `/sergiobenitez/figment`

Context7-confirmed behavior that matters for MCB:

- `merge()` gives incoming provider precedence on key conflicts.
- `join()` keeps existing values and only fills missing keys.
- `Env::prefixed("...")` scopes env ingestion to a namespace.
- `Env::split("...")` maps env key segments to nested fields.
- `extract()` deserializes to typed structs with provenance-aware errors.

Implication: in MCB, provider ordering is architecture, not implementation detail.

## Actual MCB Usage (Source of Truth)

### Runtime app config path (`mcb-infrastructure`)

Current chain in `ConfigLoader::load()`:

1. `config/default.toml` (required)
2. Optional `--config` TOML override
3. `MCP__...` environment overrides via `Env::prefixed(...).split("__").lowercase(true)`

This is immediately followed by:

- Typed extraction into `AppConfig`
- Semantic validation (`validate_app_config`)
- Fail-fast behavior on invalid auth/cache/limits/daemon/backup settings

### Validation subsystem path (`mcb-validate`)

Separate figment chain:

1. Embedded defaults (`Toml::string(...)`)
2. Optional filesystem override (`config/mcb-validate-internal.toml`)
3. `MCB_VALIDATE__...` env overrides

This confirms Figment is used beyond one crate, but with a distinct env namespace for validator-specific concerns.

### Tests that enforce behavior

`crates/mcb-infrastructure/tests/unit/config_figment_tests.rs` verifies:

- `MCP__` env loading works
- Legacy `MCB_` keys are rejected
- Auth/JWT validation is enforced
- Legacy `DISABLE_CONFIG_WATCHING` is intentionally ignored

## ADR-Critical Analysis

### ADR-025 is directionally correct, but partially stale

Strong alignment:

- The migration goal (config crate -> Figment) is implemented.
- Layered loading with env override precedence is implemented.
- Validation immediately after extraction is implemented.

Gaps/inconsistencies to track:

1. **Status metadata mismatch**
   - Frontmatter says `status: IMPLEMENTED` but also `implementation_status: Incomplete`.
   - This creates governance ambiguity for contributors and tooling.

2. **Prefix drift in examples**
   - ADR examples use `APP_` while current production config uses `MCP__`.
   - The implementation is correct; the ADR examples are outdated for this repo.

3. **Profile support claim vs practical usage**
   - ADR discusses profile-centric composition as part of migration outcomes.
   - Current `ConfigLoader` path does not use profile selection in the app config pipeline.
   - This is not necessarily wrong, but the ADR should classify profile support as optional/deferred unless implemented.

4. **Scope inflation in ADR narrative**
   - ADR scope text implies broad migration coverage across server/admin/provider areas.
   - In code, central app loading is concentrated in infrastructure loader and validator loader, with Rocket using its own figment path for server template/runtime config concerns.
   - Recommend clarifying "implemented scope" vs "historical/planned scope".

## GitHub Evidence (Upstream + In-Repo)

Upstream Figment behavior references:

- https://github.com/SergioBenitez/Figment/blob/master/src/figment.rs
- https://github.com/SergioBenitez/Figment/blob/master/src/providers/env.rs

Production-grade usage patterns in the Rust ecosystem:

- Rocket config layering with Figment: https://github.com/rwf2/Rocket/blob/master/core/lib/src/config/mod.rs
- Rocket custom figment wiring example: https://github.com/rwf2/Rocket/blob/master/core/lib/src/rocket.rs

In-repo implementation evidence:

- `crates/mcb-infrastructure/src/config/loader.rs`
- `crates/mcb-infrastructure/tests/unit/config_figment_tests.rs`
- `crates/mcb-validate/src/config/file_config.rs`
- `docs/CONFIGURATION.md`
- `docs/adr/025-figment-configuration.md`

## Strengths, Risks, and Tradeoffs

### Strengths

- Deterministic provider precedence with explicit ordering
- Strong fail-fast validation after deserialization
- Test coverage for key migration and compatibility boundaries
- Clear namespace contract for runtime env overrides (`MCP__`)

### Risks

- Documentation drift between ADR examples and production env conventions
- Dual env namespaces (`MCP__` vs `MCB_VALIDATE__`) can confuse operators if not documented together
- ADR status inconsistency can reduce trust in architecture governance metadata

### Tradeoff chosen by MCB

MCB prioritizes strict startup correctness and explicit precedence over permissive fallback behavior. This is a sound choice for operational safety, but it requires disciplined ADR/document synchronization.

## Contributor Checklist (When Touching Config)

1. Keep merge order explicit and deterministic.
2. Preserve env namespace contracts (`MCP__` for app runtime; `MCB_VALIDATE__` for validator runtime).
3. Validate immediately after `extract()`.
4. Update `docs/CONFIGURATION.md` when adding/changing keys.
5. Update ADR-025 when behavior, scope, or lifecycle status changes.
6. Add/adjust tests in `config_figment_tests.rs` for any precedence or env behavior change.

## Anti-Patterns to Avoid

- Introducing direct `std::env::var` bypasses for app config that skip Figment
- Silent fallback to legacy key names after ADR-025 migration
- Undocumented precedence changes in provider ordering
- Marking ADRs as implemented without reconciling frontmatter and code reality

## References

- Context7 library: `/sergiobenitez/figment`
- Figment docs: https://docs.rs/figment
- Figment repository: https://github.com/SergioBenitez/Figment
- ADR template reference set: https://github.com/joelparkerhenderson/architecture-decision-record
- MCB ADR template: `docs/templates/adr-template.md`
- ADR-025: `docs/adr/025-figment-configuration.md`
