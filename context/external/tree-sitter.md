# tree-sitter

Last updated: 2026-02-12

## Usage in MCB

- Used for AST parsing, static analysis, and syntax highlighting.
- Internal examples: `crates/mcb-infrastructure/src/services/highlight_service.rs`, `crates/mcb-ast-utils/src/walker.rs`.

## Key Capabilities

- Incremental parsers per language grammar.
- Query-based syntax highlighting.
- Structured node traversal for code analysis and metrics.

## Best Practices

- Reuse parser/highlighter instances where possible.
- Validate language support before executing queries.
- Keep query configuration isolated per language.

## Common Pitfalls

- Mixing grammar versions can create subtle parse mismatches.
- Ignoring encoding and offset normalization can produce incorrect ranges.

## Official References

- https://docs.rs/tree-sitter
- https://tree-sitter.github.io/tree-sitter/

## GitHub References

- https://github.com/openai/codex/blob/main/codex-rs/core/src/bash.rs
- https://github.com/github/codeql/blob/main/python/extractor/tsg-python/src/main.rs
