//! SOLID Principles Validation
//!
//! Validates code against SOLID principles:
//! - SRP: Single Responsibility Principle (file/struct size, cohesion)
//! - OCP: Open/Closed Principle (excessive match statements)
//! - LSP: Liskov Substitution Principle (partial implementations)
//! - ISP: Interface Segregation Principle (large traits)
//! - DIP: Dependency Inversion Principle (concrete dependencies)

use std::path::PathBuf;

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

define_violations! {
    no_display,
    dynamic_severity,
    ViolationCategory::Solid,
    pub enum SolidViolation {
        /// SRP: Struct/Impl has too many responsibilities (too large)
        #[violation(
            id = "SOLID001",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        TooManyResponsibilities {
            file: PathBuf,
            line: usize,
            item_type: String,
            item_name: String,
            line_count: usize,
            max_allowed: usize,
            suggestion: String,
            severity: Severity,
        },

        /// OCP: Large match statement that may need extension pattern
        #[violation(
            id = "SOLID002",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        ExcessiveMatchArms {
            file: PathBuf,
            line: usize,
            arm_count: usize,
            max_recommended: usize,
            suggestion: String,
            severity: Severity,
        },

        /// ISP: Trait has too many methods
        #[violation(
            id = "SOLID003",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        TraitTooLarge {
            file: PathBuf,
            line: usize,
            trait_name: String,
            method_count: usize,
            max_allowed: usize,
            suggestion: String,
            severity: Severity,
        },

        /// DIP: Module depends on concrete implementation
        #[violation(
            id = "SOLID004",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        ConcreteDependency {
            file: PathBuf,
            line: usize,
            dependency: String,
            layer: String,
            suggestion: String,
            severity: Severity,
        },

        /// SRP: File has multiple unrelated structs
        #[violation(
            id = "SOLID005",
            severity = Info,
            suggestion = "{suggestion}"
        )]
        MultipleUnrelatedStructs {
            file: PathBuf,
            struct_names: Vec<String>,
            suggestion: String,
            severity: Severity,
        },

        /// LSP: Trait method not implemented (only panic/todo)
        #[violation(
            id = "SOLID006",
            severity = Warning,
            suggestion = "Implement the method properly or remove the trait implementation"
        )]
        PartialTraitImplementation {
            file: PathBuf,
            line: usize,
            impl_name: String,
            method_name: String,
            severity: Severity,
        },

        /// SRP: Impl block has too many methods
        #[violation(
            id = "SOLID007",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        ImplTooManyMethods {
            file: PathBuf,
            line: usize,
            type_name: String,
            method_count: usize,
            max_allowed: usize,
            suggestion: String,
            severity: Severity,
        },

        /// OCP: String-based type dispatch instead of polymorphism
        #[violation(
            id = "SOLID008",
            severity = Warning,
            suggestion = "{suggestion}"
        )]
        StringBasedDispatch {
            file: PathBuf,
            line: usize,
            match_expression: String,
            suggestion: String,
            severity: Severity,
        },
    }
}

impl SolidViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
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
