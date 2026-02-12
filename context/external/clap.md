# clap

Last updated: 2026-02-12

## Executive Summary

Clap defines MCB's CLI contract in the `mcb` binary with derive-based parsing, typed subcommands, and argument validation.

## Context7 + External Research

- Context7 ID: `/websites/rs_clap`
- Main docs: https://docs.rs/clap/latest/clap/
- Derive tutorial: https://docs.rs/clap/latest/clap/_derive/_tutorial/
- Upstream: https://github.com/clap-rs/clap

## Actual MCB Usage (Current Source of Truth)

### 1) Root parser and command metadata

- `crates/mcb/src/main.rs:26`
- `crates/mcb/src/main.rs:30`
- `crates/mcb/src/main.rs:36`
- `crates/mcb/src/main.rs:54`

Pattern: top-level CLI uses `Parser` + `Subcommand`, then dispatches to command handlers.

### 2) Serve subcommand args

- `crates/mcb/src/cli/serve.rs:5`
- `crates/mcb/src/cli/serve.rs:8`
- `crates/mcb/src/cli/serve.rs:11`
- `crates/mcb/src/cli/serve.rs:19`

Pattern: concise flags for config path and daemon/server behavior.

### 3) Validate subcommand args and validation flags

- `crates/mcb/src/cli/validate.rs:5`
- `crates/mcb/src/cli/validate.rs:11`
- `crates/mcb/src/cli/validate.rs:27`
- `crates/mcb/src/cli/validate.rs:35`

Pattern: path, strictness, rules path, validator list, severity, and format are fully typed.

### 4) Fixture usage for CLI patterns in tests

- `crates/mcb-validate/tests/fixtures/rustlings/src/main.rs:3`
- `crates/mcb-validate/tests/fixtures/rustlings/src/dev.rs:2`

Pattern: fixtures preserve Clap-derived shapes used by validator checks.

## ADR Alignment (Critical)

- ADR-021 (`docs/adr/021-dependency-management.md`): Clap is workspace-managed with derive feature.
- ADR-001 (`docs/adr/001-modular-crates-architecture.md`): binary facade owns command routing concerns.

## GitHub Evidence (Upstream + In-Repo)

- Upstream Clap: https://github.com/clap-rs/clap
- Production CLI example (fd): https://github.com/sharkdp/fd/blob/master/src/cli.rs
- Production CLI example (ripgrep): https://github.com/BurntSushi/ripgrep/blob/master/crates/core/flags/defs.rs
- In-repo anchor: `crates/mcb/src/main.rs:44`
- In-repo anchor: `crates/mcb/src/cli/validate.rs:58`

## Best Practices in MCB

### Separation of parse and execution

MCB separates CLI parsing from command execution. The main function (`crates/mcb/src/main.rs:54`) parses arguments into typed structs, then dispatches to command handler functions. This makes command logic independently testable without parsing concerns.

### Derive-based API

MCB uses Clap's derive macro (`#[derive(Parser)]`, `#[derive(Subcommand)]`) exclusively. Manual Clap builder API is not used. This ensures CLI definitions stay declarative and type-safe.

Key derive locations:
- Root CLI struct: `crates/mcb/src/main.rs:26`
- Serve subcommand: `crates/mcb/src/cli/serve.rs:5`
- Validate subcommand: `crates/mcb/src/cli/validate.rs:5`

### Config/CLI precedence

CLI arguments take precedence over config file values. MCB loads configuration through `figment` (`context/external/figment.md`) and merges CLI overrides on top. This precedence chain must be maintained for any new arguments.

Cross-reference: `context/external/figment.md` for config layering strategy.

### Subcommand extension pattern

Adding a new CLI command requires:
1. Define a new `Args` struct with `#[derive(clap::Args)]` in `crates/mcb/src/cli/`
2. Add a variant to the `Subcommand` enum in `crates/mcb/src/main.rs`
3. Add a match arm in the dispatch function
4. Update shell completion generation if applicable

## Performance and Safety Considerations

### Argument validation at parse time

Use Clap's built-in validators (`value_parser!`, `#[arg(value_parser)]`) to reject invalid input before command execution begins. MCB's validate subcommand (`crates/mcb/src/cli/validate.rs:27`) demonstrates typed validation for paths and enum values.

### Default value consistency

Defaults declared in Clap derive attributes must match the defaults used in config file loading. Inconsistency between CLI defaults and config defaults causes confusing behavior.

## Testing and Verification Guidance

### CLI parsing tests

Test that CLI arg structs parse expected command lines correctly. Use `Cli::try_parse_from(&["mcb", "serve", "--config", "path"])` in unit tests.

### Help text verification

Run `mcb --help` and `mcb <subcommand> --help` to verify help strings are accurate and not stale after refactoring.

### Fixture CLI patterns

MCB uses Clap-derived patterns in test fixtures (`crates/mcb-validate/tests/fixtures/rustlings/src/main.rs:3`) to validate that the quality linter correctly identifies CLI patterns.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Inconsistent defaults between CLI and config | Confusing user experience | Document precedence chain; test both paths |
| Too many flags on a single command | Poor UX, hard to maintain | Use subcommands for distinct workflows |
| Missing required arg not caught at parse | Runtime failure | Use Clap's required attribute; test parse failures |
| Stale help text after refactor | User confusion | Automated help text snapshot tests |

## Migration and Version Notes

- MCB uses Clap 4.5 with derive feature.
- Clap 4.x is the current major; no breaking migration expected until Clap 5.
- ADR-021 (`docs/adr/021-dependency-management.md`) manages Clap as a workspace dependency.
- ADR-001 (`docs/adr/001-modular-crates-architecture.md`) constrains CLI concerns to the binary facade crate.

## Verification Checklist

- [ ] New command uses derive macro, not builder API
- [ ] Parse and execution logic separated
- [ ] Defaults consistent between CLI attributes and config file
- [ ] Help text updated for new/changed arguments
- [ ] Subcommand registered in main dispatch
- [ ] CLI parsing unit test covers new command

## Common Pitfalls

- Overloading a single command with too many unrelated flags instead of subcommands.
- Blending parse and execution logic so deeply that command behavior is hard to test.
- Inconsistent defaults between CLI args and config-driven behavior.
- Forgetting to register a new subcommand variant in the dispatch match.
- Using string-based argument names instead of typed derive attributes.

## References

- https://docs.rs/clap/latest/clap/
- https://docs.rs/clap/latest/clap/_derive/_tutorial/
- https://github.com/clap-rs/clap
- `docs/adr/021-dependency-management.md`
- `docs/adr/001-modular-crates-architecture.md`
- `context/external/figment.md`
