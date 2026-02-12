//! Native PMAT-style analysis provider ports.

use std::path::{Path, PathBuf};

use crate::error::Result;

/// Complexity finding for a single function.
#[derive(Debug, Clone)]
pub struct ComplexityFinding {
    /// Source file where the function is defined.
    pub file: PathBuf,
    /// Function name.
    pub function: String,
    /// Computed cyclomatic complexity.
    pub complexity: u32,
}

/// Dead code finding for a single symbol.
#[derive(Debug, Clone)]
pub struct DeadCodeFinding {
    /// Source file where the symbol is defined.
    pub file: PathBuf,
    /// 1-based line number of the declaration.
    pub line: usize,
    /// Symbol type (e.g. function).
    pub item_type: String,
    /// Symbol name.
    pub name: String,
}

/// TDG finding for a single file.
#[derive(Debug, Clone)]
pub struct TdgFinding {
    /// Source file for the score.
    pub file: PathBuf,
    /// Computed technical debt score.
    pub score: u32,
}

/// Complexity analysis provider.
pub trait ComplexityAnalyzer: Send + Sync {
    /// Analyze workspace and return functions above the provided threshold.
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<ComplexityFinding>>;
}

/// Dead code detection provider.
pub trait DeadCodeDetector: Send + Sync {
    /// Analyze workspace and return dead code symbols.
    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<DeadCodeFinding>>;
}

/// Technical Debt Gradient scoring provider.
pub trait TdgScorer: Send + Sync {
    /// Analyze workspace and return files with score above threshold.
    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<TdgFinding>>;
}
