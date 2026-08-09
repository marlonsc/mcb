# Context Directory (AI Working Memory)

Last updated: 2026-02-12 (America/Sao_Paulo)

**This directory is NOT git-tracked.** It is local AI working memory for agent bootstrapping.

## Model

- `docs/` = Analytical, detailed, git-tracked project documentation (single source of truth)
- `context/` = Concise AI quick-reference (<200 lines per file), references `docs/` for depth

## Structure

```text
context/
├── core/                        # Agent workflow patterns
│   ├── agent-patterns.md
│   ├── tool-usage.md
│   ├── error-handling.md
│   └── sync-log.md
├── development/                 # Dev practices (concise refs)
│   ├── git-workflow.md          → docs/developer/CONTRIBUTING.md
│   ├── testing-patterns.md      → docs/developer/CONTRIBUTING.md
│   └── code-review.md
├── project-intelligence/        # Project-specific state
│   ├── technical-patterns.md    → docs/architecture/PATTERNS.md
│   ├── domain-concepts.md       → docs/modules/domain.md
│   ├── integrations.md          → docs/modules/providers.md
│   ├── conventions.md           → docs/developer/CONTRIBUTING.md
│   ├── project-state.md         → docs/developer/ROADMAP.md
│   ├── clean-architecture.md    → docs/architecture/CLEAN_ARCHITECTURE.md
│   ├── architecture-boundaries.md → docs/architecture/ARCHITECTURE_BOUNDARIES.md
│   ├── modernization-audit.md
│   ├── v0.2.1-modernization-plan.md
│   ├── v0.2.1-history-and-pending.md
│   └── v0.2.1-history-and-pending-closure.md
└── external/                    # Library quick-refs
    ├── README.md
    └── [library].md             # Per-library patterns & pitfalls
```

## MVI Rule

- Each file: <200 lines, scannable in <30 seconds.
- Point to `docs/` for detailed explanations — never duplicate.
- Include `Last updated` and source paths.
