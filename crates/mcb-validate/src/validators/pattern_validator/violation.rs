//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use mcb_domain::ports::validation::{Severity, Violation, ViolationCategory};

/// Pattern violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternViolation {
    /// Concrete type used in DI instead of trait object
    ConcreteTypeInDi {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The concrete type found.
        concrete_type: String,
        /// Suggested replacement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Async trait missing Send + Sync bounds
    MissingSendSync {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// The missing bounds.
        missing_bound: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Async trait missing #[`async_trait`] attribute
    MissingAsyncTrait {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Using `std::result::Result` instead of `crate::error::Result`
    RawResultType {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Context code snippet.
        context: String,
        /// Suggested replacement.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Missing Interface trait bound for DI
    MissingInterfaceBound {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl PatternViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    #[must_use]
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}

impl PatternViolation {
    /// Render the human-readable message for this violation variant.
    fn display_message(&self) -> String {
        match self {
            Self::ConcreteTypeInDi {
                file,
                line,
                concrete_type,
                suggestion,
                ..
            } => concrete_type_message(file, *line, concrete_type, suggestion),
            Self::MissingSendSync {
                file,
                line,
                trait_name,
                missing_bound,
                ..
            } => missing_send_sync_message(file, *line, trait_name, missing_bound),
            Self::MissingAsyncTrait {
                file,
                line,
                trait_name,
                ..
            } => missing_async_trait_message(file, *line, trait_name),
            Self::RawResultType {
                file,
                line,
                context,
                suggestion,
                ..
            } => raw_result_message(file, *line, context, suggestion),
            Self::MissingInterfaceBound {
                file,
                line,
                trait_name,
                ..
            } => missing_interface_bound_message(file, *line, trait_name),
        }
    }
}

fn concrete_type_message(
    file: &std::path::Path,
    line: usize,
    concrete_type: &str,
    suggestion: &str,
) -> String {
    format!(
        "Concrete type in DI: {}:{line} - {concrete_type} (use {suggestion})",
        file.display(),
    )
}

fn missing_send_sync_message(
    file: &std::path::Path,
    line: usize,
    trait_name: &str,
    missing_bound: &str,
) -> String {
    format!(
        "Missing bound: {}:{line} - trait {trait_name} needs {missing_bound}",
        file.display(),
    )
}

fn missing_async_trait_message(file: &std::path::Path, line: usize, trait_name: &str) -> String {
    format!(
        "Missing #[async_trait]: {}:{line} - trait {trait_name}",
        file.display(),
    )
}

fn raw_result_message(
    file: &std::path::Path,
    line: usize,
    context: &str,
    suggestion: &str,
) -> String {
    format!(
        "Raw Result type: {}:{line} - {context} (use {suggestion})",
        file.display(),
    )
}

fn missing_interface_bound_message(
    file: &std::path::Path,
    line: usize,
    trait_name: &str,
) -> String {
    format!(
        "Missing Interface bound: {}:{line} - trait {trait_name} needs : Interface",
        file.display(),
    )
}

impl std::fmt::Display for PatternViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.display_message())
    }
}

impl Violation for PatternViolation {
    fn id(&self) -> &str {
        match self {
            Self::ConcreteTypeInDi { .. } => "PAT001",
            Self::MissingSendSync { .. } => "PAT002",
            Self::MissingAsyncTrait { .. } => "PAT003",
            Self::RawResultType { .. } => "PAT004",
            Self::MissingInterfaceBound { .. } => "PAT005",
        }
    }

    fn category(&self) -> ViolationCategory {
        match self {
            Self::ConcreteTypeInDi { .. } | Self::MissingInterfaceBound { .. } => {
                ViolationCategory::DependencyInjection
            }
            Self::MissingSendSync { .. } | Self::MissingAsyncTrait { .. } => {
                ViolationCategory::Async
            }
            Self::RawResultType { .. } => ViolationCategory::Quality,
        }
    }

    fn severity(&self) -> Severity {
        match self {
            Self::ConcreteTypeInDi { severity, .. }
            | Self::MissingSendSync { severity, .. }
            | Self::MissingAsyncTrait { severity, .. }
            | Self::RawResultType { severity, .. }
            | Self::MissingInterfaceBound { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::ConcreteTypeInDi { file, .. }
            | Self::MissingSendSync { file, .. }
            | Self::MissingAsyncTrait { file, .. }
            | Self::RawResultType { file, .. }
            | Self::MissingInterfaceBound { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::ConcreteTypeInDi { line, .. }
            | Self::MissingSendSync { line, .. }
            | Self::MissingAsyncTrait { line, .. }
            | Self::RawResultType { line, .. }
            | Self::MissingInterfaceBound { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::ConcreteTypeInDi { suggestion, .. } | Self::RawResultType { suggestion, .. } => {
                Some(format!("Use {suggestion}"))
            }
            Self::MissingSendSync { missing_bound, .. } => {
                Some(format!("Add {missing_bound} bounds to trait"))
            }
            Self::MissingAsyncTrait { .. } => Some("Add #[async_trait] attribute".to_owned()),
            Self::MissingInterfaceBound { .. } => Some("Add : Interface bound for DI".to_owned()),
        }
    }
}
