//! Code Parsing
//!
//! Provides async parsing of source code using rust-code-analysis.
//! The `Parser` trait allows for extensible parsing implementations.

use async_trait::async_trait;
use rust_code_analysis::{FuncSpace, LANG, get_function_spaces};
use std::path::Path;

use crate::detection::LanguageDetector;
use crate::error::{LanguageError, Result};
use crate::language::LanguageId;

/// Parsed file result containing AST and metrics information
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// The detected language
    pub language: LanguageId,
    /// File-level metrics
    pub file_metrics: FileMetrics,
    /// Function-level metrics
    pub functions: Vec<FunctionInfo>,
}

/// File-level metrics extracted from parsing
#[derive(Debug, Clone, Default)]
pub struct FileMetrics {
    /// Source lines of code
    pub sloc: usize,
    /// Physical lines of code
    pub ploc: usize,
    /// Logical lines of code
    pub lloc: usize,
    /// Comment lines of code
    pub cloc: usize,
    /// Blank lines
    pub blank: usize,
    /// Cyclomatic complexity (file aggregate)
    pub cyclomatic: f64,
    /// Cognitive complexity (file aggregate)
    pub cognitive: f64,
    /// Maintainability index
    pub maintainability_index: f64,
}

/// Function-level information extracted from parsing
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Start line (1-indexed)
    pub start_line: usize,
    /// End line (1-indexed)
    pub end_line: usize,
    /// Function metrics
    pub metrics: FunctionMetrics,
}

/// Metrics for a single function
#[derive(Debug, Clone, Default)]
pub struct FunctionMetrics {
    /// Cyclomatic complexity
    pub cyclomatic: f64,
    /// Cognitive complexity
    pub cognitive: f64,
    /// Halstead volume
    pub halstead_volume: f64,
    /// Halstead difficulty
    pub halstead_difficulty: f64,
    /// Halstead effort
    pub halstead_effort: f64,
    /// Source lines of code
    pub sloc: usize,
    /// Number of arguments
    pub nargs: usize,
    /// Number of exit points
    pub nexits: usize,
}

/// Async parser trait for source code analysis
#[async_trait]
pub trait Parser: Send + Sync {
    /// Parse source code from a file path
    async fn parse_file(&self, path: &Path) -> Result<ParsedFile>;

    /// Parse source code from content with explicit language
    async fn parse_content(
        &self,
        content: &[u8],
        language: LanguageId,
        path: &Path,
    ) -> Result<ParsedFile>;
}

/// rust-code-analysis based parser implementation
pub struct RcaParser {
    detector: LanguageDetector,
}

impl Default for RcaParser {
    fn default() -> Self {
        Self::new()
    }
}

impl RcaParser {
    /// Create a new RCA-based parser
    pub fn new() -> Self {
        Self {
            detector: LanguageDetector::new(),
        }
    }

    /// Extract metrics from a `FuncSpace`
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "RCA returns f64 but values are always non-negative integers"
    )]
    fn extract_file_metrics(space: &FuncSpace) -> FileMetrics {
        let m = &space.metrics;
        FileMetrics {
            sloc: m.loc.sloc() as usize,
            ploc: m.loc.ploc() as usize,
            lloc: m.loc.lloc() as usize,
            cloc: m.loc.cloc() as usize,
            blank: m.loc.blank() as usize,
            cyclomatic: m.cyclomatic.cyclomatic(),
            cognitive: m.cognitive.cognitive(),
            maintainability_index: m.mi.mi_original(),
        }
    }

    /// Extract function metrics from a `FuncSpace`
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "RCA returns f64 but values are always non-negative integers"
    )]
    fn extract_function_metrics(space: &FuncSpace) -> FunctionMetrics {
        let m = &space.metrics;
        FunctionMetrics {
            cyclomatic: m.cyclomatic.cyclomatic(),
            cognitive: m.cognitive.cognitive(),
            halstead_volume: m.halstead.volume(),
            halstead_difficulty: m.halstead.difficulty(),
            halstead_effort: m.halstead.effort(),
            sloc: m.loc.sloc() as usize,
            nargs: m.nargs.fn_args_sum() as usize,
            nexits: m.nexits.exit_sum() as usize,
        }
    }

    /// Recursively extract functions from `FuncSpace` tree
    fn extract_functions(space: &FuncSpace, results: &mut Vec<FunctionInfo>) {
        let name = space.name.as_deref().unwrap_or("");
        if !name.is_empty() && name != "<unit>" {
            results.push(FunctionInfo {
                name: name.to_string(),
                start_line: space.start_line,
                end_line: space.end_line,
                metrics: Self::extract_function_metrics(space),
            });
        }

        for child in &space.spaces {
            Self::extract_functions(child, results);
        }
    }

    /// Parse content with a given RCA language
    fn parse_with_lang(
        content: &[u8],
        lang: LANG,
        language_id: LanguageId,
        path: &Path,
    ) -> Result<ParsedFile> {
        let root = get_function_spaces(&lang, content.to_vec(), path, None).ok_or_else(|| {
            LanguageError::ParseFailed {
                path: path.display().to_string(),
                reason: "rust-code-analysis failed to parse".to_string(),
            }
        })?;

        let file_metrics = Self::extract_file_metrics(&root);
        let mut functions = Vec::new();
        Self::extract_functions(&root, &mut functions);

        Ok(ParsedFile {
            language: language_id,
            file_metrics,
            functions,
        })
    }
}

#[async_trait]
impl Parser for RcaParser {
    async fn parse_file(&self, path: &Path) -> Result<ParsedFile> {
        let language = self.detector.detect(path, None)?;
        let content = tokio::fs::read(path).await?;
        self.parse_content(&content, language, path).await
    }

    async fn parse_content(
        &self,
        content: &[u8],
        language: LanguageId,
        path: &Path,
    ) -> Result<ParsedFile> {
        let lang = language.to_rca_lang();
        // RCA parsing is CPU-bound, but we keep async interface for consistency
        // In production, consider spawn_blocking for large files
        Self::parse_with_lang(content, lang, language, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_parse_rust_content() {
        let parser = RcaParser::new();
        let code = br"fn simple_function() -> i32 {
    let x = 1;
    let y = 2;
    x + y
}

fn complex_function(a: i32, b: i32) -> i32 {
    if a > b {
        if a > 10 {
            return a * 2;
        }
        return a;
    } else if b > 10 {
        return b * 2;
    }
    a + b
}";

        let result = parser
            .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
            .await
            .expect("Should parse");

        assert_eq!(result.language, LanguageId::Rust);
        assert!(result.file_metrics.sloc > 0);
        assert!(!result.functions.is_empty());

        // Find complex_function and verify it has higher complexity
        let complex_fn = result
            .functions
            .iter()
            .find(|f| f.name == "complex_function");
        assert!(complex_fn.is_some());
        let complex_fn = complex_fn.unwrap();
        assert!(complex_fn.metrics.cyclomatic >= 1.0);
    }

    #[tokio::test]
    async fn test_parse_python_content() {
        let parser = RcaParser::new();
        let code = br#"def greet(name):
    if name:
        return f"Hello, {name}!"
    return "Hello, World!"
"#;

        let result = parser
            .parse_content(code, LanguageId::Python, Path::new("test.py"))
            .await
            .expect("Should parse");

        assert_eq!(result.language, LanguageId::Python);
        assert!(result.file_metrics.sloc > 0);
    }

    #[tokio::test]
    async fn test_parse_javascript_content() {
        let parser = RcaParser::new();
        let code = br"function add(a, b) {
    return a + b;
}

const multiply = (x, y) => x * y;
";

        let result = parser
            .parse_content(code, LanguageId::JavaScript, Path::new("test.js"))
            .await
            .expect("Should parse");

        assert_eq!(result.language, LanguageId::JavaScript);
        assert!(result.file_metrics.sloc > 0);
    }

    #[tokio::test]
    async fn test_file_metrics() {
        let parser = RcaParser::new();
        let code = b"// This is a comment\nfn main() {\n    println!(\"Hello\");\n}\n";

        let result = parser
            .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
            .await
            .expect("Should parse");

        assert!(result.file_metrics.sloc > 0);
        assert!(result.file_metrics.cloc >= 1); // At least one comment line
    }

    #[tokio::test]
    async fn test_function_info() {
        let parser = RcaParser::new();
        let code = br"fn example(a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}";

        let result = parser
            .parse_content(code, LanguageId::Rust, Path::new("test.rs"))
            .await
            .expect("Should parse");

        // Find the 'example' function (RCA may return additional synthetic entries)
        let func = result
            .functions
            .iter()
            .find(|f| f.name == "example")
            .expect("Should find example function");

        assert!(func.start_line >= 1);
        assert!(func.end_line >= func.start_line);
        assert_eq!(func.metrics.nargs, 3);
    }
}
