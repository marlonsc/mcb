//! Code analysis provider ports.

use std::path::{Path, PathBuf};

use crate::error::Result;

/// Unified analysis finding - all code analysis results use this type.
#[derive(Debug, Clone)]
pub enum AnalysisFinding {
    /// High cyclomatic complexity in a function.
    Complexity {
        /// File containing the complex function.
        file: PathBuf,
        /// Name of the function.
        function: String,
        /// Calculated complexity score.
        complexity: u32,
    },
    /// Dead code symbol detected.
    DeadCode {
        /// File containing the dead code.
        file: PathBuf,
        /// Line number where the item is defined.
        line: usize,
        /// Type of the dead item (e.g., function, struct).
        item_type: String,
        /// Name of the dead item.
        name: String,
    },
    /// Technical debt gradient score for a file.
    TechnicalDebt {
        /// File being scored.
        file: PathBuf,
        /// Normalized debt score.
        score: u32,
    },
}

/// Unified code analysis provider.
///
/// Covers complexity analysis, dead code detection, and technical debt scoring.
/// Implementations register via the `CODE_ANALYZERS` linkme distributed slice.
pub trait CodeAnalyzer: Send + Sync {
    /// Analyze code complexity in the workspace.
    ///
    /// # Errors
    /// Returns an error if workspace scanning or analysis fails.
    fn analyze_complexity(
        &self,
        workspace_root: &Path,
        threshold: u32,
    ) -> Result<Vec<AnalysisFinding>>;

    /// Detect dead code symbols in the workspace.
    ///
    /// # Errors
    /// Returns an error if workspace scanning fails.
    fn detect_dead_code(&self, workspace_root: &Path) -> Result<Vec<AnalysisFinding>>;

    /// Calculate technical debt gradient scores for workspace files.
    ///
    /// # Errors
    /// Returns an error if workspace scanning or scoring fails.
    fn score_tdg(&self, workspace_root: &Path, threshold: u32) -> Result<Vec<AnalysisFinding>>;
}
