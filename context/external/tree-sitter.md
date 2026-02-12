# tree-sitter

Last updated: 2026-02-12

## Executive Summary

Tree-sitter is the parsing backbone for MCB's language-aware chunking, AST traversal, and syntax highlighting pipelines.

## Context7 + External Research

- Context7 ID: `/websites/rs_tree-sitter`
- Rust crate docs: https://docs.rs/tree-sitter/latest/tree_sitter/
- Official guide: https://tree-sitter.github.io/tree-sitter/
- Upstream: https://github.com/tree-sitter/tree-sitter

## Actual MCB Usage (Current Source of Truth)

### 1) Parser lifecycle in language engine

- `crates/mcb-providers/src/language/engine.rs:178`
- `crates/mcb-providers/src/language/engine.rs:180`
- `crates/mcb-providers/src/language/engine.rs:200`

Pattern: parser and language selection drive chunk extraction for multi-language source inputs.

### 2) Language config and processor abstraction

- `crates/mcb-providers/src/language/common/config.rs:70`
- `crates/mcb-providers/src/language/common/config.rs:106`
- `crates/mcb-providers/src/language/common/processor.rs:35`

Pattern: processors receive typed `tree_sitter::Tree` and keep language-specific logic isolated.

### 3) Node/cursor traversal and chunk creation

- `crates/mcb-providers/src/language/common/traverser.rs:60`
- `crates/mcb-providers/src/language/common/traverser.rs:131`
- `crates/mcb-providers/src/language/common/traverser.rs:199`

Pattern: cursor-driven AST traversal feeds chunk boundaries and metadata extraction.

### 4) Highlighting and AST utilities

- `crates/mcb-infrastructure/src/services/highlight_service.rs:12`
- `crates/mcb-validate/src/ast/decoder.rs:7`
- `crates/mcb-ast-utils/src/walker.rs:5`

Pattern: highlighting and validator AST decoding are both tied to tree-sitter structures.

## ADR Alignment (Critical)

- ADR-028 (`docs/adr/028-advanced-code-browser-v020.md`): tree-sitter-based syntax/highlighting architecture.
- ADR-042 (`docs/adr/042-knowledge-graph-code-context.md`): tree-sitter extraction strategy for semantic relationships.
- ADR-015 (`docs/adr/015-workspace-shared-libraries.md`): tree-sitter analysis planned/shared in workspace design.

## GitHub Evidence (Upstream + In-Repo)

- Upstream tree-sitter: https://github.com/tree-sitter/tree-sitter
- Zed parser/query example: https://github.com/zed-industries/zed/blob/main/crates/settings_json/src/settings_json.rs
- Postgres language server example: https://github.com/supabase-community/postgres-language-server/blob/main/crates/pgls_treesitter/src/queries/relations.rs
- In-repo anchor: `crates/mcb-providers/src/language/rust.rs:60`

## Common Pitfalls

- Recreating parser/query structures in tight loops instead of reuse.
- Letting grammar/version skew drift across language crates.
- Ignoring `has_error()` and treating every parse as semantically valid.

## References

- https://docs.rs/tree-sitter/latest/tree_sitter/
- https://tree-sitter.github.io/tree-sitter/
- https://github.com/tree-sitter/tree-sitter
- `docs/adr/028-advanced-code-browser-v020.md`
- `docs/adr/042-knowledge-graph-code-context.md`
