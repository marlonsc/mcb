//! PMAT Integration Validator (native implementation).
//!
//! ## What
//! Validates code against PMAT (Property-Based Maintenance Analysis Tool) standards
//! natively within the Rust process, without shelling out to external binaries.
//! Checks include cyclomatic complexity, dead code detection, and technical debt scoring.
//!
//! ## Why
//! - **Performance**: Native implementation avoids process overhead.
//! - **Integration**: Direct access to AST and analysis structures.
//! - **Compliance**: Enforces code quality gates as defined in **ADR-036**.
//!
//! ## References
//! - **ADR-036**: Enforcement Layer (Policies and Guards)
//! - **ADR-033**: Native Handler Consolidation

use std::path::PathBuf;

use crate::constants::{DEFAULT_COMPLEXITY_THRESHOLD, DEFAULT_TDG_THRESHOLD};
use crate::traits::violation::{Violation, ViolationCategory};
use mcb_domain::ports::providers::{ComplexityAnalyzer, DeadCodeDetector, TdgScorer};

use crate::define_violations;
use crate::validators::pmat_native::NativePmatAnalyzer;
use crate::{Result, Severity, ValidationConfig};

define_violations! {
    dynamic_severity,
    ViolationCategory::Pmat,
    pub enum PmatViolation {
        /// High cyclomatic complexity.
        ///
        /// ## Why
        /// Complex functions are harder to test, maintain, and reason about.
        /// Enforces code quality standards defined in **ADR-036 (Enforcement Layer)**.
        #[violation(
            id = "PMAT001",
            severity = Warning,
            message = "High complexity: {file}::{function} - complexity {complexity} (threshold: {threshold})",
            suggestion = "Consider refactoring '{function}' to reduce complexity from {complexity} to below {threshold}. Split into smaller functions or simplify control flow."
        )]
        HighComplexity {
            file: PathBuf,
            function: String,
            complexity: u32,
            threshold: u32,
            severity: Severity,
        },
        /// Dead code detected.
        ///
        /// ## Why
        /// Unused code bloats the codebase, confuses readers, and increases maintenance burden.
        /// See **ADR-036** for policy on code hygiene.
        #[violation(
            id = "PMAT002",
            severity = Warning,
            message = "Dead code: {file}:{line} - {item_type} '{name}'",
            suggestion = "{item_type} {name}"
        )]
        DeadCode {
            file: PathBuf,
            line: usize,
            item_type: String,
            name: String,
            severity: Severity,
        },
        /// Low TDG score (high technical debt).
        ///
        /// ## Why
        /// Technical Debt Gradient (TDG) predicts maintenance effort.
        /// Compares code complexity against its churn/age to identify "hotspots".
        #[violation(
            id = "PMAT003",
            severity = Warning,
            message = "High technical debt: {file} - TDG score {score} (threshold: {threshold})",
            suggestion = "Technical debt score {score} exceeds threshold {threshold}. Address code smells and reduce complexity."
        )]
        LowTdgScore {
            file: PathBuf,
            score: u32,
            threshold: u32,
            severity: Severity,
        },
        /// PMAT tooling unavailable.
        #[violation(
            id = "PMAT004",
            severity = Info,
            message = "PMAT unavailable: {message}"
        )]
        PmatUnavailable {
            message: String,
            severity: Severity,
        },
        /// PMAT execution error.
        #[violation(
            id = "PMAT005",
            severity = Error,
            message = "PMAT error running '{command}': {error}",
            suggestion = "Check analyzer configuration for '{command}'."
        )]
        PmatError {
            command: String,
            error: String,
            severity: Severity,
        },
    }
}

/// PMAT integration validator.
///
/// ## Why
/// Validates code quality metrics (complexity, dead code, technical debt)
/// directly within the MCB pipeline, as prescribed by **ADR-036**.
/// Uses native Rust providers instead of shelling out, for better performance and integration.
pub struct PmatValidator {
    /// Configuration for the validation context (workspace root, etc.).
    config: ValidationConfig,
    /// Configured limit for cyclomatic complexity.
    complexity_threshold: u32,
    /// Configured threshold for technical debt gradient.
    tdg_threshold: u32,
    /// Provider for complexity analysis logic (Strategy pattern).
    complexity_analyzer: Box<dyn ComplexityAnalyzer>,
    /// Provider for dead code detection logic.
    dead_code_detector: Box<dyn DeadCodeDetector>,
    /// Provider for TDG scoring logic.
    tdg_scorer: Box<dyn TdgScorer>,
}

impl PmatValidator {
    /// Creates a new PMAT validator with default configuration.
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_config(ValidationConfig::new(workspace_root))
    }

    /// Creates a validator with custom configuration.
    #[must_use]
    pub fn with_config(config: ValidationConfig) -> Self {
        Self {
            config,
            complexity_threshold: DEFAULT_COMPLEXITY_THRESHOLD,
            tdg_threshold: DEFAULT_TDG_THRESHOLD,
            complexity_analyzer: Box::new(NativePmatAnalyzer),
            dead_code_detector: Box::new(NativePmatAnalyzer),
            tdg_scorer: Box::new(NativePmatAnalyzer),
        }
    }

    /// Sets the cyclomatic complexity threshold (builder pattern).
    #[must_use]
    pub fn with_complexity_threshold(mut self, threshold: u32) -> Self {
        self.complexity_threshold = threshold;
        self
    }

    /// Sets the Technical Debt Gradient threshold (builder pattern).
    #[must_use]
    pub fn with_tdg_threshold(mut self, threshold: u32) -> Self {
        self.tdg_threshold = threshold;
        self
    }

    /// Native analyzer path is always available.
    #[must_use]
    pub fn is_available(&self) -> bool {
        true
    }

    /// Runs all PMAT validations and returns detected violations.
    ///
    /// # Errors
    ///
    /// Returns an error if any analysis step fails.
    pub fn validate_all(&self) -> Result<Vec<PmatViolation>> {
        let mut violations = Vec::new();
        violations.extend(self.validate_complexity()?);
        violations.extend(self.validate_dead_code()?);
        violations.extend(self.validate_tdg()?);
        Ok(violations)
    }

    /// Runs cyclomatic complexity analysis using native analyzer.
    ///
    /// # Errors
    ///
    /// Returns an error if the complexity analyzer fails.
    pub fn validate_complexity(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .complexity_analyzer
            .analyze_complexity(&self.config.workspace_root, self.complexity_threshold)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::HighComplexity {
                file: f.file,
                function: f.function,
                complexity: f.complexity,
                threshold: self.complexity_threshold,
                severity: if f.complexity > self.complexity_threshold * 2 {
                    Severity::Warning
                } else {
                    Severity::Info
                },
            })
            .collect())
    }

    /// Runs dead code analysis using native analyzer.
    ///
    /// # Errors
    ///
    /// Returns an error if the dead code detector fails.
    pub fn validate_dead_code(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .dead_code_detector
            .detect_dead_code(&self.config.workspace_root)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::DeadCode {
                file: f.file,
                line: f.line,
                item_type: f.item_type,
                name: f.name,
                severity: Severity::Info,
            })
            .collect())
    }

    /// Runs Technical Debt Gradient analysis using native analyzer.
    ///
    /// # Errors
    ///
    /// Returns an error if the TDG scorer fails.
    pub fn validate_tdg(&self) -> Result<Vec<PmatViolation>> {
        let findings = self
            .tdg_scorer
            .score_tdg(&self.config.workspace_root, self.tdg_threshold)
            .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

        Ok(findings
            .into_iter()
            .map(|f| PmatViolation::LowTdgScore {
                file: f.file,
                score: f.score,
                threshold: self.tdg_threshold,
                severity: if f.score > self.tdg_threshold + 25 {
                    Severity::Warning
                } else {
                    Severity::Info
                },
            })
            .collect())
    }
}

impl crate::traits::validator::Validator for PmatValidator {
    fn name(&self) -> &'static str {
        "pmat"
    }

    fn description(&self) -> &'static str {
        "Native PMAT-style analysis for cyclomatic complexity, dead code detection, and TDG scoring"
    }

    fn validate(&self, _config: &ValidationConfig) -> crate::Result<Vec<Box<dyn Violation>>> {
        let violations = self.validate_all()?;
        Ok(violations
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Violation>)
            .collect())
    }
}
