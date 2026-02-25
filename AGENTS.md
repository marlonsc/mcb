# AGENTS.md — AI Agent Configuration Index

This repository uses multiple AI coding agents. All share a **single source of truth**
for project rules, architecture, and conventions: [`CLAUDE.md`](CLAUDE.md).

## Agent Configuration Map

| Agent | Config File | Role |
|-------|-------------|------|
| **Claude Code** | [`CLAUDE.md`](CLAUDE.md) | Canonical — all rules defined here |
| **Gemini Code Assist** | [`.gemini/styleguide.md`](.gemini/styleguide.md) | PR review priorities, delegates to CLAUDE.md |
| **GitHub Copilot** | [`.github/copilot-instructions.md`](.github/copilot-instructions.md) | Pointer to CLAUDE.md |
| **Cursor AI** | [`.cursor/rules/mcb.mdc`](.cursor/rules/mcb.mdc) | Pointer to CLAUDE.md |
| **OpenAI Codex** | [`codex.md`](codex.md) | Pointer to CLAUDE.md |
| **Cline** | [`.clinerules`](.clinerules) | Pointer to CLAUDE.md |
| **Windsurf** | [`.windsurfrules`](.windsurfrules) | Pointer to CLAUDE.md |
| **Continue.dev** | [`.continue/rules/mcb.md`](.continue/rules/mcb.md) | Pointer to CLAUDE.md |
| **Aider** | [`CONVENTIONS.md`](CONVENTIONS.md) + [`.aider.conf.yml`](.aider.conf.yml) | Auto-loads CONVENTIONS.md + CLAUDE.md |

## How It Works

```
AGENTS.md (this file — index)
    |
    v
CLAUDE.md (single source of truth — all rules, patterns, commands)
    |
    +-- .gemini/styleguide.md            (Gemini: PR review priorities)
    +-- .github/copilot-instructions.md  (Copilot: pointer)
    +-- .cursor/rules/mcb.mdc            (Cursor: pointer)
    +-- codex.md                         (Codex: pointer)
    +-- .clinerules                      (Cline: pointer)
    +-- .windsurfrules                   (Windsurf: pointer)
    +-- .continue/rules/mcb.md           (Continue: pointer)
    +-- CONVENTIONS.md                   (Aider: pointer)
```

## Maintenance Rules

1. **All rule changes go to `CLAUDE.md` first** — agent-specific files only add tool-specific behavior.
2. **Never duplicate rules** across agent configs — reference `CLAUDE.md` sections instead.
3. **Agent-specific files must stay under 50 lines** — they are pointers, not copies.
4. When updating architecture, conventions, or quality gates, update `CLAUDE.md` only.

## Documentation References (git-tracked)

- [Architecture](docs/architecture/ARCHITECTURE.md) — layers, crate map, dependency flow
- [Clean Architecture](docs/architecture/CLEAN_ARCHITECTURE.md) — layer rules, extension guide
- [Architecture Boundaries](docs/architecture/ARCHITECTURE_BOUNDARIES.md) — dependency rules, violations
- [ADRs](docs/adr/) — 52 Architecture Decision Records
- [Contributing](docs/developer/CONTRIBUTING.md) — dev setup, coding standards, PR process
- [MCP Tools](docs/MCP_TOOLS.md) — full tool API schemas
- [Configuration](docs/CONFIGURATION.md) — all environment variables and config options
- [Roadmap](docs/developer/ROADMAP.md) — version plans and feature timeline
