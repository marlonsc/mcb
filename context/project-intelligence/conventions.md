# Project Conventions

Last updated: 2026-02-11
Source baseline: `docs/context/conventions.md`

Coding style:

- Rust naming: snake_case for functions/modules, PascalCase for types/traits.
- Crate naming: `mcb-*` for package names and `mcb_*` for library names.
- Import order follows rustfmt defaults (std, external, workspace, local).

Quality and safety:

- Workspace lints deny unsafe code, dead code, unused vars/imports.
- Error handling uses domain `Error` factories and `Result<T>` propagation.
- `unwrap()` and `expect()` are test-only.

Workflow:

- Make-first execution: `make fmt`, `make lint`, `make test`, `make validate`.
- Conventional commits: `type(scope): description`.
- Tests primarily in crate `tests/` trees.

Documentation:

- Public API docs expected (`//!`, `///` patterns).
- Keep context files concise and actionable.

Related:

- `context/development/git-workflow.md`
- `context/development/testing-patterns.md`
