//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
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
use crate::define_violations;
use mcb_domain::ports::validation::ViolationCategory;

define_violations! {
    dynamic_severity,
    ViolationCategory::Solid,
    pub enum SolidViolation {
        /// SRP: Struct/Impl has too many responsibilities (too large)
        #[violation(
            id = "SOLID001",
            severity = Warning,
            message = "SRP: {item_type} {item_name} too large: {file}:{line} ({line_count} lines, max: {max_allowed}) - {suggestion}",
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
            message = "OCP: Excessive match arms: {file}:{line} ({arm_count} arms, recommended max: {max_recommended}) - {suggestion}",
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
            message = "ISP: Trait {trait_name} too large: {file}:{line} ({method_count} methods, max: {max_allowed}) - {suggestion}",
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
            message = "DIP: Concrete dependency: {file}:{line} - {dependency} in {layer} layer - {suggestion}",
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
            message = "SRP: Multiple unrelated structs in {file}: [{struct_names}] - {suggestion}",
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
            message = "LSP: Partial implementation: {file}:{line} - {impl_name}::{method_name} uses panic!/todo!",
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
            message = "SRP: Impl {type_name} has too many methods: {file}:{line} ({method_count} methods, max: {max_allowed}) - {suggestion}",
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
            message = "OCP: String-based dispatch: {file}:{line} - {match_expression} - {suggestion}",
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
