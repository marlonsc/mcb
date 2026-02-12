# handlebars-rust

Last updated: 2026-02-12

## Executive Summary

Handlebars-rust powers MCB's server-side HTML rendering for admin/web views, helper functions, and template orchestration.

## Context7 + External Research

- Context7 ID: `/sunng87/handlebars-rust`
- Docs: https://docs.rs/handlebars/latest/handlebars/
- Upstream: https://github.com/sunng87/handlebars-rust
- Handlebars spec: https://handlebarsjs.com/guide/

## Actual MCB Usage (Current Source of Truth)

### 1) Engine registration and rendering

- `crates/mcb-server/src/templates/engine/handlebars_engine.rs:3`
- `crates/mcb-server/src/templates/engine/handlebars_engine.rs:15`
- `crates/mcb-server/src/templates/engine/handlebars_engine.rs:26`

Pattern: templates are registered once and rendered with serializable view-model context.

### 2) Engine trait boundary and transport coupling

- `crates/mcb-server/src/templates/engine/mod.rs:4`
- `crates/mcb-server/src/templates/template.rs:9`

Pattern: templating concerns stay in server UI layer and do not leak to domain/application contracts.

### 3) Custom helper surface

- `crates/mcb-server/src/admin/web/helpers.rs:8`
- `crates/mcb-server/src/admin/web/helpers.rs:397`

Pattern: helper registration centralizes formatting and view-specific transformations.

## ADR Alignment (Critical)

- ADR-028 (`docs/adr/028-advanced-code-browser-v020.md`): web/admin rendering stack includes template rendering strategy.
- ADR-007 (`docs/adr/007-integrated-web-administration-interface.md`): admin UI depends on server-side route + render integration.
- ADR-026 (`docs/adr/026-routing-refactor-rocket-poem.md`): Rocket + templating integration constraints affect view delivery.

## GitHub Evidence (Upstream + In-Repo)

- Upstream handlebars-rust: https://github.com/sunng87/handlebars-rust
- Tauri production templating example: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/msi/mod.rs
- JSR email templating example: https://github.com/jsr-io/jsr/blob/main/api/src/emails/mod.rs
- In-repo anchor: `crates/mcb-server/src/admin/web/router.rs:14`

## Common Pitfalls

- Registering templates lazily per request instead of once at startup.
- Silent missing-field rendering when strict/validation controls are absent.
- Spreading helper logic across files, making templates hard to maintain.

## References

- https://docs.rs/handlebars/latest/handlebars/
- https://github.com/sunng87/handlebars-rust
- `docs/adr/028-advanced-code-browser-v020.md`
- `docs/adr/007-integrated-web-administration-interface.md`
