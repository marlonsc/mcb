//! Common validation types and traits.

use std::fmt::Display;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Opaque error type for validator failures.
pub type ValidatorError = Box<dyn std::error::Error + Send + Sync>;
/// Specialized result for validation operations.
pub type ValidatorResult<T> = std::result::Result<T, ValidatorError>;

/// Severity level of a code violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    /// Blocking issue that must be fixed.
    Error,
    /// Non-blocking issue that should be fixed.
    Warning,
    /// Suggestion for improvement or informational note.
    Info,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARNING"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

/// Categories of code violations identified by validators.
///
/// Used for grouping and filtering results in the validation report.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    strum_macros::Display,
    strum_macros::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum ViolationCategory {
    /// Architectural integrity, layering, and boundary violations.
    #[strum(serialize = "Architecture", serialize = "clean-architecture")]
    Architecture,
    /// General code quality and maintainability issues.
    #[strum(serialize = "Quality")]
    Quality,
    /// Project organization and module structure.
    #[strum(serialize = "Organization")]
    Organization,
    /// SOLID principles violations.
    #[strum(serialize = "SOLID")]
    Solid,
    /// Dependency Injection (linkme) registration and wiring issues.
    #[strum(
        serialize = "DI/linkme",
        serialize = "di",
        serialize = "dependency_injection"
    )]
    DependencyInjection,
    /// Configuration management and environment setup.
    #[strum(serialize = "Configuration")]
    Configuration,
    /// Web framework (Loco/Axum) specific patterns.
    #[strum(
        serialize = "Web Framework",
        serialize = "web-framework",
        serialize = "web_framework"
    )]
    WebFramework,
    /// Performance bottlenecks and inefficient patterns.
    #[strum(serialize = "Performance")]
    Performance,
    /// Async/await usage and potential deadlocks.
    #[strum(serialize = "Async")]
    Async,
    /// Missing or insufficient documentation (KISS).
    #[strum(serialize = "Documentation")]
    Documentation,
    /// Testing standards and coverage.
    #[strum(serialize = "Testing")]
    Testing,
    /// Naming conventions and style guide.
    #[strum(serialize = "Naming")]
    Naming,
    /// KISS (Keep It Simple, Stupid) principle violations.
    #[strum(serialize = "KISS")]
    Kiss,
    /// Refactoring opportunities and legacy code smells.
    #[strum(serialize = "Refactoring", serialize = "migration")]
    Refactoring,
    /// Error boundary and graceful degradation.
    #[strum(serialize = "Error Boundary", serialize = "error_boundary")]
    ErrorBoundary,
    /// Implementation details and best practices.
    #[strum(serialize = "Implementation")]
    Implementation,
    /// PMAT (Persistence, Models, Actions, Tasks) pattern.
    #[strum(serialize = "PMAT")]
    Pmat,
    /// Metrics and thresholds.
    #[strum(serialize = "Metrics")]
    Metrics,
}

/// Shared interface for all code violations.
pub trait Violation: Display + Send + Sync + std::fmt::Debug {
    /// Unique identifier for the violation type (e.g., "ARCH001").
    fn id(&self) -> &str;
    /// Category used for grouping violations in reports.
    fn category(&self) -> ViolationCategory;
    /// Importance of the violation.
    fn severity(&self) -> Severity;
    /// Path to the file containing the violation, if applicable.
    fn file(&self) -> Option<&PathBuf>;
    /// Line number where the violation occurs, if applicable.
    fn line(&self) -> Option<usize>;

    /// Human-readable description of the violation.
    fn message(&self) -> String {
        self.to_string()
    }

    /// Actionable advice on how to fix the violation.
    fn suggestion(&self) -> Option<String> {
        None
    }

    /// Helper to convert the violation into a trait object.
    fn boxed(self) -> Box<dyn Violation>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}
