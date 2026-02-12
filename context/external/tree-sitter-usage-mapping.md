# Tree-Sitter Library - Internal Usage Mapping

**Library**: `tree-sitter` + 14 language parsers (v0.20.x)  
**Status**: IMPLEMENTED (v0.1.0+)  
**ADR Reference**: [ADR-028: Advanced Code Browser UI v0.2.0](../../docs/adr/028-advanced-code-browser-v020.md)  
**Purpose**: Language-aware AST parsing, code chunking, syntax highlighting, and complexity analysis

## Architecture Overview

Tree-Sitter is integrated as the **core AST parsing engine** across MCB. It provides:
1. **Code Chunking**: Language-aware semantic code splitting via `IntelligentChunker`
2. **Syntax Highlighting**: Server-side code highlighting via `HighlightServiceImpl`
3. **Complexity Analysis**: AST-based code metrics via `ComplexityAnalyzer`
4. **Symbol Extraction**: Function/class/method discovery via `SymbolExtractor`

### Design Pattern
- **Pattern**: Language-specific processors + Fallback to generic chunking
- **Scope**: 14 programming languages (Rust, Python, JS, TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin)
- **Integration**: Async chunking with `tokio::task::spawn_blocking`

---

## Core Language Support

### 1. Language Processor Registry
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:24-84`

| Line | Component | Purpose |
|------|-----------|---------|
| 25-84 | `LANGUAGE_PROCESSORS` LazyLock | Static registry of language processors |
| 30-33 | Rust processor | `RustProcessor::new()` |
| 34-37 | Python processor | `PythonProcessor::new()` |
| 38-41 | JavaScript processor | `JavaScriptProcessor::new(false)` |
| 42-45 | TypeScript processor | `JavaScriptProcessor::new(true)` |
| 46-49 | Go processor | `GoProcessor::new()` |
| 50-53 | Java processor | `JavaProcessor::new()` |
| 54-57 | C processor | `CProcessor::new()` |
| 58-61 | C++ processor | `CppProcessor::new()` |
| 62-65 | C# processor | `CSharpProcessor::new()` |
| 66-69 | Ruby processor | `RubyProcessor::new()` |
| 70-73 | PHP processor | `PhpProcessor::new()` |
| 74-77 | Swift processor | `SwiftProcessor::new()` |
| 78-81 | Kotlin processor | `KotlinProcessor::new()` |

---

## Code Chunking Engine

### 2. Intelligent Chunker
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:86-191`

| Line | Component | Purpose |
|------|-----------|---------|
| 86-88 | `IntelligentChunker` struct | Main chunking orchestrator |
| 91-94 | `new()` | Constructor |
| 97-121 | `chunk_code()` | Synchronous chunking with fallback |
| 104 | `self.parse_with_tree_sitter(content, processor.get_language())` | Tree-sitter parsing |
| 106-107 | `processor.extract_chunks_with_tree_sitter(&tree, ...)` | Language-specific extraction |
| 113-116 | Fallback to generic chunking | On parse failure |
| 124-136 | `chunk_code_async()` | Async chunking via spawn_blocking |
| 139-172 | `chunk_generic()` | Fallback chunking for unsupported languages |
| 175-190 | `parse_with_tree_sitter()` | Tree-sitter parser initialization |
| 180 | `let mut parser = tree_sitter::Parser::new();` | Parser creation |
| 182 | `parser.set_language(&language)` | Language configuration |
| 186 | `parser.parse(content, None)` | Parsing call |

### 3. Language Processor Trait
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/common/processor.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 32-35 | `extract_chunks_with_tree_sitter()` | Trait method for chunk extraction |
| 42 | `get_language()` | Get tree-sitter language |
| 70-72 | Default implementation | Generic fallback |

### 4. Language Configuration
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/common/config.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 70 | `pub ts_language: tree_sitter::Language,` | Tree-sitter language field |
| 79 | `pub fn new(language: tree_sitter::Language) -> Self` | Constructor |
| 105-106 | `get_language()` | Language getter |

---

## Language-Specific Processors

### 5. Rust Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/rust.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 24 | `LanguageConfig::new(tree_sitter_rust::LANGUAGE.into())` | Rust language config |
| 58-66 | `extract_chunks_with_tree_sitter()` | Rust-specific chunking |

### 6. Python Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/python.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `LanguageConfig::new(tree_sitter_python::LANGUAGE.into())` | Python language config |
| 50-58 | `extract_chunks_with_tree_sitter()` | Python-specific chunking |

### 7. JavaScript/TypeScript Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/javascript.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 26-28 | TypeScript/JavaScript language selection | Conditional language config |
| 60-68 | `extract_chunks_with_tree_sitter()` | JS/TS-specific chunking |

### 8. Go Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/go.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `LanguageConfig::new(tree_sitter_go::LANGUAGE.into())` | Go language config |
| 51-59 | `extract_chunks_with_tree_sitter()` | Go-specific chunking |

### 9. Java Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/java.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `LanguageConfig::new(tree_sitter_java::LANGUAGE.into())` | Java language config |
| 51-59 | `extract_chunks_with_tree_sitter()` | Java-specific chunking |

### 10. C/C++/C# Processors
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/c.rs`, `cpp.rs`, `csharp.rs`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `c.rs` | 25 | `tree_sitter_c::LANGUAGE.into()` | C language config |
| `cpp.rs` | 25 | `tree_sitter_cpp::LANGUAGE.into()` | C++ language config |
| `csharp.rs` | 25 | `tree_sitter_c_sharp::LANGUAGE.into()` | C# language config |

### 11. Ruby/PHP/Swift Processors
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/ruby.rs`, `php.rs`, `swift.rs`

| File | Line | Component | Purpose |
|------|------|-----------|---------|
| `ruby.rs` | 24 | `tree_sitter_ruby::LANGUAGE.into()` | Ruby language config |
| `php.rs` | 25 | `tree_sitter_php::LANGUAGE_PHP.into()` | PHP language config |
| `swift.rs` | 25 | `tree_sitter_swift::LANGUAGE.into()` | Swift language config |

### 12. Kotlin Processor
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/kotlin.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 25 | `tree_sitter_kotlin_ng::LANGUAGE.into()` | Kotlin language config |

---

## AST Traversal

### 13. Tree Traverser
**File**: `/home/marlonsc/mcb/crates/mcb-providers/src/language/common/traverser.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! This module provides the AstTraverser that walks tree-sitter ASTs` | Module documentation |
| 60 | `cursor: &mut tree_sitter::TreeCursor,` | Tree cursor parameter |
| 114 | `fn extract_node_content(node: tree_sitter::Node, content: &str)` | Node content extraction |
| 131 | `node: tree_sitter::Node,` | Node parameter |
| 162 | `node: tree_sitter::Node,` | Node parameter |
| 199 | `fn create_chunk_from_node(&self, node: tree_sitter::Node, ...)` | Chunk creation from node |

---

## Syntax Highlighting

### 14. Highlight Service
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/services/highlight_service.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1-6 | Module documentation | Tree-sitter based highlighting |
| 12-13 | Imports | `tree_sitter::Language`, `tree_sitter_highlight::*` |
| 50-51 | `HighlightServiceImpl` struct | Highlight service implementation |
| 52 | `highlighter: Arc<tokio::sync::Mutex<Highlighter>>` | Async-safe highlighter |
| 55-60 | `new()` | Constructor |
| 63-129 | `get_language_config()` | Language configuration lookup |
| 67-71 | Rust highlighting | `tree_sitter_rust::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 72-76 | Python highlighting | `tree_sitter_python::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 77-81 | JavaScript highlighting | `tree_sitter_javascript::LANGUAGE`, `HIGHLIGHT_QUERY` |
| 82-86 | TypeScript highlighting | `tree_sitter_typescript::LANGUAGE_TYPESCRIPT`, `HIGHLIGHTS_QUERY` |
| 87-91 | TSX highlighting | `tree_sitter_typescript::LANGUAGE_TSX`, `HIGHLIGHTS_QUERY` |
| 92-96 | Go highlighting | `tree_sitter_go::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 97-101 | Java highlighting | `tree_sitter_java::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 102-106 | C highlighting | `tree_sitter_c::LANGUAGE`, `HIGHLIGHT_QUERY` |
| 107-111 | C++ highlighting | `tree_sitter_cpp::LANGUAGE`, `HIGHLIGHT_QUERY` |
| 112-116 | Ruby highlighting | `tree_sitter_ruby::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 117-121 | PHP highlighting | `tree_sitter_php::LANGUAGE_PHP`, `HIGHLIGHTS_QUERY` |
| 122-126 | Swift highlighting | `tree_sitter_swift::LANGUAGE`, `HIGHLIGHTS_QUERY` |
| 131-146 | `create_highlight_config()` | Configuration creation |
| 135-141 | `HighlightConfiguration::new()` | Config instantiation |
| 144 | `config.configure(&HIGHLIGHT_NAMES)` | Highlight name configuration |
| 148-150 | `highlight_code_internal()` | Internal highlighting method |

### 15. Highlight Category Mapping
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/services/highlight_service.rs:34-48`

| Line | Component | Purpose |
|------|-----------|---------|
| 35-48 | `map_highlight_to_category()` | Maps tree-sitter names to categories |
| 37 | `"keyword" => HighlightCategory::Keyword` | Keyword mapping |
| 38 | `"string" => HighlightCategory::String` | String mapping |
| 39 | `"comment" => HighlightCategory::Comment` | Comment mapping |
| 40 | `"function" => HighlightCategory::Function` | Function mapping |
| 41 | `"type" => HighlightCategory::Type` | Type mapping |
| 42 | `"variable" => HighlightCategory::Variable` | Variable mapping |
| 43 | `"number" => HighlightCategory::Number` | Number mapping |
| 44 | `"operator" => HighlightCategory::Operator` | Operator mapping |
| 45 | `"punctuation" => HighlightCategory::Punctuation` | Punctuation mapping |

### 16. Highlight Constants
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/src/constants/highlight.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `/// Tree-sitter highlight capture names (order must match HighlightConfiguration)` | Documentation |

---

## Complexity Analysis

### 17. Complexity Analyzer
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/complexity.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! Provides tree-sitter based complexity analysis independent of rust-code-analysis.` | Module documentation |
| 7 | `use tree_sitter::Node;` | Node import |
| 25 | `pub struct ComplexityAnalyzer` | Analyzer struct |

---

## Symbol Extraction

### 18. Symbol Extractor
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/symbols.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 7 | `use tree_sitter::{Node, Tree};` | Tree and Node imports |

---

## AST Utilities

### 19. Walker
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/walker.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! Provides utilities for walking AST trees using tree-sitter.` | Module documentation |
| 5 | `use tree_sitter::{Node, Tree};` | Tree and Node imports |

### 20. Cursor Utilities
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/cursor.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! Provides utilities for working with tree-sitter cursors.` | Module documentation |
| 5 | `use tree_sitter::{Node, TreeCursor};` | Cursor import |
| 7 | `pub struct CursorUtils` | Cursor utilities struct |

### 21. Visitor Pattern
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/visitor.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 5 | `use tree_sitter::Node;` | Node import |

---

## Validation & Code Analysis

### 22. Rust Extractor
**File**: `/home/marlonsc/mcb/crates/mcb-validate/src/extractor/rust_extractor.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! Uses `rust-code-analysis` (which wraps `tree-sitter`) to parse Rust source` | Module documentation |
| 58 | `let mut stack = vec![root.0]; // Access inner tree-sitter node` | Tree-sitter node access |

### 23. Duplication Detector
**File**: `/home/marlonsc/mcb/crates/mcb-validate/src/duplication/detector.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 3 | `//! Provides accurate clone detection using tree-sitter AST analysis.` | Module documentation |
| 211 | `/// tree-sitter for language-aware tokenization.` | Documentation |

### 24. Unwrap Detector
**File**: `/home/marlonsc/mcb/crates/mcb-validate/src/ast/unwrap_detector.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 4 | `//! Replaces the tree-sitter direct implementation with RCA Callback pattern.` | Module documentation |
| 83 | `// Recurse through children via inner tree-sitter node (public in our fork)` | Comment on tree-sitter usage |

### 25. AST Decoder
**File**: `/home/marlonsc/mcb/crates/mcb-validate/src/ast/decoder.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 7 | `use tree_sitter::Node;` | Node import |
| 16 | `pub fn decode_tree(tree: &tree_sitter::Tree, source: &str) -> AstNode` | Tree decoding function |

---

## Testing & Validation

### 26. AST Utils Tests
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/tests/unit/common.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 1 | `use tree_sitter::Parser;` | Parser import |
| 3 | `pub fn parse_rust(code: &str) -> tree_sitter::Tree` | Rust parsing test helper |
| 6 | `.set_language(&tree_sitter_rust::LANGUAGE.into())` | Language configuration |
| 14 | `pub fn parse_python(code: &str) -> tree_sitter::Tree` | Python parsing test helper |

### 27. Walker Tests
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/tests/unit/walker_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 11 | `let mut parser = tree_sitter::Parser::new();` | Parser creation |
| 13 | `.set_language(&tree_sitter_rust::LANGUAGE.into())` | Language configuration |
| (Multiple) | Multiple test cases | Walker functionality tests |

### 28. Highlight Service Tests
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/tests/unit/highlight_service_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 4 | `use mcb_infrastructure::services::highlight_service::HighlightServiceImpl;` | Service import |
| 8 | `let service = HighlightServiceImpl::new();` | Service instantiation |

### 29. Server Highlight Tests
**File**: `/home/marlonsc/mcb/crates/mcb-server/tests/unit/highlight_service_tests.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 9 | `HighlightServiceImpl, map_highlight_to_category,` | Service and helper imports |
| 14 | `let service = HighlightServiceImpl::new();` | Service instantiation |

### 30. Golden E2E Tests
**File**: `/home/marlonsc/mcb/crates/mcb-server/tests/integration/golden_highlight_service_e2e_integration.rs`

| Line | Component | Purpose |
|------|-----------|---------|
| 7 | `use mcb_server::handlers::highlight_service::HighlightServiceImpl;` | Service import |
| 9-10 | `fn get_service() -> HighlightServiceImpl` | Service factory for E2E tests |

---

## Cargo.toml Dependencies

### 31. Core Tree-Sitter
**File**: `/home/marlonsc/mcb/crates/mcb-providers/Cargo.toml:85-101`

```toml
tree-sitter = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-python = { workspace = true }
tree-sitter-javascript = { workspace = true }
tree-sitter-typescript = { workspace = true }
tree-sitter-go = { workspace = true }
tree-sitter-java = { workspace = true }
tree-sitter-c = { workspace = true }
tree-sitter-cpp = { workspace = true }
tree-sitter-c-sharp = { workspace = true }
tree-sitter-ruby = { workspace = true }
tree-sitter-php = { workspace = true }
tree-sitter-swift = { workspace = true }
tree-sitter-kotlin-ng = { workspace = true }
```

### 32. Highlighting
**File**: `/home/marlonsc/mcb/crates/mcb-infrastructure/Cargo.toml:156-171`

```toml
tree-sitter = { workspace = true }
tree-sitter-highlight = { workspace = true }
tree-sitter-rust = { workspace = true }
tree-sitter-python = { workspace = true }
tree-sitter-javascript = { workspace = true }
tree-sitter-typescript = { workspace = true }
tree-sitter-go = { workspace = true }
tree-sitter-java = { workspace = true }
tree-sitter-c = { workspace = true }
tree-sitter-cpp = { workspace = true }
tree-sitter-c-sharp = { workspace = true }
tree-sitter-ruby = { workspace = true }
tree-sitter-php = { workspace = true }
tree-sitter-swift = { workspace = true }
tree-sitter-kotlin-ng = { workspace = true }
```

---

## ADR Alignment

### ADR-028: Advanced Code Browser UI v0.2.0
- **Status**: IMPLEMENTED (v0.1.2+)
- **Rationale**:
  - Tree-sitter chosen for accurate, efficient parsing
  - Language-aware chunking for semantic code splitting
  - Full syntax highlighting with line numbers
  - Supports 14 programming languages
  - Fallback to generic chunking for unsupported languages
- **Key Features**:
  - Tree view navigation with collapsible directories
  - Full syntax highlighting (tree-sitter based, not regex)
  - Line numbers with clickable links
  - Chunk boundaries visually marked
  - Minimap for large files
  - Word wrap toggle
  - Inline semantic search results highlighted
  - Similarity score visualization
- **Trade-offs**:
  - Requires language-specific parsers (14 crates)
  - Larger binary size
  - Parsing overhead for large files

---

## Error Handling

### Language Support Errors
**File**: `/home/marlonsc/mcb/crates/mcb-ast-utils/src/error.rs:17-18`

| Line | Component | Purpose |
|------|-----------|---------|
| 17-18 | `UnsupportedLanguage` error | Unsupported language error variant |

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Core Parsing** | `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:175-190` |
| **Language Registry** | `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:24-84` |
| **Chunking** | `/home/marlonsc/mcb/crates/mcb-providers/src/language/engine.rs:97-121` |
| **Highlighting** | `/home/marlonsc/mcb/crates/mcb-infrastructure/src/services/highlight_service.rs:50-150` |
| **Complexity** | `/home/marlonsc/mcb/crates/mcb-ast-utils/src/complexity.rs` |
| **Symbols** | `/home/marlonsc/mcb/crates/mcb-ast-utils/src/symbols.rs` |
| **Traversal** | `/home/marlonsc/mcb/crates/mcb-ast-utils/src/walker.rs` |
| **Languages** | 14 (Rust, Python, JS, TS, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, TSX) |
| **Tests** | 30+ test cases across walker, highlight, and E2E tests |
| **ADR** | ADR-028 (primary) |
| **Status** | IMPLEMENTED, production-ready |

