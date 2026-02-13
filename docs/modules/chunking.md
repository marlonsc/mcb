<!-- markdownlint-disable MD013 MD024 MD025 MD060 -->
# chunking Module

**Source**: `crates/mcb-providers/src/language/`
**Crate**: `mcb-providers`

## Overview

Chunking uses tree-sitter processors for**13 languages**via**12 parser implementations**. JavaScript processor handles both JavaScript and TypeScript using mode-specific behavior.

## Language Processors

| Language | Processor File | Parser |
| ---------- | ---------------- | -------- |
| Rust | `rust.rs` | tree-sitter-rust |
| Python | `python.rs` | tree-sitter-python |
| JavaScript | `javascript.rs` | tree-sitter-javascript |
| TypeScript | `javascript.rs` (TS mode) | tree-sitter-javascript (TS handling in processor) |
| Go | `go.rs` | tree-sitter-go |
| Java | `java.rs` | tree-sitter-java |
| C | `c.rs` | tree-sitter-c |
| C++ | `cpp.rs` | tree-sitter-cpp |
| C# | `csharp.rs` | tree-sitter-c-sharp |
| Ruby | `ruby.rs` | tree-sitter-ruby |
| PHP | `php.rs` | tree-sitter-php |
| Swift | `swift.rs` | tree-sitter-swift |
| Kotlin | `kotlin.rs` | tree-sitter-kotlin |

## File Structure

```text
crates/mcb-providers/src/language/
├── rust.rs
├── python.rs
├── javascript.rs
├── go.rs
├── java.rs
├── c.rs
├── cpp.rs
├── csharp.rs
├── ruby.rs
├── php.rs
├── swift.rs
├── kotlin.rs
├── detection.rs
├── engine.rs
└── mod.rs
```

## Cross-References

- **Providers**: [providers.md](./providers.md)
- **Application services**: [services.md](./services.md)
- **Domain types**: [domain.md](./domain.md)

---

### Updated 2026-02-12 - Reflects current language processing architecture (v0.2.1)
