# clap

Last updated: 2026-02-12

## Usage in MCB

- Command-line argument parsing for binary entry points.
- Strongly typed flags, options, and subcommands.

## Key Capabilities in Use

- Derive API: `Parser`, `Subcommand`, `Args`.
- Automatic help/usage generation.
- Enum and typed argument validation.

## Best Practices

1. Keep CLI parsing separate from business logic orchestration.
2. Use subcommands to group operational domains.
3. Apply CLI overrides after loading config files.
4. Keep help text synced via Rust doc comments.

## Common Pitfalls

- Large flat flag sets reduce usability.
- Parsing + execution in one module hurts maintainability.

## Official References

- https://docs.rs/clap
- https://docs.rs/clap/latest/clap/_cookbook/index.html

## GitHub References

- https://github.com/sharkdp/fd/blob/master/src/cli.rs
- https://github.com/BurntSushi/ripgrep/blob/master/crates/core/flags/defs.rs
