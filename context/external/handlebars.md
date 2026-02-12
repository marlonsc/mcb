# handlebars-rust

Last updated: 2026-02-12

## Usage in MCB

- Server-side rendering for admin/web templates.
- Internal examples: `crates/mcb-server/src/templates/engine/handlebars_engine.rs`, `crates/mcb-server/src/admin/web/helpers.rs`.

## Key Capabilities

- Template, partial, and helper registry.
- Strict mode to catch missing fields.
- Rendering with `serde`-serializable context.

## Best Practices

- Validate template registration at startup.
- Enable strict mode in critical admin/web pages.
- Centralize helper registration to avoid duplication.

## Common Pitfalls

- Silent rendering failures if templates are not validated on boot.
- Overly dynamic context structures reduce maintainability.

## Official References

- https://docs.rs/handlebars
- https://github.com/sunng87/handlebars-rust

## GitHub References

- https://github.com/sunng87/handlebars-rust/blob/master/src/lib.rs
- https://github.com/sunng87/handlebars-rust/blob/master/examples/quick.rs
