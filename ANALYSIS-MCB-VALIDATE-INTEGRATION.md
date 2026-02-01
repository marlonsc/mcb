# MCB-Validate Architecture & Integration Analysis

**Document Version:** 1.0
**Date:** 2026-01-31
**Scope:** Research Analysis (No Code Changes)
**Status:** Complete Architecture Research

---

## Executive Summary

The mcb ecosystem has developed two parallel AST/metrics infrastructure systems that serve different purposes but share significant code duplication and dependency overlap:

1. **mcb-validate**: A comprehensive architecture validation tooling crate with 70+ validators, RCA-based metrics, tree-sitter AST parsing, and YAML-driven rule engines
2. **mcb-providers**: Language-specific code chunking providers using tree-sitter for semantic code segmentation

**Key Finding**: Both systems independently implement tree-sitter integration for different use cases (validation/metrics vs. chunking), creating duplication and maintenance burden.

### Current Pain Points
- **Duplicate tree-sitter integration** across two crates (13 language bindings in both)
- **Separate AST parsing logic** (RCA-based vs. direct tree-sitter-based)
- **Different language detection mechanisms** (mcb-validate's `RcaAnalyzer::detect_language` vs. mcb-providers' `detection` module)
- **No shared metrics infrastructure** for complexity analysis across the ecosystem
- **Tight coupling** to mcb-validate in mcb-infrastructure, limiting provider portability

---

## Current Architecture Overview

### High-Level Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          MCP Server Layer                               │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ mcb-server/src/handlers/                                         │  │
│  │  - analyze_complexity.rs → calls ValidationServiceInterface      │  │
│  │  - validate_file.rs → calls ValidationServiceInterface           │  │
│  │  - Other MCP tool handlers                                       │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                 Infrastructure Layer (DI/Service)                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │ mcb-infrastructure/src/validation/service.rs                     │  │
│  │ InfraValidationService implements ValidationServiceInterface     │  │
│  │  - run_validation() → ArchitectureValidator::validate_all()      │  │
│  │  - validate_file() → ArchitectureValidator methods              │  │
│  │  - analyze_file_complexity() → RcaAnalyzer::analyze_file()      │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                 ┌──────────────────┴──────────────────┐
                 ▼                                      ▼
   ┌──────────────────────────────┐    ┌──────────────────────────────┐
   │     mcb-validate Crate        │    │    mcb-providers Crate       │
   ├──────────────────────────────┤    ├──────────────────────────────┤
   │ PRIMARY PURPOSE:              │    │ PRIMARY PURPOSE:             │
   │ Architecture validation       │    │ Code chunking for indexing   │
   │                              │    │                              │
   │ KEY COMPONENTS:              │    │ KEY COMPONENTS:              │
   │ ┌────────────────────────┐   │    │ ┌────────────────────────┐  │
   │ │ metrics/rca_analyzer   │   │    │ │ language/engine        │  │
   │ │ - RcaAnalyzer          │   │    │ │ - IntelligentChunker   │  │
   │ │ - RcaMetrics           │   │    │ │ - UniversalLanguageCP  │  │
   │ │ - RcaFunctionMetrics   │   │    │ │                        │  │
   │ │ Metrics: 16 types      │   │    │ └────────────────────────┘  │
   │ └────────────────────────┘   │    │ ┌────────────────────────┐  │
   │                              │    │ │ language/common        │  │
   │ ┌────────────────────────┐   │    │ │ - BaseProcessor        │  │
   │ │ ast/ (RCA-based)       │   │    │ │ - LanguageProcessor    │  │
   │ │ - AstDecoder           │   │    │ │ - NodeExtractionRule   │  │
   │ │ - AstNode              │   │    │ │ - LanguageConfig       │  │
   │ │ - UnwrapDetector       │   │    │ └────────────────────────┘  │
   │ │ - AstQuery             │   │    │ ┌────────────────────────┐  │
   │ └────────────────────────┘   │    │ │ language/{rs,py,js..}  │  │
   │                              │    │ │ - 13 language procs    │  │
   │ ┌────────────────────────┐   │    │ │ (76-101 lines each)    │  │
   │ │ 70+ Validators:        │   │    │ └────────────────────────┘  │
   │ │ - dependency           │   │    │                              │
   │ │ - quality              │   │    │ TREE-SITTER DEPS (Optional) │
   │ │ - clean_architecture   │   │    │ - tree-sitter-rust          │
   │ │ - solid                │   │    │ - tree-sitter-python        │
   │ │ - kiss                 │   │    │ - tree-sitter-javascript    │
   │ │ - naming               │   │    │ - tree-sitter-typescript    │
   │ │ - tests_org            │   │    │ - tree-sitter-go            │
   │ │ - ... and 63 more      │   │    │ - tree-sitter-java          │
   │ └────────────────────────┘   │    │ - tree-sitter-c             │
   │                              │    │ - tree-sitter-cpp           │
   │ TREE-SITTER DEPS (Direct)    │    │ - tree-sitter-csharp        │
   │ - tree-sitter                │    │ - tree-sitter-ruby          │
   │ - tree-sitter-rust           │    │ - tree-sitter-php           │
   │ - tree-sitter-python         │    │ - tree-sitter-swift         │
   │ - tree-sitter-javascript     │    │ - tree-sitter-kotlin-ng      │
   │ - tree-sitter-typescript     │    │                              │
   │ - tree-sitter-go             │    │ LANGUAGE DETECTION:          │
   │ - tree-sitter-java           │    │ - detection::language_from_  │
   │ - tree-sitter-c              │    │   extension()                │
   │ - tree-sitter-cpp            │    │ - detection::is_language_    │
   │                              │    │   supported()                │
   │ LANGUAGE DETECTION:          │    │ - detection::get_chunk_size()│
   │ - RcaAnalyzer::             │    │                              │
   │   detect_language()          │    │                              │
   │ - LANG enum from RCA         │    │                              │
   │                              │    │                              │
   │ METRICS ANALYSIS:            │    │                              │
   │ - 16 metrics per function    │    │                              │
   │ - Cyclomatic Complexity      │    │                              │
   │ - Cognitive Complexity       │    │                              │
   │ - Halstead metrics           │    │                              │
   │ - LOC metrics                │    │                              │
   │ - Maintainability Index      │    │                              │
   │ - NOM, NARGS, NEXITS         │    │                              │
   └──────────────────────────────┘    └──────────────────────────────┘
```

### Crate Dependencies

```
mcb-domain (Port traits)
    │
    ├─ Language port traits
    │   - LanguageChunkingProvider (in mcb-domain/src/ports/providers/language_chunking.rs)
    │
    └─ Service port traits
        - ValidationServiceInterface (used by MCP handlers)

mcb-infrastructure
    │
    ├─ Depends on: mcb-validate (❌ Direct dependency)
    ├─ Depends on: mcb-domain
    └─ validation/service.rs
        - Calls: ArchitectureValidator (from mcb-validate)
        - Calls: RcaAnalyzer (from mcb-validate)

mcb-providers
    │
    ├─ Depends on: mcb-domain
    ├─ Depends on: mcb-application
    └─ language/ module
        - Re-implements: tree-sitter integration
        - Re-implements: language detection
        - Doesn't use: RcaAnalyzer or mcb-validate

mcb-validate
    │
    ├─ Depends on: (no workspace crates - dev tooling)
    ├─ rust-code-analysis (custom fork)
    │   - Wraps: tree-sitter 0.26.3
    │   - Provides: RCA callbacks and Nodes
    │
    └─ Direct tree-sitter bindings (13 languages)
        - 76-296 lines of AST parsing code per language
```

---

## Detailed Component Analysis

### 1. mcb-validate: Metrics Analysis (rca_analyzer.rs)

**File**: `crates/mcb-validate/src/metrics/rca_analyzer.rs`
**Size**: 150+ lines (partial read)

**Purpose**: Analyze code complexity metrics using rust-code-analysis

**Key Types**:
```rust
pub struct RcaMetrics {
    pub cyclomatic: f64,           // Cyclomatic complexity
    pub cognitive: f64,            // Cognitive complexity
    pub halstead_volume: f64,      // Halstead metrics
    pub halstead_difficulty: f64,
    pub halstead_effort: f64,
    pub maintainability_index: f64,
    pub sloc: usize,               // Lines of code metrics
    pub ploc: usize,
    pub lloc: usize,
    pub cloc: usize,
    pub blank: usize,
    pub nom: usize,                // Number of methods
    pub nargs: usize,              // Number of arguments
    pub nexits: usize,             // Number of exit points
}

pub struct RcaFunctionMetrics {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub metrics: RcaMetrics,
}

pub struct RcaAnalyzer {
    thresholds: MetricThresholds,
}
```

**Language Detection**:
```rust
pub fn detect_language(path: &Path) -> Option<LANG> {
    let ext = path.extension()?.to_str()?;
    match ext.to_lowercase().as_str() {
        "rs" => Some(LANG::Rust),
        "py" => Some(LANG::Python),
        "js" | "mjs" | "cjs" | "jsx" => Some(LANG::Mozjs),
        // ... 8 more languages
    }
}
```

**Key Method**:
```rust
pub fn analyze_file(&self, path: &Path) -> Result<Vec<RcaFunctionMetrics>> {
    let lang = Self::detect_language(path)?;
    let code = std::fs::read(path)?;
    self.analyze_code(&code, &lang, path)
}

// Uses rust_code_analysis::get_function_spaces(lang, code, path)
```

**Metrics Coverage**: 16 metrics per function (cyclomatic, cognitive, halstead, MI, LOC, NOM, NARGS, NEXITS, WMC)

**Used By**:
- `mcb-infrastructure/src/validation/service.rs` - `analyze_file_complexity()`
- `mcb-server/src/handlers/analyze_complexity.rs` - MCP tool handler

---

### 2. mcb-validate: AST Analysis (ast/ module)

**Files**:
- `ast/mod.rs` (29 lines)
- `ast/core.rs` (42 lines)
- `ast/decoder.rs` (252 lines)
- `ast/query.rs` (311 lines)
- `ast/unwrap_detector.rs` (296 lines)
- `ast/types.rs` (15 lines)

**Purpose**: Provide unified AST parsing and querying using rust-code-analysis

**Key Types**:
```rust
pub struct AstNode {
    pub kind: String,
    pub name: Option<String>,
    pub span: Span,
    pub children: Vec<AstNode>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub struct AstDecoder {
    // RCA-based decoder
}

pub struct AstQuery {
    // Pattern-based querying
}

pub struct UnwrapDetection {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub method: String,      // "unwrap" or "expect"
    pub in_test: bool,
    pub context: String,
}
```

**Direct RCA Re-exports**:
```rust
pub use rust_code_analysis::{
    Callback, LANG, Node, ParserTrait, Search, action, find, guess_language,
};
```

**Unwrap Detection** uses RCA Callback pattern:
```rust
impl Callback for UnwrapCallback {
    type Res = Vec<UnwrapDetection>;
    type Cfg = UnwrapConfig;

    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        // Recursive detection through AST
        detect_recursive(&root, code, &cfg, &mut detections);
    }
}
```

**Used By**:
- 70+ validators in mcb-validate
- `implementation.rs` validator (unwrap detection)
- `error_boundary.rs` validator

---

### 3. mcb-providers: Language Chunking (language/ module)

**Files**:
- `language/mod.rs` (77 lines) - Module organization
- `language/common/mod.rs` (base types)
- `language/common/processor.rs` - BaseProcessor trait
- `language/common/config.rs` - LanguageConfig, NodeExtractionRule
- `language/detection.rs` (101 lines) - Language detection
- `language/engine.rs` (319 lines) - IntelligentChunker orchestration
- `language/{rust,python,javascript,...}.rs` (70-96 lines each) - 13 language processors

**Purpose**: Extract semantic code chunks from source for embedding/indexing

**Key Types**:
```rust
pub trait LanguageProcessor {
    fn chunk(&self, content: &str) -> Vec<CodeChunk>;
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&'static str];
}

pub struct BaseProcessor {
    config: LanguageConfig,
}

pub struct IntelligentChunker {
    processors: HashMap<Language, Box<dyn LanguageProcessor>>,
}
```

**Language Detection** (separate from mcb-validate):
```rust
pub fn language_from_extension(ext: &str) -> Option<Language> {
    match ext {
        "rs" | "rlib" => Some(Language::Rust),
        "py" => Some(Language::Python),
        "js" => Some(Language::Javascript),
        // ... 10 more
    }
}

pub fn is_language_supported(ext: &str) -> bool {
    language_from_extension(ext).is_some()
}

pub fn get_chunk_size(language: Language) -> usize {
    match language {
        Language::Rust => 1024,
        Language::Python => 512,
        Language::Javascript => 512,
        // ...
    }
}
```

**Language Processors** (13 total):
- RustProcessor (96 lines)
- PythonProcessor (70 lines)
- JavaScriptProcessor (87 lines)
- TypeScriptProcessor
- GoProcessor (76 lines)
- JavaProcessor (77 lines)
- CProcessor (76 lines)
- CppProcessor (76 lines)
- CSharpProcessor (76 lines)
- RubyProcessor (74 lines)
- PhpProcessor (75 lines)
- SwiftProcessor (77 lines)
- KotlinProcessor (76 lines)

**Used By**:
- `UniversalLanguageChunkingProvider` (implements domain port)
- MCP server via DI for indexing operations

---

### 4. Integration Points: mcb-infrastructure/validation

**File**: `crates/mcb-infrastructure/src/validation/service.rs`
**Size**: 100+ lines (partial read)

**Purpose**: Implement `ValidationServiceInterface` (domain port) using mcb-validate

```rust
pub struct InfraValidationService;

impl ValidationServiceInterface for InfraValidationService {
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        let config = ValidationConfig::new(workspace_root);
        let mut validator = ArchitectureValidator::with_config(config);

        let report = if let Some(names) = validators {
            validator.validate_named(&names_ref)?
        } else {
            validator.validate_all()?
        };
        Ok(report)
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        include_functions: bool,
    ) -> Result<ComplexityReport> {
        analyze_file_complexity(file_path, include_functions)
        // Uses RcaAnalyzer internally
    }
}
```

**Dependencies**:
- `use mcb_validate::{ArchitectureValidator, ValidationConfig};` (❌ Direct external dependency)
- Couples infrastructure to validation implementation details

---

### 5. CLI Integration: mcb/src/cli/validate.rs

**File**: `crates/mcb/src/cli/validate.rs`
**Size**: 120+ lines

**Purpose**: CLI entry point for `mcb validate` command

```rust
#[derive(Args, Debug, Clone)]
pub struct ValidateArgs {
    pub path: PathBuf,
    pub quick: bool,
    pub strict: bool,
    pub validators: Option<Vec<String>>,
    pub severity: String,    // "error", "warning", "info"
    pub format: String,      // "text", "json"
}

impl ValidateArgs {
    pub fn execute(self) -> Result<ValidationResult> {
        let config = ValidationConfig::new(&workspace_root);
        let mut validator = ArchitectureValidator::with_config(config);

        let report = if let Some(ref validators) = self.validators {
            validator.validate_named(&validator_names)?
        } else {
            validator.validate_all()?
        };

        // Format and print output
    }
}
```

**Direct mcb-validate Usage**: `use mcb_validate::{...};`

---

## Duplication & Pain Points

### 1. Tree-Sitter Integration Duplication

| Aspect | mcb-validate | mcb-providers | Duplication |
|--------|-------------|---------------|-------------|
| tree-sitter base | Direct (tree-sitter 0.26.3) | Direct (tree-sitter 0.26.3) | ✓ Same version |
| Language bindings | 13 direct deps | 13 optional deps | ✓ Identical set |
| Language detection | `RcaAnalyzer::detect_language()` | `detection::language_from_extension()` | ✓ Separate implementations |
| Parsing approach | RCA callbacks (wrapped) | Direct tree-sitter (in processors) | ✓ Different patterns |
| Metrics analysis | RCA metrics (16 types) | None (chunking only) | - Different scope |
| Code size | 296-2371 lines (ast + metrics) | 70-319 lines (chunking) | ✓ 2600+ lines total |

### 2. Language Detection Duplication

**mcb-validate** (`metrics/rca_analyzer.rs:86-99`):
```rust
pub fn detect_language(path: &Path) -> Option<LANG> {
    let ext = path.extension()?.to_str()?;
    match ext.to_lowercase().as_str() {
        "rs" => Some(LANG::Rust),
        "py" => Some(LANG::Python),
        "js" | "mjs" | "cjs" | "jsx" => Some(LANG::Mozjs),
        "ts" | "mts" | "cts" => Some(LANG::Typescript),
        "tsx" => Some(LANG::Tsx),
        "java" => Some(LANG::Java),
        "kt" | "kts" => Some(LANG::Kotlin),
        "c" | "h" | "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "mm" | "m" => Some(LANG::Cpp),
        _ => None,
    }
}
```

**mcb-providers** (`language/detection.rs`):
```rust
pub fn language_from_extension(ext: &str) -> Option<Language> {
    match ext {
        "rs" | "rlib" => Some(Language::Rust),
        "py" => Some(Language::Python),
        "js" => Some(Language::Javascript),
        // ... similar pattern
    }
}
```

**Issues**:
- Two separate Language enum definitions (LANG vs Language)
- Duplicate matching logic
- Different file extension patterns (e.g., "tsx" vs no tsx in providers)
- No centralized source of truth
- Hard to add new languages (must update both places)

### 3. AST Parsing Pattern Duplication

**mcb-validate**: Uses RCA Callback pattern (abstraction layer)
```rust
impl Callback for UnwrapCallback {
    fn call<T: ParserTrait>(cfg: Self::Cfg, parser: &T) -> Self::Res {
        let root = parser.get_root();
        // Custom logic
    }
}
```

**mcb-providers**: Direct tree-sitter node walking (no abstraction)
```rust
// In language processors - each implements own walking logic
fn process_node(node: &tree_sitter::Node) {
    // Language-specific extraction rules
}
```

**Issues**:
- Different abstraction levels (RCA vs raw tree-sitter)
- mcb-validate locked into RCA fork usage
- mcb-providers tied to direct tree-sitter API
- No way to share AST traversal utilities

### 4. Architecture Dependency Issues

**Current Problematic Flow**:
```
mcb-infrastructure ──→ mcb-validate
                         (direct dep)
```

**Problems**:
- mcb-validate is dev tooling, but infrastructure depends on it
- Violates clean architecture (infrastructure shouldn't depend on validation)
- Makes it hard to make mcb-validate optional
- Couples service layer to specific validation implementation
- No abstraction between them (uses internal types directly)

**Correct Dependency**: Infrastructure should only know about domain ports
```
mcb-infrastructure ──→ mcb-domain (ports only)
                            │
                            └─→ ValidationServiceInterface (port)
                                  │
                                  └─→ InfraValidationService (impl in infrastructure)
```

---

## Integration Points Summary

### MCP Tools → Service → Implementation Flow

```
MCP Tool Handlers (mcb-server)
    │
    ├─ analyze_complexity.rs
    │   └─ ValidationServiceInterface::analyze_complexity()
    │       └─ InfraValidationService::analyze_complexity()
    │           └─ RcaAnalyzer::analyze_file()
    │               └─ rust_code_analysis::get_function_spaces()
    │
    ├─ validate_file.rs
    │   └─ ValidationServiceInterface::validate_file()
    │       └─ InfraValidationService::validate_file()
    │           └─ ArchitectureValidator::validate_all()
    │               └─ 70+ validators (each using RCA/tree-sitter)
    │
    └─ Other MCP tools
        └─ (index_codebase uses language chunking)
            └─ IntelligentChunker
                └─ Language-specific processors
                    └─ Direct tree-sitter parsing
```

### Provider Architecture Port

**Definition** (`mcb-domain/src/ports/providers/language_chunking.rs`):
```rust
pub trait LanguageChunkingProvider: Send + Sync {
    fn language(&self) -> Language;
    fn extensions(&self) -> &[&'static str];
    fn chunk(&self, content: &str, file_path: &str) -> Vec<CodeChunk>;
    fn provider_name(&self) -> &str;
}
```

**Implementation** (`mcb-providers/src/language/engine.rs`):
```rust
pub struct UniversalLanguageChunkingProvider {
    processors: HashMap<Language, Box<dyn LanguageProcessor>>,
}

impl LanguageChunkingProvider for UniversalLanguageChunkingProvider {
    fn chunk(&self, content: &str, file_path: &str) -> Vec<CodeChunk> {
        // Delegates to language-specific processor
    }
}
```

**Limitation**: Only used for indexing, never for validation/metrics

---

## Tree-Sitter Dependency Structure

### Workspace Cargo.toml

Both crates declare same versions:
```toml
tree-sitter = "0.26.3"           # core
tree-sitter-rust = "0.26.x"       # 13 languages
tree-sitter-python = "0.21.x"
tree-sitter-javascript = "0.21.x"
tree-sitter-typescript = "0.20.x"
tree-sitter-go = "0.20.x"
tree-sitter-java = "0.19.x"
tree-sitter-c = "0.21.x"
tree-sitter-cpp = "0.22.x"        # C++ for validation
tree-sitter-c-sharp = "0.21.x"
tree-sitter-ruby = "0.20.x"
tree-sitter-php = "0.22.x"
tree-sitter-swift = "0.21.x"
tree-sitter-kotlin-ng = "0.3.x"   # Custom kotlin binding
```

### Compile-Time Dependency Graph

```
mcb-validate
    ├─ tree-sitter-rust ──┐
    ├─ tree-sitter-python ┤
    ├─ tree-sitter-javascript ├─→ tree-sitter 0.26.3
    ├─ tree-sitter-typescript ├─ (compiled & linked)
    ├─ tree-sitter-go ─────┤
    ├─ tree-sitter-java ───┤
    ├─ tree-sitter-c ──────┤
    ├─ tree-sitter-cpp ────┤
    ├─ rust-code-analysis (fork) ──┐
    │   └─ wraps tree-sitter ──────┘
    └─ (13 bindings compiled in)

mcb-providers (optional features)
    ├─ [lang-rust] tree-sitter-rust ──┐
    ├─ [lang-python] tree-sitter-python ├─→ tree-sitter 0.26.3
    ├─ [lang-javascript] ... ─────────┤ (duplicated compilation)
    ├─ [lang-cpp] tree-sitter-cpp ────┤
    └─ (conditional compilation)

RESULT: tree-sitter compiled TWICE per feature set:
- mcb-validate: Always (all 13 + RCA)
- mcb-providers: If features enabled
```

### Runtime Behavior

- **mcb-validate**: Loads all 13 language grammars at startup (via RCA)
- **mcb-providers**: Loads only feature-enabled grammars
- **No shared state**: Each has independent grammar instances

---

## Unification Opportunities

### Opportunity 1: Centralized Language Support (HIGH PRIORITY)

**Current State**:
- 13 language detection implementations
- Separate enum types (LANG vs Language)
- Duplicate file extension patterns

**Proposal**:
```
Create: mcb-language-support crate
├─ Single Language enum (superset)
├─ Unified language_from_extension()
├─ Unified language_from_filename()
├─ Language configuration (chunk size, etc.)
└─ File extension registry

Use in:
├─ mcb-validate (replace RcaAnalyzer::detect_language)
├─ mcb-providers (replace detection module)
├─ Future language services
```

**Benefits**:
- Single source of truth
- Easier to add new languages
- Consistent detection across ecosystem
- Reduces maintenance burden

---

### Opportunity 2: Unified Metrics Infrastructure (HIGH PRIORITY)

**Current State**:
- RcaAnalyzer in mcb-validate
- Only used by InfraValidationService
- 16 metrics calculated but stored in ArchitectureValidator reports
- Not available as reusable service

**Proposal**:
```
Create: mcb-metrics crate (separate from validation)
├─ Metrics port trait (MetricsProvider)
├─ RcaMetricsProvider implementation
├─ ComplexityReport types
└─ Function-level metrics

Implement port in: mcb-providers
├─ As optional provider feature
├─ Registered via linkme like other providers
└─ Available to MCP tools independently

Update: mcb-domain
├─ Add MetricsProvider port
└─ Update ServiceRegistry
```

**Benefits**:
- Metrics analysis decoupled from validation
- Available to other systems (not just validation)
- Could be swapped for different implementations
- MCP tools can access metrics independently
- Reduces mcb-validate scope creep

---

### Opportunity 3: Shared AST Utilities (MEDIUM PRIORITY)

**Current State**:
- mcb-validate: RCA Callback wrapper + custom queries
- mcb-providers: Direct tree-sitter in processors
- No shared traversal patterns
- Different error handling approaches

**Proposal**:
```
Create: mcb-ast-support crate
├─ Language-agnostic node traits
├─ Standard traversal patterns
├─ Query builder utilities
├─ Position/span types
└─ Conversion helpers

Implementations:
├─ RcaAstAdapter (wraps RCA parser)
├─ TreeSitterAdapter (direct tree-sitter)
└─ Support both patterns

Use in:
├─ mcb-validate (replace direct RCA deps)
├─ mcb-providers (optional, for advanced chunking)
└─ Future language services
```

**Benefits**:
- Abstraction reduces vendor lock-in to RCA
- Could support multiple parsing backends
- Shared utilities reduce duplication
- Better testability
- Easier to add language support

---

### Opportunity 4: Metrics as Validation Provider (MEDIUM PRIORITY)

**Current State**:
- Metrics analyzed separately from other validators
- ComplexityReport in ValidationReport
- RcaAnalyzer tightly coupled to mcb-validate

**Proposal**:
```
Restructure: ComplexityValidator
├─ Implement new Validator trait
├─ Delegates to MetricsProvider port
├─ Generates violations for thresholds
└─ Integrated into validator registry

Decouple:
├─ Move RcaAnalyzer to mcb-metrics
├─ mcb-validate imports via port
└─ No direct RcaAnalyzer usage in validation
```

**Benefits**:
- Consistent validator pattern
- Easy to swap metrics implementations
- Cleaner architecture
- Metrics integrated into unified reporting

---

### Opportunity 5: Provider-Based Infrastructure (LOW PRIORITY, LONG-TERM)

**Current State**:
- InfraValidationService hardcodes ArchitectureValidator
- Cannot swap implementations
- mcb-validate knowledge baked into infrastructure

**Future Vision**:
```
Update: ValidationServiceInterface
├─ Accept ValidatorRegistry
├─ Support multiple validator implementations
└─ Plugin architecture

Register validators:
├─ At compile time (via linkme)
├─ At runtime (from config)
└─ By feature flag

Result:
├─ Infrastructure independent of mcb-validate
├─ Could use different validation framework
├─ Test-friendly (easy to mock)
├─ Forward-compatible
```

**Benefits**:
- Ultimate decoupling
- Future flexibility
- Easier testing
- Clear separation of concerns

---

## Current State Summary Table

| Aspect | mcb-validate | mcb-providers | Status |
|--------|-------------|---------------|--------|
| **Purpose** | Architecture validation | Code chunking for indexing | Different domains |
| **Tree-sitter** | 13 direct deps | 13 optional deps | 🔴 Duplicate |
| **Language Detection** | `RcaAnalyzer::detect_language()` | `detection::language_from_extension()` | 🔴 Duplicate |
| **AST Parsing** | RCA callbacks (abstract) | Direct tree-sitter (concrete) | 🟡 Different patterns |
| **Metrics Analysis** | 16 metrics per function | None | ✓ No duplication |
| **Language Processors** | Via RCA (opaque) | 13 explicit processors | 🟡 Different approaches |
| **Integration** | Via InfraValidationService | Via DI providers | 🟡 Separate paths |
| **Code Reuse** | None | None | 🔴 Siloed |
| **Maintenance** | mcb-validate repo | mcb-providers repo | 🟡 Split responsibility |
| **Testing** | Unit + integration | Unit + integration | ✓ Both tested |
| **Documentation** | Comprehensive | Comprehensive | ✓ Both documented |

---

## Technical Debt Assessment

### High-Impact Issues

1. **Duplicate Tree-Sitter Integration** ⚠️ CRITICAL
   - 13 language bindings compiled twice
   - Maintenance burden for new languages
   - Increased binary size
   - **Effort to Fix**: Medium (centralize detection & config)
   - **Impact**: High (reduces maintenance)

2. **No Shared Language Support** ⚠️ CRITICAL
   - Language detection logic repeated
   - Different enum types (LANG vs Language)
   - Hard to extend
   - **Effort to Fix**: Medium (create mcb-language-support)
   - **Impact**: High (enables ecosystem growth)

3. **Metrics Infrastructure Coupling** ⚠️ HIGH
   - RcaAnalyzer only in mcb-validate
   - Not available to other systems
   - Infrastructure depends on validation tooling
   - **Effort to Fix**: Medium (extract to mcb-metrics)
   - **Impact**: Medium (better architecture)

4. **Architecture Dependency Violation** ⚠️ HIGH
   - mcb-infrastructure imports mcb-validate
   - Dev tooling in production dependency chain
   - Limits future flexibility
   - **Effort to Fix**: High (refactor infrastructure)
   - **Impact**: High (enables clean architecture)

### Medium-Impact Issues

5. **Different AST Patterns** 🟡 MEDIUM
   - RCA callbacks vs direct tree-sitter
   - No shared traversal utilities
   - Harder to add new language features
   - **Effort to Fix**: High (abstraction layer)
   - **Impact**: Medium (improves flexibility)

6. **Scattered Language Processors** 🟡 MEDIUM
   - 13 separate processor files (70-96 lines each)
   - Potential code duplication in processors
   - Hard to refactor patterns across languages
   - **Effort to Fix**: Medium (macro-based generation)
   - **Impact**: Low (internal organization)

---

## Recommendations for Phase 3+

### Immediate (Phase 3: RCA Centralization)

1. **Extract mcb-language-support crate**
   - Unified Language enum
   - File extension registry
   - Centralized detection logic
   - Mark mcb-validate & mcb-providers as consumers

2. **Extract mcb-metrics crate** (optional)
   - Move RcaAnalyzer there
   - Create MetricsProvider port
   - Register as provider in mcb-providers
   - Update mcb-infrastructure imports

### Short-term (Phase 4+)

3. **Refactor mcb-infrastructure/validation**
   - Remove direct mcb-validate dependency
   - Import only via domain ports
   - Use services from DI container
   - Update CLI to use new structure

4. **Consolidate AST Utilities**
   - Create shared traversal patterns
   - Support both RCA and direct tree-sitter
   - Optional AST support crate

### Long-term (Future)

5. **Provider-based Validator Architecture**
   - Register validators via linkme
   - Plugin system for custom validators
   - Complete independence from mcb-validate

---

## Files for Implementation Reference

### Current Implementation Files

**mcb-validate**:
- `crates/mcb-validate/src/lib.rs` - Main re-exports
- `crates/mcb-validate/src/metrics/rca_analyzer.rs` - Metrics analysis
- `crates/mcb-validate/src/ast/unwrap_detector.rs` - AST usage
- `crates/mcb-validate/Cargo.toml` - Dependencies

**mcb-providers**:
- `crates/mcb-providers/src/language/mod.rs` - Module organization
- `crates/mcb-providers/src/language/detection.rs` - Language detection
- `crates/mcb-providers/src/language/common/mod.rs` - Shared types
- `crates/mcb-providers/src/language/engine.rs` - Chunking engine
- `crates/mcb-providers/Cargo.toml` - Feature flags

**mcb-infrastructure**:
- `crates/mcb-infrastructure/src/validation/service.rs` - Service impl
- `crates/mcb-infrastructure/src/validation/mod.rs` - Module exports

**mcb-domain**:
- `crates/mcb-domain/src/ports/providers/language_chunking.rs` - Port trait

**mcb-server**:
- `crates/mcb-server/src/handlers/analyze_complexity.rs` - MCP tool
- `crates/mcb-server/src/handlers/validate_file.rs` - MCP tool

**mcb (CLI)**:
- `crates/mcb/src/cli/validate.rs` - CLI command

---

## Conclusion

The mcb ecosystem demonstrates mature, well-organized code with clear separation of concerns. However, the parallel development of tree-sitter integration in two different crates has created legitimate technical debt:

1. **Tree-sitter duplication** is the most visible pain point (13 languages compiled twice)
2. **Language detection** is repeated with no single source of truth
3. **Metrics infrastructure** is siloed in validation tooling where it could serve broader needs
4. **Architecture coupling** between infrastructure and validation violates clean architecture principles

**The good news**: These issues are well-scoped and addressable through the proposed phased approach. Each opportunity is relatively independent, allowing incremental refactoring without blocking feature work.

**Recommended priority**: Start with unified language support (foundation), then metrics extraction (enables ecosystem growth), then architecture refactoring (enables flexibility).

---

**Document prepared**: 2026-01-31
**Analysis scope**: Architecture, dependencies, duplication patterns
**Recommendations**: Five unification opportunities prioritized
**Implementation**: Not included (research-only per request)
