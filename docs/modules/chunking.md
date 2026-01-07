# chunking Module

**Source**: `src/chunking/`
**Files**: 7
**Lines of Code**: 1152
**Traits**: 1
**Structs**: 7
**Enums**: 0
**Functions**: 0

## Overview

Intelligent code chunking using tree-sitter for structural parsing
//!
Provides language-aware chunking that respects code structure rather than
naive line-based or character-based splitting.

## Key Exports

`config::{LanguageConfig, NodeExtractionRule, NodeExtractionRuleBuilder},engine::IntelligentChunker,processor::LanguageProcessor,languages::*,`

## File Structure

```text
config.rs
processor.rs
traverser.rs
fallback.rs
engine.rs
languages/mod.rs
mod.rs
```

---

*Auto-generated from source code on qua 07 jan 2026 11:52:25 -03*
