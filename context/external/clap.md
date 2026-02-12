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

## Common Pitfalls

- Overloading a single command with too many unrelated flags instead of subcommands.
- Blending parse and execution logic so deeply that command behavior is hard to test.
- Inconsistent defaults between CLI args and config-driven behavior.

## References

- https://docs.rs/clap/latest/clap/
- https://docs.rs/clap/latest/clap/_derive/_tutorial/
- https://github.com/clap-rs/clap
- `docs/adr/021-dependency-management.md`
