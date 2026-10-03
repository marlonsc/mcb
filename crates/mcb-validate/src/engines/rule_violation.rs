//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Concrete violation type shared by all rule engines.

use derive_more::Display;

use mcb_domain::ports::validation::{Severity, Violation, ViolationCategory};

/// Concrete violation structure for rule engines
#[derive(Debug, Clone, Display)]
#[display("[{id}] {message}")]
pub struct RuleViolation {
    /// Unique identifier for the violation
    pub id: String,
    /// Category of the violation (SOLID, quality, etc.)
    pub category: ViolationCategory,
    /// Severity level
    pub severity: Severity,
    /// Detailed error message
    pub message: String,
    /// Path to the file containing the violation
    pub file: Option<std::path::PathBuf>,
    /// Line number of the violation
    pub line: Option<usize>,
    /// Column number of the violation
    pub column: Option<usize>,
    /// Additional context or code snippet
    pub context: Option<String>,
}

impl Violation for RuleViolation {
    fn id(&self) -> &str {
        &self.id
    }

    fn category(&self) -> ViolationCategory {
        self.category
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn file(&self) -> Option<&std::path::PathBuf> {
        self.file.as_ref()
    }

    fn line(&self) -> Option<usize> {
        self.line
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

impl RuleViolation {
    /// Create a new rule violation
    pub fn new(
        id: impl Into<String>,
        category: ViolationCategory,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            severity,
            message: message.into(),
            file: None,
            line: None,
            column: None,
            context: None,
        }
    }

    /// Attach personal file to the violation
    #[must_use]
    pub fn with_file(mut self, file: std::path::PathBuf) -> Self {
        self.file = Some(file);
        self
    }

    /// Set the location of the violation
    #[must_use]
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Attach context information
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}
