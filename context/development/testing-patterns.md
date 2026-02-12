# Testing Patterns

Last updated: 2026-02-11 (America/Sao_Paulo)

## Purpose

Keep high-value testing and validation expectations visible for
contributors and agents.

## Core Checks

- `make test` for full test suite.
- `make lint` for style and static quality.
- `make validate` for architecture rules.
- `make quality` for end-to-end quality pipeline.

## Documentation Checks

- `make docs-lint`
- `make docs-validate QUICK=1`

## Practical Rules

- Run the narrowest useful test loop while iterating.
- Run broader checks before closing work.
- Treat architecture validation as a required gate, not optional.

## Sources

- `README.md`
- `docs/developer/CONTRIBUTING.md`
- `docs/operations/CI_RELEASE.md`

## Update Notes

- 2026-02-11: Added condensed validation matrix for fast execution decisions.
