<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# MCB Project Skills Index

Project-specific ECC skills for the MCB Rust workspace. Load the relevant skill
before editing code.

The skill files live under `.claude/skills/<name>/SKILL.md`. That tree is
workstation-local: `.gitignore` excludes `.claude/*`, so the skills are loaded by
the agent runtime rather than versioned here, and this index names them instead
of linking to repository paths that do not exist.

## Central index

- **`mcb-patterns`** — Central quick reference and skill router.

## Domain skills

| Skill | Use when |
|-------|----------|
| `mcb-make-verbs` | Running build, test, lint, validate, ship, or bootstrap commands |
| `mcb-architecture-layers` | Adding modules, reviewing crate boundaries, or moving responsibilities |
| `mcb-error-handling` | Writing fallible code paths, choosing `?` vs `.context()` vs `Error::*` |
| `mcb-import-rules` | Adding imports, resolving cycles, reviewing visibility |
| `mcb-testing-patterns` | Writing or reviewing tests, fixtures, and test helpers |
| `mcb-quality-gates` | Validating changes before commit or debugging gate failures |

## Coordination skill

- **`orchestrate`** — Multi-step coordinator↔executor loop with beads.

## Usage

In Claude Code, invoke with `/skill:<name>` or the `Skill` tool. In other ECC environments, use the platform-specific skill activation command.

## Maintenance

When MCB patterns change, update the corresponding skill. The skill front matter should always cite the canonical source files (`Cargo.toml`, `Makefile`, `AGENTS.md`, etc.).
