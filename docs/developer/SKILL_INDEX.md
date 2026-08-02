<!-- markdownlint-disable MD013 MD024 MD025 MD003 MD022 MD031 MD032 MD036 MD041 MD060 -->

# MCB Project Skills Index

Project-specific ECC skills for the MCB Rust workspace. Load the relevant skill before editing code.

## Central index

- **[mcb-patterns](../../.agents/skills/mcb-patterns/SKILL.md)** — Central quick reference and skill router.

## Domain skills

| Skill | Use when |
|-------|----------|
| [mcb-make-verbs](../../.agents/skills/mcb-make-verbs/SKILL.md) | Running build, test, lint, validate, ship, or bootstrap commands |
| [mcb-architecture-layers](../../.agents/skills/mcb-architecture-layers/SKILL.md) | Adding modules, reviewing crate boundaries, or moving responsibilities |
| [mcb-error-handling](../../.agents/skills/mcb-error-handling/SKILL.md) | Writing fallible code paths, choosing `?` vs `.context()` vs `Error::*` |
| [mcb-import-rules](../../.agents/skills/mcb-import-rules/SKILL.md) | Adding imports, resolving cycles, reviewing visibility |
| [mcb-testing-patterns](../../.agents/skills/mcb-testing-patterns/SKILL.md) | Writing or reviewing tests, fixtures, and test helpers |
| [mcb-quality-gates](../../.agents/skills/mcb-quality-gates/SKILL.md) | Validating changes before commit or debugging gate failures |

## Coordination skill

- **[orchestrate](../../.agents/skills/orchestrate/SKILL.md)** — Multi-step coordinator↔executor loop with beads.

## Usage

In Claude Code, invoke with `/skill:<name>` or the `Skill` tool. In other ECC environments, use the platform-specific skill activation command.

## Maintenance

When MCB patterns change, update the corresponding skill. The skill front matter should always cite the canonical source files (`Cargo.toml`, `Makefile`, `AGENTS.md`, etc.).
