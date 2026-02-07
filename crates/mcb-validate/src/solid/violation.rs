//! SOLID Principles Validation
//!
//! Validates code against SOLID principles:
//! - SRP: Single Responsibility Principle (file/struct size, cohesion)
//! - OCP: Open/Closed Principle (excessive match statements)
//! - LSP: Liskov Substitution Principle (partial implementations)
//! - ISP: Interface Segregation Principle (large traits)
//! - DIP: Dependency Inversion Principle (concrete dependencies)

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

/// SOLID violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SolidViolation {
    /// SRP: Struct/Impl has too many responsibilities (too large)
    TooManyResponsibilities {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Type of the item (struct, impl).
        item_type: String,
        /// Name of the item.
        item_name: String,
        /// Current line count.
        line_count: usize,
        /// Maximum allowed line count.
        max_allowed: usize,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// OCP: Large match statement that may need extension pattern
    ExcessiveMatchArms {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Number of match arms found.
        arm_count: usize,
        /// Recommended maximum number of arms.
        max_recommended: usize,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// ISP: Trait has too many methods
    TraitTooLarge {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Number of methods in the trait.
        method_count: usize,
        /// Maximum allowed method count.
        max_allowed: usize,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// DIP: Module depends on concrete implementation
    ConcreteDependency {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the concrete dependency.
        dependency: String,
        /// Layer where the violation occurred.
        layer: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// SRP: File has multiple unrelated structs
    MultipleUnrelatedStructs {
        /// File where the violation occurred.
        file: PathBuf,
        /// List of unrelated struct names.
        struct_names: Vec<String>,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// LSP: Trait method not implemented (only panic/todo)
    PartialTraitImplementation {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the implementation.
        impl_name: String,
        /// Name of the method.
        method_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// SRP: Impl block has too many methods
    ImplTooManyMethods {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the type.
        type_name: String,
        /// Number of methods in the impl block.
        method_count: usize,
        /// Maximum allowed method count.
        max_allowed: usize,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// OCP: String-based type dispatch instead of polymorphism
    StringBasedDispatch {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The match expression being dispatched on.
        match_expression: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
}

impl SolidViolation {
    /// Returns the severity level of the violation.
    pub fn severity(&self) -> Severity {
        match self {
            Self::TooManyResponsibilities { severity, .. }
            | Self::ExcessiveMatchArms { severity, .. }
            | Self::TraitTooLarge { severity, .. }
            | Self::ConcreteDependency { severity, .. }
            | Self::MultipleUnrelatedStructs { severity, .. }
            | Self::PartialTraitImplementation { severity, .. }
            | Self::ImplTooManyMethods { severity, .. }
            | Self::StringBasedDispatch { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for SolidViolation {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyResponsibilities {
                file,
                line,
                item_type,
                item_name,
                line_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: {} {} too large: {}:{} ({} lines, max: {}) - {}",
                    item_type,
                    item_name,
                    file.display(),
                    line,
                    line_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::ExcessiveMatchArms {
                file,
                line,
                arm_count,
                max_recommended,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "OCP: Excessive match arms: {}:{} ({} arms, recommended max: {}) - {}",
                    file.display(),
                    line,
                    arm_count,
                    max_recommended,
                    suggestion
                )
            }
            Self::TraitTooLarge {
                file,
                line,
                trait_name,
                method_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "ISP: Trait {} too large: {}:{} ({} methods, max: {}) - {}",
                    trait_name,
                    file.display(),
                    line,
                    method_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::ConcreteDependency {
                file,
                line,
                dependency,
                layer,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "DIP: Concrete dependency: {}:{} - {} in {} layer - {}",
                    file.display(),
                    line,
                    dependency,
                    layer,
                    suggestion
                )
            }
            Self::MultipleUnrelatedStructs {
                file,
                struct_names,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: Multiple unrelated structs in {}: [{}] - {}",
                    file.display(),
                    struct_names.join(", "),
                    suggestion
                )
            }
            Self::PartialTraitImplementation {
                file,
                line,
                impl_name,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "LSP: Partial implementation: {}:{} - {}::{} uses panic!/todo!",
                    file.display(),
                    line,
                    impl_name,
                    method_name
                )
            }
            Self::ImplTooManyMethods {
                file,
                line,
                type_name,
                method_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "SRP: Impl {} has too many methods: {}:{} ({} methods, max: {}) - {}",
                    type_name,
                    file.display(),
                    line,
                    method_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::StringBasedDispatch {
                file,
                line,
                match_expression,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "OCP: String-based dispatch: {}:{} - {} - {}",
                    file.display(),
                    line,
                    match_expression,
                    suggestion
                )
            }
        }
    }
}

impl Violation for SolidViolation {
    fn id(&self) -> &str {
        match self {
            Self::TooManyResponsibilities { .. } => "SOLID001",
            Self::ExcessiveMatchArms { .. } => "SOLID002",
            Self::TraitTooLarge { .. } => "SOLID003",
            Self::ConcreteDependency { .. } => "SOLID004",
            Self::MultipleUnrelatedStructs { .. } => "SOLID005",
            Self::PartialTraitImplementation { .. } => "SOLID006",
            Self::ImplTooManyMethods { .. } => "SOLID007",
            Self::StringBasedDispatch { .. } => "SOLID008",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Solid
    }

    fn severity(&self) -> Severity {
        match self {
            Self::TooManyResponsibilities { severity, .. }
            | Self::ExcessiveMatchArms { severity, .. }
            | Self::TraitTooLarge { severity, .. }
            | Self::ConcreteDependency { severity, .. }
            | Self::MultipleUnrelatedStructs { severity, .. }
            | Self::PartialTraitImplementation { severity, .. }
            | Self::ImplTooManyMethods { severity, .. }
            | Self::StringBasedDispatch { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::TooManyResponsibilities { file, .. }
            | Self::ExcessiveMatchArms { file, .. }
            | Self::TraitTooLarge { file, .. }
            | Self::ConcreteDependency { file, .. }
            | Self::MultipleUnrelatedStructs { file, .. }
            | Self::PartialTraitImplementation { file, .. }
            | Self::ImplTooManyMethods { file, .. }
            | Self::StringBasedDispatch { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::MultipleUnrelatedStructs { .. } => None,
            Self::TooManyResponsibilities { line, .. }
            | Self::ExcessiveMatchArms { line, .. }
            | Self::TraitTooLarge { line, .. }
            | Self::ConcreteDependency { line, .. }
            | Self::PartialTraitImplementation { line, .. }
            | Self::ImplTooManyMethods { line, .. }
            | Self::StringBasedDispatch { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::PartialTraitImplementation { .. } => {
                Some("Implement the method properly or remove the trait implementation".to_string())
            }
            Self::TooManyResponsibilities { suggestion, .. }
            | Self::ExcessiveMatchArms { suggestion, .. }
            | Self::TraitTooLarge { suggestion, .. }
            | Self::ConcreteDependency { suggestion, .. }
            | Self::MultipleUnrelatedStructs { suggestion, .. }
            | Self::ImplTooManyMethods { suggestion, .. }
            | Self::StringBasedDispatch { suggestion, .. } => Some(suggestion.clone()),
        }
    }
}
