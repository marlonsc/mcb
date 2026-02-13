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

use serde::{Deserialize, Serialize};

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

/// Clean Architecture violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CleanArchitectureViolation {
    /// Domain layer contains implementation logic
    DomainContainsImplementation {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number where the implementation was found.
        line: usize,
        /// Type of implementation found (e.g., "function body", "struct implementation").
        impl_type: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Handler creates service directly instead of using DI
    HandlerCreatesService {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number where the direct creation was found.
        line: usize,
        /// Name of the service being created directly.
        service_name: String,
        /// Context description of the violation for better debugging.
        context: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Port implementation missing dill provider registration
    PortMissingComponentDerive {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number where the missing derive should be.
        line: usize,
        /// Name of the struct that is missing the DI component registration.
        struct_name: String,
        /// Name of the trait (port) being implemented.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Entity missing identity field
    EntityMissingIdentity {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the entity struct definition.
        line: usize,
        /// Name of the entity struct missing an identity field.
        entity_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Value object has mutable method
    ValueObjectMutable {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the mutable method definition.
        line: usize,
        /// Name of the value object struct.
        vo_name: String,
        /// Name of the mutable method (e.g., using `&mut self`).
        method_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Server imports provider directly
    ServerImportsProviderDirectly {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the incorrect import.
        line: usize,
        /// The forbidden import path (e.g., `use providers_crate::...`).
        import_path: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Infrastructure layer imports concrete service from Application
    ///
    /// CA007: Infrastructure should only import trait interfaces, not concrete types.
    InfrastructureImportsConcreteService {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the incorrect import.
        line: usize,
        /// The full forbidden import path.
        import_path: String,
        /// Name of the concrete implementation type being imported.
        concrete_type: String,
        /// Severity level of the violation.
        severity: Severity,
    },
    /// Application layer imports ports from wrong location
    ///
    /// CA008: Application should import ports from the domain crate, not locally.
    ApplicationWrongPortImport {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the incorrect import.
        line: usize,
        /// The incorrect import path found.
        import_path: String,
        /// The expected/correct import path.
        should_be: String,
        /// Severity level of the violation.
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
    InfrastructureImportsApplication {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the incorrect import.
        line: usize,
        /// The forbidden import path crossing layers in the wrong direction.
        import_path: String,
        /// Suggested remediation action to fix the layer violation.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },
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

impl Violation for CleanArchitectureViolation {
    fn id(&self) -> &str {
        match self {
            Self::DomainContainsImplementation { .. } => "CA001",
            Self::HandlerCreatesService { .. } => "CA002",
            Self::PortMissingComponentDerive { .. } => "CA003",
            Self::EntityMissingIdentity { .. } => "CA004",
            Self::ValueObjectMutable { .. } => "CA005",
            Self::ServerImportsProviderDirectly { .. } => "CA006",
            Self::InfrastructureImportsConcreteService { .. } => "CA007",
            Self::ApplicationWrongPortImport { .. } => "CA008",
            Self::InfrastructureImportsApplication { .. } => "CA009",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Architecture
    }

    fn severity(&self) -> Severity {
        match self {
            Self::DomainContainsImplementation { severity, .. }
            | Self::HandlerCreatesService { severity, .. }
            | Self::PortMissingComponentDerive { severity, .. }
            | Self::EntityMissingIdentity { severity, .. }
            | Self::ValueObjectMutable { severity, .. }
            | Self::ServerImportsProviderDirectly { severity, .. }
            | Self::InfrastructureImportsConcreteService { severity, .. }
            | Self::ApplicationWrongPortImport { severity, .. }
            | Self::InfrastructureImportsApplication { severity, .. } => *severity,
        }
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::DomainContainsImplementation { file, .. }
            | Self::HandlerCreatesService { file, .. }
            | Self::PortMissingComponentDerive { file, .. }
            | Self::EntityMissingIdentity { file, .. }
            | Self::ValueObjectMutable { file, .. }
            | Self::ServerImportsProviderDirectly { file, .. }
            | Self::InfrastructureImportsConcreteService { file, .. }
            | Self::ApplicationWrongPortImport { file, .. }
            | Self::InfrastructureImportsApplication { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::DomainContainsImplementation { line, .. }
            | Self::HandlerCreatesService { line, .. }
            | Self::PortMissingComponentDerive { line, .. }
            | Self::EntityMissingIdentity { line, .. }
            | Self::ValueObjectMutable { line, .. }
            | Self::ServerImportsProviderDirectly { line, .. }
            | Self::InfrastructureImportsConcreteService { line, .. }
            | Self::ApplicationWrongPortImport { line, .. }
            | Self::InfrastructureImportsApplication { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::DomainContainsImplementation { .. } => {
                Some("Move implementation logic to providers or infrastructure layer".to_string())
            }
            Self::HandlerCreatesService { .. } => Some(
                "Inject service via constructor injection instead of creating directly".to_string(),
            ),
            Self::PortMissingComponentDerive { trait_name, .. } => Some(format!(
                "Add proper DI component registration for {trait_name}"
            )),
            Self::EntityMissingIdentity { .. } => {
                Some("Add id: Uuid or similar identity field to entity".to_string())
            }
            Self::ValueObjectMutable { .. } => {
                Some("Value objects should be immutable - return new instance instead".to_string())
            }
            Self::ServerImportsProviderDirectly { .. } => {
                Some("Import providers through infrastructure re-exports".to_string())
            }
            Self::InfrastructureImportsConcreteService { .. } => Some(
                // TODO(NAME001): Bad type name: interfaces (expected CamelCase) - Likely false positive in string
                "Import only trait interfaces from Application, not concrete implementations"
                    .to_string(),
            ),
            Self::ApplicationWrongPortImport { should_be, .. } => {
                Some(format!("Import ports from {should_be} instead"))
            }
            Self::InfrastructureImportsApplication { suggestion, .. } => Some(suggestion.clone()),
        }
    }
}
