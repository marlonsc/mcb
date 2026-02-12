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

## Best Practices in MCB

### Template registration at startup

MCB registers all Handlebars templates once during server initialization through the `HandlebarsEngine` (`crates/mcb-server/src/templates/engine/handlebars_engine.rs:15`). Templates are loaded from disk, compiled, and stored in the engine instance.

Never register templates lazily per-request. Template compilation is expensive and should happen at boot time.

### Helper centralization

All custom Handlebars helpers are defined in `crates/mcb-server/src/admin/web/helpers.rs`. This file is the single source of truth for template functions available in admin views. Adding a helper requires registering it in this file.

Keep helper implementations pure functions that format or transform data. Helpers should not perform I/O, database queries, or service calls.

### Engine trait boundary

MCB wraps Handlebars behind a `TemplateEngine` trait (`crates/mcb-server/src/templates/engine/mod.rs:4`). This abstraction allows testing with mock engines and theoretically swapping to a different template engine. All rendering calls go through the trait, not the Handlebars crate directly.

Cross-reference: `context/external/rocket.md` for the server-side rendering integration.

### View model serialization

Template contexts are serialized using Serde before being passed to Handlebars. View model structs should derive `Serialize` and present only the data needed for rendering. Avoid passing entire domain entities to templates.

Cross-reference: `context/external/serde.md` for serialization conventions.

## Performance and Safety Considerations

### Template compilation cost

Handlebars template compilation involves parsing and AST generation. For MCB's template set, this is a one-time startup cost. If templates are loaded from user-configurable paths, validate the file count and size to prevent startup delays.

### Strict mode

Enable Handlebars strict mode to catch missing variables at render time instead of silently rendering empty strings. MCB should configure `handlebars.set_strict_mode(true)` to fail fast on template/data mismatches.

### HTML escaping

Handlebars escapes HTML by default in `{{ }}` expressions. Use `{{{ }}}` (triple braces) only when intentionally rendering raw HTML. MCB's admin views should default to escaped output to prevent XSS.

## Testing and Verification Guidance

### Template render tests

Test that templates render correctly with known view model data. Create fixture view models and assert on the rendered output containing expected strings.

### Helper unit tests

Each custom helper should have a standalone test that verifies its output for various inputs. This is more reliable than testing helpers only through full template renders.

### Missing field detection

With strict mode enabled, test that rendering with incomplete data produces a clear error rather than silent empty strings. This catches data contract mismatches early.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Template not registered at startup | Render-time panic/error | Register all templates in initialization; test coverage |
| Missing field in non-strict mode | Silent empty rendering | Enable strict mode; test with incomplete data |
| Helper with side effects | Unpredictable rendering behavior | Keep helpers pure; code review enforcement |
| XSS from raw HTML output | Security vulnerability | Default to escaped output; review triple-brace usage |
| Large template set slowing startup | Boot time regression | Monitor startup duration; consider template caching |

Cross-reference: `context/external/tracing.md` for instrumenting template render latency.

## Migration and Version Notes

- MCB uses handlebars 6.x (current major).
- Handlebars 6.x introduced breaking changes from 5.x in helper signatures and error types.
- ADR-028 (`docs/adr/028-advanced-code-browser-v020.md`) established the server-side rendering stack.
- ADR-007 (`docs/adr/007-integrated-web-administration-interface.md`) requires the admin UI to use server-rendered templates.
- If admin UI moves to a SPA architecture, Handlebars would be replaced by a client-side framework. This is not currently planned.

## Verification Checklist

- [ ] Templates registered once at startup, not per-request
- [ ] Strict mode enabled on Handlebars instance
- [ ] Custom helpers centralized in `admin/web/helpers.rs`
- [ ] Helpers are pure functions (no I/O or service calls)
- [ ] View models derive `Serialize` and expose only needed fields
- [ ] HTML escaping used by default (triple braces only with explicit justification)
- [ ] Template render tests with known fixture data
- [ ] Missing field tests with strict mode

## Common Pitfalls

- Registering templates lazily per request instead of once at startup.
- Silent missing-field rendering when strict/validation controls are absent.
- Spreading helper logic across files, making templates hard to maintain.
- Using triple braces (`{{{ }}}`) without explicit XSS review.
- Passing full domain entities to template context instead of purpose-built view models.

## References

- https://docs.rs/handlebars/latest/handlebars/
- https://github.com/sunng87/handlebars-rust
- https://handlebarsjs.com/guide/
- `docs/adr/028-advanced-code-browser-v020.md`
- `docs/adr/007-integrated-web-administration-interface.md`
- `docs/adr/026-routing-refactor-rocket-poem.md`
- `context/external/rocket.md`
- `context/external/serde.md`
- `context/external/tracing.md`
