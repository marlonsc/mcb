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
use std::path::Path;

use crate::filters::LanguageDetector;
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
    #[must_use]
    pub fn new() -> Self {
        Self {
            thresholds: MetricThresholds::default(),
            detector: LanguageDetector::new(),
        }
    }

    /// Create analyzer with custom thresholds
    #[must_use]
    pub fn with_thresholds(thresholds: MetricThresholds) -> Self {
        Self {
            thresholds,
            detector: LanguageDetector::new(),
        }
    }

    /// Detect language from file path via RCA.
    #[must_use]
    pub fn detect_language(&self, path: &Path) -> Option<LANG> {
        self.detector.detect_rca_lang(path, None)
    }

    /// Analyze a file and return all function metrics
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or its language is unsupported.
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
    ///
    /// # Errors
    ///
    /// Returns an error if code analysis fails for the given language.
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
                name: name.to_owned(),
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
    ///
    /// # Errors
    ///
    /// Returns an error if file analysis fails.
    pub fn find_violations(&self, path: &Path) -> Result<Vec<MetricViolation>> {
        let functions = self.analyze_file(path)?;
        let mut violations = Vec::new();

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
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or analyzed.
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
