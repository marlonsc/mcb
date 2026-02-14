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
use crate::violation_trait::{Violation, ViolationCategory};

define_violations! {
    no_display,
    dynamic_severity,
    ViolationCategory::Architecture,
    pub enum CleanArchitectureViolation {
        /// Domain layer contains implementation logic
        #[violation(
            id = "CA001",
            severity = Warning,
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

impl std::fmt::Display for CleanArchitectureViolation {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DomainContainsImplementation {
                file,
                line,
                impl_type,
                ..
            } => {
                write!(
                    f,
                    "Domain layer contains {impl_type} at {}:{}",
                    file.display(),
                    line
                )
            }
            Self::HandlerCreatesService {
                file,
                line,
                service_name,
                context,
                ..
            } => {
                write!(
                    f,
                    "Handler creates {} directly at {}:{} - {}",
                    service_name,
                    file.display(),
                    line,
                    context
                )
            }
            Self::PortMissingComponentDerive {
                file,
                line,
                struct_name,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "{} implements {} but missing proper DI interface registration at {}:{}",
                    struct_name,
                    trait_name,
                    file.display(),
                    line
                )
            }
            Self::EntityMissingIdentity {
                file,
                line,
                entity_name,
                ..
            } => {
                write!(
                    f,
                    "Entity {} missing id/uuid field at {}:{}",
                    entity_name,
                    file.display(),
                    line
                )
            }
            Self::ValueObjectMutable {
                file,
                line,
                vo_name,
                method_name,
                ..
            } => {
                write!(
                    f,
                    "Value object {} has mutable method {} at {}:{}",
                    vo_name,
                    method_name,
                    file.display(),
                    line
                )
            }
            Self::ServerImportsProviderDirectly {
                file,
                line,
                import_path,
                ..
            } => {
                write!(
                    f,
                    "Server imports provider directly: {} at {}:{}",
                    import_path,
                    file.display(),
                    line
                )
            }
            Self::InfrastructureImportsConcreteService {
                file,
                line,
                concrete_type,
                ..
            } => {
                write!(
                    f,
                    "CA007: Infrastructure imports concrete service {} at {}:{}",
                    concrete_type,
                    file.display(),
                    line
                )
            }
            Self::ApplicationWrongPortImport {
                file,
                line,
                import_path,
                should_be,
                ..
            } => {
                write!(
                    f,
                    "CA008: Application imports from {} but should import from {} at {}:{}",
                    import_path,
                    should_be,
                    file.display(),
                    line
                )
            }
            Self::InfrastructureImportsApplication {
                file,
                line,
                import_path,
                ..
            } => {
                write!(
                    f,
                    "CA009: Infrastructure imports from Application layer: {} at {}:{} - violates Clean Architecture dependency direction",
                    import_path,
                    file.display(),
                    line
                )
            }
        }
    }
}
