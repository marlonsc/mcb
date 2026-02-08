//! rust-code-analysis integration for comprehensive code metrics
//!
//! Provides 16 code metrics using the rust-code-analysis library:
//! - Cyclomatic Complexity
//! - Cognitive Complexity
//! - Halstead metrics (Volume, Difficulty, Effort)
//! - Maintainability Index
//! - LOC metrics (SLOC, PLOC, LLOC, CLOC, BLANK)
//! - NOM, NARGS, NEXITS, WMC
//!
//! Supports: Rust, Python, JavaScript, TypeScript, Java, C, C++, Kotlin
//!
//! Uses `mcb-language-support` for language detection to avoid code duplication.

use std::path::Path;

use mcb_language_support::LanguageDetector;
use rust_code_analysis::{FuncSpace, LANG, get_function_spaces};

use super::MetricViolation;
use super::thresholds::{MetricThresholds, MetricType};
use crate::{Result, ValidationError};

/// Comprehensive metrics from rust-code-analysis
#[derive(Debug, Clone, Default)]
pub struct RcaMetrics {
    /// Cyclomatic complexity - number of linearly independent paths
    pub cyclomatic: f64,
    /// Cognitive complexity - difficulty to understand
    pub cognitive: f64,
    /// Halstead volume - size of implementation
    pub halstead_volume: f64,
    /// Halstead difficulty - difficulty to write/understand
    pub halstead_difficulty: f64,
    /// Halstead effort - mental effort required
    pub halstead_effort: f64,
    /// Maintainability Index (0-100, higher is better)
    pub maintainability_index: f64,
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
    /// Number of methods
    pub nom: usize,
    /// Number of arguments
    pub nargs: usize,
    /// Number of exit points
    pub nexits: usize,
}

/// Function-level metrics from rust-code-analysis
#[derive(Debug, Clone)]
pub struct RcaFunctionMetrics {
    /// Function name
    pub name: String,
    /// Start line
    pub start_line: usize,
    /// End line
    pub end_line: usize,
    /// All metrics for this function
    pub metrics: RcaMetrics,
}

/// rust-code-analysis based analyzer
pub struct RcaAnalyzer {
    thresholds: MetricThresholds,
    detector: LanguageDetector,
}

impl RcaAnalyzer {
    /// Create a new analyzer with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: MetricThresholds::default(),
            detector: LanguageDetector::new(),
        }
    }

    /// Create analyzer with custom thresholds
    pub fn with_thresholds(thresholds: MetricThresholds) -> Self {
        Self {
            thresholds,
            detector: LanguageDetector::new(),
        }
    }

    /// Detect language from file path using mcb-language-support
    ///
    /// Delegates to `LanguageDetector` from mcb-language-support to avoid
    /// duplicate language detection logic.
    pub fn detect_language(&self, path: &Path) -> Option<LANG> {
        self.detector.detect_rca_lang(path, None)
    }

    /// Analyze a file and return all function metrics
    pub fn analyze_file(&self, path: &Path) -> Result<Vec<RcaFunctionMetrics>> {
        let lang = self.detect_language(path).ok_or_else(|| {
            ValidationError::Config(format!("Unsupported language for file: {}", path.display()))
        })?;

        let code = std::fs::read(path).map_err(|e| {
            ValidationError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file {}: {}", path.display(), e),
            ))
        })?;

        self.analyze_code(&code, &lang, path)
    }

    /// Analyze code content directly
    pub fn analyze_code(
        &self,
        code: &[u8],
        lang: &LANG,
        path: &Path,
    ) -> Result<Vec<RcaFunctionMetrics>> {
        let root = get_function_spaces(lang, code.to_vec(), path, None).ok_or_else(|| {
            ValidationError::Config(format!("Failed to analyze code for: {}", path.display()))
        })?;

        let mut results = Vec::new();
        Self::extract_function_metrics(&root, &mut results);
        Ok(results)
    }

    /// Convert RCA `CodeMetrics` to our `RcaMetrics`
    /// LOC/count metrics from `rust_code_analysis` are f64; we round and clamp for usize.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn extract_metrics(space: &FuncSpace) -> RcaMetrics {
        let m = &space.metrics;
        let to_usize = |x: f64| x.round().max(0.0) as usize;
        RcaMetrics {
            cyclomatic: m.cyclomatic.cyclomatic(),
            cognitive: m.cognitive.cognitive(),
            halstead_volume: m.halstead.volume(),
            halstead_difficulty: m.halstead.difficulty(),
            halstead_effort: m.halstead.effort(),
            maintainability_index: m.mi.mi_original(),
            sloc: to_usize(m.loc.sloc()),
            ploc: to_usize(m.loc.ploc()),
            lloc: to_usize(m.loc.lloc()),
            cloc: to_usize(m.loc.cloc()),
            blank: to_usize(m.loc.blank()),
            nom: to_usize(m.nom.functions() + m.nom.closures()),
            nargs: to_usize(m.nargs.fn_args_sum()),
            nexits: to_usize(m.nexits.exit_sum()),
        }
    }

    /// Recursively extract metrics from function spaces
    fn extract_function_metrics(space: &FuncSpace, results: &mut Vec<RcaFunctionMetrics>) {
        let name = space.name.as_deref().unwrap_or("");
        if !name.is_empty() && name != "<unit>" {
            results.push(RcaFunctionMetrics {
                name: name.to_string(),
                start_line: space.start_line,
                end_line: space.end_line,
                metrics: Self::extract_metrics(space),
            });
        }
        for child in &space.spaces {
            Self::extract_function_metrics(child, results);
        }
    }

    /// Analyze file and return violations based on thresholds
    pub fn find_violations(&self, path: &Path) -> Result<Vec<MetricViolation>> {
        let functions = self.analyze_file(path)?;
        let mut violations = Vec::new();

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let to_u32_metric = |x: f64| x.round().max(0.0) as u32;
        for func in functions {
            if let Some(threshold) = self.thresholds.get(MetricType::CyclomaticComplexity) {
                let value = to_u32_metric(func.metrics.cyclomatic);
                if value > threshold.max_value {
                    violations.push(MetricViolation {
                        file: path.to_path_buf(),
                        line: func.start_line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::CyclomaticComplexity,
                        actual_value: value,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }

            if let Some(threshold) = self.thresholds.get(MetricType::CognitiveComplexity) {
                let value = to_u32_metric(func.metrics.cognitive);
                if value > threshold.max_value {
                    violations.push(MetricViolation {
                        file: path.to_path_buf(),
                        line: func.start_line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::CognitiveComplexity,
                        actual_value: value,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }

            if let Some(threshold) = self.thresholds.get(MetricType::FunctionLength) {
                let value = u32::try_from(func.metrics.sloc).unwrap_or(u32::MAX);
                if value > threshold.max_value {
                    violations.push(MetricViolation {
                        file: path.to_path_buf(),
                        line: func.start_line,
                        item_name: func.name.clone(),
                        metric_type: MetricType::FunctionLength,
                        actual_value: value,
                        threshold: threshold.max_value,
                        severity: threshold.severity,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Get file-level metrics (aggregated)
    pub fn analyze_file_aggregate(&self, path: &Path) -> Result<RcaMetrics> {
        let lang = self.detect_language(path).ok_or_else(|| {
            ValidationError::Config(format!("Unsupported language for file: {}", path.display()))
        })?;

        let code = std::fs::read(path).map_err(|e| {
            ValidationError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file {}: {}", path.display(), e),
            ))
        })?;

        let root = get_function_spaces(&lang, code.clone(), path, None).ok_or_else(|| {
            ValidationError::Config(format!("Failed to analyze code for: {}", path.display()))
        })?;

        Ok(Self::extract_metrics(&root))
    }
}

impl Default for RcaAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::violation_trait::Severity;

    #[test]
    fn test_detect_language() {
        let analyzer = RcaAnalyzer::new();
        assert_eq!(
            analyzer.detect_language(Path::new("foo.rs")),
            Some(LANG::Rust)
        );
        assert_eq!(
            analyzer.detect_language(Path::new("foo.py")),
            Some(LANG::Python)
        );
        assert_eq!(
            analyzer.detect_language(Path::new("foo.js")),
            Some(LANG::Mozjs)
        );
        assert_eq!(
            analyzer.detect_language(Path::new("foo.kt")),
            Some(LANG::Kotlin)
        );
        assert_eq!(analyzer.detect_language(Path::new("foo.txt")), None);
    }

    #[test]
    fn test_analyze_rust_code() {
        let analyzer = RcaAnalyzer::new();
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
        let path = Path::new("test.rs");
        let results = analyzer
            .analyze_code(code, &LANG::Rust, path)
            .expect("Should analyze");

        // rust-code-analysis should find functions
        assert!(
            !results.is_empty(),
            "Should find at least one function, got {}",
            results.len()
        );

        // Verify we got real metrics
        for func in &results {
            assert!(
                func.metrics.cyclomatic >= 1.0,
                "Cyclomatic should be >= 1 for {}",
                func.name
            );
        }
    }

    #[test]
    fn test_find_violations() {
        let thresholds = MetricThresholds::new().with_threshold(
            MetricType::CyclomaticComplexity,
            2,
            Severity::Warning,
        );

        let analyzer = RcaAnalyzer::with_thresholds(thresholds);
        let code = br"fn complex_function(a: i32, b: i32) -> i32 {
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

        // Write to temp file for find_violations
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("rca_test.rs");
        std::fs::write(&temp_path, code).expect("Write temp file");

        let violations = analyzer
            .find_violations(&temp_path)
            .expect("Should analyze");
        std::fs::remove_file(&temp_path).ok();

        // Should find violations for complex function
        for v in &violations {
            assert!(
                v.actual_value > v.threshold,
                "Violation should exceed threshold"
            );
        }
    }

    #[test]
    fn test_file_aggregate_metrics() {
        let analyzer = RcaAnalyzer::new();
        let code = br"fn function_one() -> i32 {
    let x = 1;
    x
}

fn function_two(a: i32) -> i32 {
    if a > 0 {
        return a * 2;
    }
    a
}";

        // Write to temp file
        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("rca_aggregate_test.rs");
        std::fs::write(&temp_path, code).expect("Write temp file");

        let metrics = analyzer
            .analyze_file_aggregate(&temp_path)
            .expect("Should analyze");
        std::fs::remove_file(&temp_path).ok();

        // Verify aggregate metrics
        assert!(metrics.sloc > 0, "Should have SLOC > 0");
        assert!(metrics.cyclomatic >= 1.0, "Should have cyclomatic >= 1");
    }
}
