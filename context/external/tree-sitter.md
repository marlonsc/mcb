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

## Best Practices in MCB

### Parser lifecycle management

MCB creates parsers once per language and reuses them across files. The language engine (`crates/mcb-providers/src/language/engine.rs:178`) selects the grammar, sets the language on the parser, and then feeds source text. Parser instances are not shared across threads — each parse operation owns its parser.

### Cursor-driven traversal

MCB uses `TreeCursor` for efficient AST traversal rather than recursive node walking. The traverser (`crates/mcb-providers/src/language/common/traverser.rs:60`) drives chunk boundary detection by walking cursor positions and emitting chunk metadata.

This approach is more memory-efficient than collecting all nodes into a vector.

### Language-specific processor isolation

Each supported language has its own processor module (e.g., `crates/mcb-providers/src/language/rust.rs`). Processors receive a `tree_sitter::Tree` and extract language-specific semantic information. This keeps cross-language concerns separated.

Cross-reference: `context/external/tokio.md` for spawn_blocking when parsing large files.

### Error handling on parse results

Always check `tree.root_node().has_error()` before treating parse results as semantically complete. MCB's traverser should gracefully degrade on partial parses rather than emitting corrupt chunks.

## Performance and Safety Considerations

### Blocking nature of parsing

Tree-sitter parsing is CPU-bound and synchronous. MCB offloads parsing to `spawn_blocking` in `crates/mcb-providers/src/language/engine.rs:130` to avoid blocking Tokio worker threads.

For batch indexing, consider limiting concurrent parse operations to avoid saturating the blocking thread pool.

### Memory pressure from large files

Very large source files (10K+ lines) can produce substantial AST trees. MCB mitigates this by chunking at function/class boundaries rather than holding the entire AST in memory.

### Query compilation cost

Tree-sitter queries (`.scm` files) should be compiled once and cached. Recompiling queries per-file is expensive. MCB's highlighting service should pre-compile queries at initialization.

### Grammar version consistency

All language grammars in the workspace must be compatible with the tree-sitter runtime version. A mismatch causes panics at parse time. Pin grammar crate versions alongside the tree-sitter version in `Cargo.toml`.

## Testing and Verification Guidance

### Unit testing parsers

Test that each supported language parser:
1. Successfully parses a known-good source file
2. Reports `has_error()` on intentionally malformed input
3. Produces expected node kinds at known positions

### Testing chunk boundaries

Chunk extraction tests should verify that function/class/module boundaries are correctly identified. Use small fixture files with known structure.

### Fixture location

MCB stores test fixtures in `crates/mcb-validate/tests/fixtures/` and `crates/mcb-providers/tests/`. Language-specific parse fixtures should be co-located with the processor module.

## Operational Risk and Monitoring

| Risk | Impact | Mitigation |
|---|---|---|
| Grammar/runtime version mismatch | Parse-time panic | Pin grammar versions alongside tree-sitter in Cargo.toml |
| Blocking parse on async thread | Worker starvation | Enforce spawn_blocking for all parse operations |
| Large file parse OOM | Memory exhaustion | Chunk at boundaries; skip files above size threshold |
| Corrupt parse (has_error) silently used | Bad chunk metadata | Check has_error() and degrade gracefully |
| Missing language grammar | Runtime failure for unsupported language | Enumerate supported languages; return clear error for unsupported |

Cross-reference: `context/external/tracing.md` for instrumenting parse latency.

## Migration and Version Notes

- MCB uses the tree-sitter Rust crate (currently tracking latest stable).
- Tree-sitter 0.22+ changed the query API. Verify query compatibility on upgrades.
- ADR-028 (`docs/adr/028-advanced-code-browser-v020.md`) mandates tree-sitter for syntax highlighting.
- ADR-042 (`docs/adr/042-knowledge-graph-code-context.md`) plans tree-sitter-based semantic extraction for knowledge graph features.
- Grammar crates (tree-sitter-rust, tree-sitter-python, etc.) must be updated together with the core tree-sitter crate.

## Verification Checklist

- [ ] Parser created once per language, not per file
- [ ] All parse operations run inside `spawn_blocking`
- [ ] `has_error()` checked on parse results before consuming AST
- [ ] Grammar versions pinned and consistent with tree-sitter runtime
- [ ] Chunk boundary tests cover each supported language
- [ ] Large file threshold configured to skip or limit parse scope
- [ ] Highlighting queries pre-compiled at initialization

## Common Pitfalls

- Recreating parser/query structures in tight loops instead of reuse.
- Letting grammar/version skew drift across language crates.
- Ignoring `has_error()` and treating every parse as semantically valid.
- Running CPU-bound parsing on async threads without spawn_blocking.
- Compiling tree-sitter queries per-file instead of caching them.

## References

- https://docs.rs/tree-sitter/latest/tree_sitter/
- https://tree-sitter.github.io/tree-sitter/
- https://github.com/tree-sitter/tree-sitter
- `docs/adr/028-advanced-code-browser-v020.md`
- `docs/adr/042-knowledge-graph-code-context.md`
- `docs/adr/015-workspace-shared-libraries.md`
- `context/external/tokio.md`
- `context/external/tracing.md`
