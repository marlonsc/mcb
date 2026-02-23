//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::traits::violation::{Severity, Violation, ViolationCategory};

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

impl std::fmt::Display for PatternViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConcreteTypeInDi {
                file,
                line,
                concrete_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Concrete type in DI: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    concrete_type,
                    suggestion
                )
            }
            Self::MissingSendSync {
                file,
                line,
                trait_name,
                missing_bound,
                ..
            } => {
                write!(
                    f,
                    "Missing bound: {}:{} - trait {} needs {}",
                    file.display(),
                    line,
                    trait_name,
                    missing_bound
                )
            }
            Self::MissingAsyncTrait {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing #[async_trait]: {}:{} - trait {}",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::RawResultType {
                file,
                line,
                context,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Raw Result type: {}:{} - {} (use {})",
                    file.display(),
                    line,
                    context,
                    suggestion
                )
            }
            Self::MissingInterfaceBound {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Missing Interface bound: {}:{} - trait {} needs : Interface",
                    file.display(),
                    line,
                    trait_name
                )
            }
        }
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
