# Code Review

Last updated: 2026-02-15 (America/Sao_Paulo)

## Purpose

MCB-specific review checklist aligned with Clean Architecture and CI gates.

## Architecture Checks (Critical)

- [ ] Dependencies flow inward: `server -> infra -> app -> domain`
- [ ] Domain crate has zero infra imports (no `sqlx`, `rocket`, `git2`)
- [ ] Port traits defined in `mcb-domain`, not `mcb-application`
- [ ] Providers implement domain ports, register via `linkme`
- [ ] No circular dependencies between crates

## Code Quality Checks

- [ ] `make lint` passes (clippy + fmt)
- [ ] No `#[allow(unused)]` hiding real issues
- [ ] No `.unwrap()` in non-test code (use `?` or `McpError`)
- [ ] Error types use `thiserror` constructors, not raw strings
- [ ] No `to_string_lossy()` — handle encoding properly
- [ ] No hardcoded paths or fallback values (use `ConfigLoader`)
- [ ] Async functions use `tokio` runtime, not `block_on` in async context

## Testing Checks

- [ ] New logic has corresponding tests
- [ ] `make test` passes (or pre-existing failures documented)
- [ ] Integration tests don't depend on network by default
- [ ] Test helpers use `rstest` fixtures where applicable
- [ ] Golden tests updated if output format changed

## Configuration Checks

- [ ] New config fields added to `config/default.toml`
- [ ] Config types in `mcb-infrastructure/src/config/types/`
- [ ] Sensitive fields use `#[serde(skip_serializing)]` or redaction
- [ ] Environment variable overrides follow `MCB_` prefix convention

## PR Description Expectations

- **What** changed (files, modules, scope)
- **Why** it was needed (issue ref, problem statement)
- **How** it was validated (which `make` targets, manual tests)
- **Impact** on config, migrations, or API surface

## CI Gate Requirements

| PR Type | Required Gates |
|---------|---------------|
| Draft | None (all skipped) |
| Bot (Dependabot) | lint + test + startup + validate |
| Human (ready) | lint + test + startup + validate + audit + golden + coverage + release-build |

## `make validate` Violation Codes

| Code | Rule | Fix |
|------|------|-----|
| CA001 | Layer violation | Move import to correct layer |
| CA002 | Circular dependency | Use DI/ports to break cycle |
| CA007 | Port duplication | Move trait to `mcb-domain` |
| ORG016 | File placement | Move file to correct directory |
| ORG019 | Ignore pattern | Update validation config |

## Sources

- `.github/workflows/ci.yml` (CI gates)
- `crates/mcb-validate/` (architecture validator)
- `make/quality.mk` (quality targets)

## Update Notes

- 2026-02-15: Full rewrite with MCB-specific checks, CI gates, violation codes.
- 2026-02-11: Initial generic placeholder.
