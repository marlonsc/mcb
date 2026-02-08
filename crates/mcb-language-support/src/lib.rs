//! MCB Language Support
//!
//! Unified language abstraction for MCP Context Browser built on Mozilla's
//! rust-code-analysis (RCA) library. Provides language detection, parsing,
//! metrics analysis, and code chunking.
//!
//! ## Features
//!
//! - **Language Detection**: Identify programming languages from file extensions and content
//! - **Code Parsing**: Extract AST and metrics using rust-code-analysis
//! - **Chunking Strategies**: Break code into semantic units for embedding
//!
//! ## Supported Languages
//!
//! Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin
//!
//! ## Example
//!
//! ```no_run
//! use mcb_language_support::{LanguageDetector, LanguageId, RcaParser, Parser};
//! use std::path::Path;
//!
//! # async fn example() -> mcb_language_support::Result<()> {
//! // Detect language from file
//! let detector = LanguageDetector::new();
//! let lang = detector.detect(Path::new("main.rs"), None)?;
//! assert_eq!(lang, LanguageId::Rust);
//!
//! // Parse code and get metrics
//! let parser = RcaParser::new();
//! let parsed = parser.parse_file(Path::new("main.rs")).await?;
//! println!("File has {} functions", parsed.functions.len());
//! # Ok(())
//! # }
//! ```

pub mod chunking;
pub mod detection;
pub mod error;
pub mod language;
pub mod parser;

// Re-export main types for convenience
pub use chunking::{
    ChunkType, ChunkingConfig, ChunkingStrategy, LineBasedChunking, ParsedChunk, SemanticChunking,
};
pub use detection::LanguageDetector;
pub use error::{LanguageError, Result};
pub use language::{LanguageId, LanguageInfo, LanguageRegistry};
pub use parser::{
    FunctionInfo, ParsedFile, ParsedFileMetrics, ParsedFunctionMetrics, Parser, RcaParser,
};
/// Re-export rust-code-analysis LANG enum for direct access when needed
pub use rust_code_analysis::LANG;
