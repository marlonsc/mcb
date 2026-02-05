# Phase 8: Quick Reference Card

## 🎯 Top 5 Libraries/Patterns

| # | Name | Version | Purpose | Link |
|---|------|---------|---------|------|
| 1 | **tree-sitter** | 0.26.3+ | Core AST parser (incremental, zero-copy) | <https://docs.rs/tree-sitter> |
| 2 | **tree-sitter-highlight** | 0.26.3+ | Official syntax highlighting (GitHub-proven) | <https://docs.rs/tree-sitter-highlight> |
| 3 | **tree-sitter-traversal** | 0.1.2+ | Efficient AST walking with TreeCursor | <https://docs.rs/tree-sitter-traversal> |
| 4 | **tree-sitter-visitor** | - | Procedural macros for visitor patterns | tibordp/tree-sitter-visitor |
| 5 | **bat/zat** | 0.26.1+ | Production ANSI highlighting reference | sharkdp/bat, neurocyte/zat |

---

## 📦 Cargo.toml Dependencies

```toml
[dependencies]
tree-sitter = "0.26"
tree-sitter-highlight = "0.26"
tree-sitter-traversal = "0.1"
tree-sitter-rust = "0.24"
tree-sitter-javascript = "0.24"
tree-sitter-python = "0.23"
tree-sitter-go = "0.23"

[dev-dependencies]
criterion = "0.7"
tempfile = "3.8"
```

---

## 🏗️ Implementation Architecture

```
Code Input → Parser (tree-sitter) → Query Matching (highlights.scm)
                                            ↓
                          ┌─────────────────┴──────────────┐
                          ↓                                 ↓
                    ANSI Output              HTML Output (MCP Apps)
                  (Terminal Display)      (Browser Visualization)
```

---

## 🔑 Key Concepts

### Query System

-   **Pattern Language**: S-expressions in `.scm` files
-   **Example**: `"fn" @keyword` or `(function_declaration name: (identifier) @function.def)`
-   **Power**: Context-aware, not just tokens; incremental; zero-copy

### Output Formats

-   **ANSI 256-color**: `\x1b[38;5;196m` for color, `\x1b[0m` for reset
-   **ANSI Truecolor**: `\x1b[38;2;255;0;0m` (24-bit RGB)
-   **HTML**: `<span class="highlight-keyword">fn</span>`

### Performance Targets

-   **Parsing**: O(1) incremental for edits
-   **Highlighting**: O(n) but optimized
-   **Benchmark**: <50ms for 10KB file

---

## 🚀 Phase 8 Implementation Timeline

### Phase 8a: Core Infrastructure (2-3 days)

```rust
let mut parser = Parser::new();
parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
let tree = parser.parse(source_code, None)?;
```

### Phase 8b: Query System (2-3 days)

Load `.scm` files from language grammars, implement pattern matching

### Phase 8c: Output Formatters (2-3 days)

ANSI code generator + HTML generator + theme system

### Phase 8d: MCP Integration (2 days)

Tool: `highlight_code()` | Resource: `code://`

---

## 📊 Confidence Matrix

| Component | Confidence | Risk | Priority |
|-----------|------------|------|----------|
| Core parser | 🟢 High | Low | P0 |
| Highlighting lib | 🟢 High | Low | P0 |
| ANSI output | 🟢 High | Low | P1 |
| HTML output | 🟡 Medium | Medium | P1 |
| MCP integration | 🟡 Medium | Medium | P2 |
| Query writing | 🟡 Medium | High | P2 |
| Performance @ scale | 🔴 Low | Medium | P3 |

---

## 🔗 Key Resources

**Official Docs**

-   tree-sitter.GitHub.io/tree-sitter (full guide)
-   modelcontextprotocol.io (MCP spec + Apps)
-   docs.rs/tree-sitter-highlight (API ref)

**Code Examples**

-   bat: <https://github.com/sharkdp/bat>
-   zat: <https://github.com/neurocyte/zat>
-   siraben.dev/2022/03/22/tree-sitter-linter.html

**Learning Resources**

-   brianmoniaga.com/Blog/posts/syntaxhighlighting/ (tutorial)
-   dev.to/shrsv: Unraveling Tree-Sitter Queries
-   tree-sitter repo tests (real-world .scm examples)

---

## ✅ Immediate Next Steps

1.  **Add dependencies** to Cargo.toml (15 min)
2.  **Write basic test** with tree-sitter Parser (30 min)
3.  **Load a query file** from grammar repo (1 hour)
4.  **Generate ANSI output** from highlights (2-3 hours)
5.  **Design MCP tool schema** (30 min)
6.  **Benchmark parsing performance** (1 hour)

---

## 🎓 Learning Path

1.  **Start**: tree-sitter core API (docs.rs)
2.  **Understand**: Query system (tree-sitter.GitHub.io guide)
3.  **Implement**: Basic ANSI highlighter
4.  **Reference**: Study bat/zat codebases for patterns
5.  **Optimize**: Benchmark and profile
6.  **Integrate**: MCP server context

---

## 💡 Pro Tips

-   **Language Detection**: Match file extension → grammar name (build lookup table)
-   **Caching**: Store parsed trees + query results (LRU cache) for repeated operations
-   **Theme Support**: Map highlight names → ANSI/CSS (VS Code theme format)
-   **Incremental**: Store Tree reference, use `.edit()` for partial updates
-   **Testing**: Use tree-sitter test fixtures (caret/arrow syntax)

---

## 🚨 Known Challenges

| Challenge | Solution |
|-----------|----------|
| **Learning .scm queries** | Start simple, reference existing grammars, study tree-sitter docs |
| **HTML sanitization** | Use `ammonia` crate for safe output |
| **Theme mapping** | Build Theme struct, map highlight names consistently |
| **Performance at 1MB+** | Benchmark early, consider chunking, profile with Criterion |
| **Browser context** | Design for MCP Apps (2026 feature), plan ahead |

---

Generated: 2025 | Next Review: Phase 8 kickoff
