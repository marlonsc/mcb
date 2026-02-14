//! Clean Architecture Validation
//!
//! Validates strict Clean Architecture compliance:
//! - Domain layer contains only traits and types (minimal implementations)
//! - Handlers use dependency injection (no direct service creation)
//! - Port implementations have dill provider registration
//! - Entities have identity fields
//! - Value objects are immutable
//! - Server layer boundaries are respected

use std::path::PathBuf;

use crate::Severity;
use crate::traits::violation::{Violation, ViolationCategory};

crate::define_violations! {
    dynamic_severity,
    ViolationCategory::Architecture,
    pub enum CleanArchitectureViolation {
        /// Domain layer contains implementation logic
        #[violation(
            id = "CA001",
            severity = Warning,
            message = "Domain layer contains {impl_type} at {file}:{line}",
            suggestion = "Move implementation logic to providers or infrastructure layer"
        )]
        DomainContainsImplementation {
            file: PathBuf,
            line: usize,
            impl_type: String,
            severity: Severity,
        },
        /// Handler creates service directly instead of using DI
        #[violation(
            id = "CA002",
            severity = Warning,
            message = "Handler creates {service_name} directly at {file}:{line} - {context}",
            suggestion = "Inject service via constructor injection instead of creating directly"
        )]
        HandlerCreatesService {
            file: PathBuf,
            line: usize,
            service_name: String,
            context: String,
            severity: Severity,
        },
        /// Port implementation missing dill provider registration
        #[violation(
            id = "CA003",
            severity = Warning,
            message = "{struct_name} implements {trait_name} but missing proper DI interface registration at {file}:{line}",
            suggestion = "Add proper DI component registration for {trait_name}"
        )]
        PortMissingComponentDerive {
            file: PathBuf,
            line: usize,
            struct_name: String,
            trait_name: String,
            severity: Severity,
        },
        /// Entity missing identity field
        #[violation(
            id = "CA004",
            severity = Warning,
            message = "Entity {entity_name} missing id/uuid field at {file}:{line}",
            suggestion = "Add id: Uuid or similar identity field to entity"
        )]
        EntityMissingIdentity {
            file: PathBuf,
            line: usize,
            entity_name: String,
            severity: Severity,
        },
        /// Value object has mutable method
        #[violation(
            id = "CA005",
            severity = Warning,
            message = "Value object {vo_name} has mutable method {method_name} at {file}:{line}",
            suggestion = "Value objects should be immutable - return new instance instead"
        )]
        ValueObjectMutable {
            file: PathBuf,
            line: usize,
            vo_name: String,
            method_name: String,
            severity: Severity,
        },
        /// Server imports provider directly
        #[violation(
            id = "CA006",
            severity = Warning,
            message = "Server imports provider directly: {import_path} at {file}:{line}",
            suggestion = "Import providers through infrastructure re-exports"
        )]
        ServerImportsProviderDirectly {
            file: PathBuf,
            line: usize,
            import_path: String,
            severity: Severity,
        },
        /// Infrastructure layer imports concrete service from Application
        ///
        /// CA007: Infrastructure should only import trait interfaces, not concrete types.
        #[violation(
            id = "CA007",
            severity = Error,
            message = "CA007: Infrastructure imports concrete service {concrete_type} at {file}:{line}",
            suggestion = "Import only trait interfaces from Application, not concrete implementations"
        )]
        InfrastructureImportsConcreteService {
            file: PathBuf,
            line: usize,
            import_path: String,
            concrete_type: String,
            severity: Severity,
        },
        /// Application layer imports ports from wrong location
        ///
        /// CA008: Application should import ports from the domain crate, not locally.
        #[violation(
            id = "CA008",
            severity = Error,
            message = "CA008: Application imports from {import_path} but should import from {should_be} at {file}:{line}",
            suggestion = "Import ports from {should_be} instead"
        )]
        ApplicationWrongPortImport {
            file: PathBuf,
            line: usize,
            import_path: String,
            should_be: String,
            severity: Severity,
        },
        /// Infrastructure layer imports from Application layer
        ///
        /// CA009: Infrastructure should NOT depend on Application layer.
        /// Per Clean Architecture, the dependency flow is:
        /// Server -> Infrastructure -> Domain
        ///               |                  ^
        ///               v                  |
        ///          Providers ---------> Application
        ///
        /// Infrastructure importing from Application creates circular dependencies.
        #[violation(
            id = "CA009",
            severity = Error,
            message = "CA009: Infrastructure imports from Application layer: {import_path} at {file}:{line} - violates Clean Architecture dependency direction",
            suggestion = "{suggestion}"
        )]
        InfrastructureImportsApplication {
            file: PathBuf,
            line: usize,
            import_path: String,
            suggestion: String,
            severity: Severity,
        },
    }
}

impl CleanArchitectureViolation {
    /// Returns the severity level of the violation.
    ///
    /// Delegates to the [`Violation`] trait implementation to avoid duplication.
    pub fn severity(&self) -> Severity {
        <Self as Violation>::severity(self)
    }
}
