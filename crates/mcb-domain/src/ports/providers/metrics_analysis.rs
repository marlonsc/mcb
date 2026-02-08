//! Metrics Analysis Provider Port
//!
//! Port for code metrics analysis providers. Implementations use static
//! analysis tools (like rust-code-analysis) to extract complexity metrics,
//! maintainability indices, and other code quality measures.
//!
//! ## Provider Pattern
//!
//! This port follows the same pattern as [`EmbeddingProvider`] and
//! [`VectorStoreProvider`], enabling consistent provider registration,
//! factory creation, and feature-flag based compilation.
//!
//! ## Metrics Provided
//!
//! | Metric | Description |
//! |--------|-------------|
//! | Cyclomatic Complexity | Number of independent paths through code |
//! | Cognitive Complexity | Mental effort required to understand code |
//! | Maintainability Index | Combined metric for code maintainability |
//! | SLOC | Source lines of code |
//! | Halstead Metrics | Program vocabulary, length, volume, difficulty |

use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::value_objects::SupportedLanguage;

// ============================================================================
// Metrics Types
// ============================================================================

/// File-level code metrics
///
/// Contains comprehensive metrics for a single file including complexity,
/// maintainability, and size metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    /// File path
    pub file: String,

    /// Detected language
    pub language: Option<SupportedLanguage>,

    /// Cyclomatic complexity (McCabe's complexity)
    pub cyclomatic: f64,

    /// Cognitive complexity (SonarSource metric)
    pub cognitive: f64,

    /// Maintainability index (0-100, higher is better)
    pub maintainability_index: f64,

    /// Source lines of code (executable lines)
    pub sloc: usize,

    /// Physical lines of code (total lines including blank/comments)
    pub ploc: usize,

    /// Logical lines of code (statements)
    pub lloc: usize,

    /// Comment lines of code
    pub cloc: usize,

    /// Blank lines
    pub blank: usize,

    /// Halstead metrics (optional, may not be available for all languages)
    pub halstead: Option<HalsteadMetrics>,
}

impl Default for FileMetrics {
    fn default() -> Self {
        Self {
            file: String::new(),
            language: None,
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            ploc: 0,
            lloc: 0,
            cloc: 0,
            blank: 0,
            halstead: None,
        }
    }
}

/// Halstead software science metrics
///
/// Measures program complexity based on operators and operands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    /// Number of distinct operators
    pub n1: usize,
    /// Number of distinct operands
    pub n2: usize,
    /// Total number of operators
    pub n1_total: usize,
    /// Total number of operands
    pub n2_total: usize,
    /// Program vocabulary (n1 + n2)
    pub vocabulary: usize,
    /// Program length (N1 + N2)
    pub length: usize,
    /// Calculated length estimate
    pub calculated_length: f64,
    /// Program volume
    pub volume: f64,
    /// Program difficulty
    pub difficulty: f64,
    /// Programming effort
    pub effort: f64,
    /// Estimated bugs (B = V / 3000)
    pub bugs: f64,
    /// Estimated time to program (seconds)
    pub time: f64,
}

/// Function-level metrics
///
/// Contains metrics for a single function/method within a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetrics {
    /// Function name
    pub name: String,

    /// Start line number (1-indexed)
    pub start_line: usize,

    /// End line number (1-indexed)
    pub end_line: usize,

    /// Cyclomatic complexity
    pub cyclomatic: f64,

    /// Cognitive complexity
    pub cognitive: f64,

    /// Source lines of code in this function
    pub sloc: usize,

    /// Number of parameters
    pub parameters: usize,

    /// Nesting depth (maximum)
    pub nesting_depth: usize,
}

// ============================================================================
// Provider Trait
// ============================================================================

/// Code Metrics Analysis Provider
///
/// Defines the contract for providers that analyze source code and extract
/// complexity metrics. Implementations typically use tools like rust-code-analysis
/// (RCA) or other static analysis tools.
#[async_trait]
pub trait MetricsAnalysisProvider: Send + Sync {
    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Get languages supported by this provider
    fn supported_languages(&self) -> &[SupportedLanguage];

    /// Check if a language is supported
    fn supports_language(&self, lang: SupportedLanguage) -> bool {
        self.supported_languages().contains(&lang)
    }

    /// Analyze a file and return its metrics
    ///
    /// # Arguments
    /// * `path` - Path to the source file
    ///
    /// # Returns
    /// File-level metrics or an error if the file cannot be analyzed
    async fn analyze_file(&self, path: &Path) -> Result<FileMetrics>;

    /// Analyze code content with specified language
    ///
    /// # Arguments
    /// * `content` - Source code content as bytes
    /// * `language` - Programming language of the content
    /// * `file_path` - Optional file path for context (used in results)
    ///
    /// # Returns
    /// File-level metrics for the provided code
    async fn analyze_code(
        &self,
        content: &[u8],
        language: SupportedLanguage,
        file_path: Option<&str>,
    ) -> Result<FileMetrics>;

    /// Get function-level metrics for a file
    ///
    /// # Arguments
    /// * `path` - Path to the source file
    ///
    /// # Returns
    /// Vector of function-level metrics, ordered by start line
    async fn analyze_functions(&self, path: &Path) -> Result<Vec<FunctionMetrics>>;

    /// Get metrics for a specific function in a file
    ///
    /// # Arguments
    /// * `path` - Path to the source file
    /// * `function_name` - Name of the function to analyze
    ///
    /// # Returns
    /// Metrics for the specified function, or None if not found
    async fn analyze_function(
        &self,
        path: &Path,
        function_name: &str,
    ) -> Result<Option<FunctionMetrics>> {
        let functions = self.analyze_functions(path).await?;
        Ok(functions.into_iter().find(|f| f.name == function_name))
    }

    /// Check if a file can be analyzed
    ///
    /// # Arguments
    /// * `path` - Path to check
    ///
    /// # Returns
    /// true if the file's language is supported
    fn can_analyze(&self, path: &Path) -> bool {
        SupportedLanguage::from_path(path)
            .map(|lang| self.supports_language(lang))
            .unwrap_or(false)
    }
}
