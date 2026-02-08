//! Organization violation types and trait implementations

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Severity;
use crate::violation_trait::{Violation, ViolationCategory};

/// Represents a specific violation of code organization rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationViolation {
    /// Indicates a magic number usage that should be replaced with a named constant.
    MagicNumber {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The magic number value found.
        value: String,
        /// The code context surrounding the violation.
        context: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a string literal that is duplicated across multiple files.
    DuplicateStringLiteral {
        /// The duplicated string value.
        value: String,
        /// List of locations where the string appears.
        occurrences: Vec<(PathBuf, usize)>,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a constant that is defined in an inappropriate module and should be centralized.
    DecentralizedConstant {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the constant.
        constant_name: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a type definition placed in a layer that violates architectural rules.
    TypeInWrongLayer {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the type.
        type_name: String,
        /// The layer where the type is currently located.
        current_layer: String,
        /// The layer where the type belongs.
        expected_layer: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a file located in a directory that does not match its architectural responsibility.
    FileInWrongLocation {
        /// File where the violation occurred.
        file: PathBuf,
        /// The current location description.
        current_location: String,
        /// The expected location description.
        expected_location: String,
        /// Reason for the placement violation.
        reason: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates multiple declarations with the same name, causing ambiguity or collisions.
    DeclarationCollision {
        /// The colliding name.
        name: String,
        /// List of locations where the name is declared.
        locations: Vec<(PathBuf, usize, String)>, // (file, line, type)
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a trait definition found outside the designated ports directory.
    TraitOutsidePorts {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the trait.
        trait_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates an adapter implementation found outside the infrastructure layer.
    AdapterOutsideInfrastructure {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the implementation.
        impl_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a constants file that has exceeded the maximum allowed size.
    ConstantsFileTooLarge {
        /// File where the violation occurred.
        file: PathBuf,
        /// Current line count of the file.
        line_count: usize,
        /// Maximum allowed line count.
        max_allowed: usize,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a recurring magic number pattern that suggests a missing shared constant.
    CommonMagicNumber {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The magic number value.
        value: String,
        /// Type of pattern detected.
        pattern_type: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a large file that lacks proper module decomposition.
    LargeFileWithoutModules {
        /// File where the violation occurred.
        file: PathBuf,
        /// Current line count of the file.
        line_count: usize,
        /// Maximum allowed line count.
        max_allowed: usize,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a type or service defined in multiple layers, violating separation of concerns.
    DualLayerDefinition {
        /// Name of the type.
        type_name: String,
        /// Locations where the type is defined.
        locations: Vec<(PathBuf, String)>, // (file, layer)
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates the server layer is instantiating services directly instead of using dependency injection.
    ServerCreatingServices {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the service being created.
        service_name: String,
        /// Suggested remediation action.
        suggestion: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates the application layer is importing from the server layer, violating dependency rules.
    ApplicationImportsServer {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// The problematic import statement.
        import_statement: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a component placed in a directory that strictly contradicts its type.
    StrictDirectoryViolation {
        /// File where the violation occurred.
        file: PathBuf,
        /// Type of the component.
        component_type: crate::ComponentType,
        /// The current directory name.
        current_directory: String,
        /// The expected directory name.
        expected_directory: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates an implementation block in the domain layer that contains business logic (should be trait-only).
    DomainLayerImplementation {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Type of implementation (e.g., method).
        impl_type: String,
        /// Name of the type being implemented.
        type_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a handler implementation found outside the handlers directory.
    HandlerOutsideHandlers {
        /// File where the violation occurred.
        file: PathBuf,
        /// Line number of the violation.
        line: usize,
        /// Name of the handler.
        handler_name: String,
        /// Severity level of the violation.
        severity: Severity,
    },

    /// Indicates a port trait definition found outside the ports directory.
    PortOutsidePorts {
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

impl OrganizationViolation {
    /// Returns the severity level of the violation.
    pub fn severity(&self) -> Severity {
        match self {
            Self::MagicNumber { severity, .. }
            | Self::DuplicateStringLiteral { severity, .. }
            | Self::DecentralizedConstant { severity, .. }
            | Self::TypeInWrongLayer { severity, .. }
            | Self::FileInWrongLocation { severity, .. }
            | Self::DeclarationCollision { severity, .. }
            | Self::TraitOutsidePorts { severity, .. }
            | Self::AdapterOutsideInfrastructure { severity, .. }
            | Self::ConstantsFileTooLarge { severity, .. }
            | Self::CommonMagicNumber { severity, .. }
            | Self::LargeFileWithoutModules { severity, .. }
            | Self::DualLayerDefinition { severity, .. }
            | Self::ServerCreatingServices { severity, .. }
            | Self::ApplicationImportsServer { severity, .. }
            | Self::StrictDirectoryViolation { severity, .. }
            | Self::DomainLayerImplementation { severity, .. }
            | Self::HandlerOutsideHandlers { severity, .. }
            | Self::PortOutsidePorts { severity, .. } => *severity,
        }
    }
}

impl std::fmt::Display for OrganizationViolation {
    #[allow(clippy::too_many_lines)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MagicNumber {
                file,
                line,
                value,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Magic number: {}:{} - {} ({})",
                    file.display(),
                    line,
                    value,
                    suggestion
                )
            }
            Self::DuplicateStringLiteral {
                value,
                occurrences,
                suggestion,
                ..
            } => {
                let locations: Vec<String> = occurrences
                    .iter()
                    .map(|(p, l)| format!("{}:{}", p.display(), l))
                    .collect();
                write!(
                    f,
                    "Duplicate string literal \"{}\": [{}] - {}",
                    value,
                    locations.join(", "),
                    suggestion
                )
            }
            Self::DecentralizedConstant {
                file,
                line,
                constant_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Decentralized constant: {}:{} - {} ({})",
                    file.display(),
                    line,
                    constant_name,
                    suggestion
                )
            }
            Self::TypeInWrongLayer {
                file,
                line,
                type_name,
                current_layer,
                expected_layer,
                ..
            } => {
                write!(
                    f,
                    "Type in wrong layer: {}:{} - {} is in {} but should be in {}",
                    file.display(),
                    line,
                    type_name,
                    current_layer,
                    expected_layer
                )
            }
            Self::FileInWrongLocation {
                file,
                current_location,
                expected_location,
                reason,
                ..
            } => {
                write!(
                    f,
                    "File in wrong location: {} is in {} but should be in {} ({})",
                    file.display(),
                    current_location,
                    expected_location,
                    reason
                )
            }
            Self::DeclarationCollision {
                name, locations, ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, l, t)| format!("{}:{}({})", p.display(), l, t))
                    .collect();
                write!(
                    f,
                    "Declaration collision: {} found at [{}]",
                    name,
                    locs.join(", ")
                )
            }
            Self::TraitOutsidePorts {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "Trait outside ports: {}:{} - {} should be in domain/ports",
                    file.display(),
                    line,
                    trait_name
                )
            }
            Self::AdapterOutsideInfrastructure {
                file,
                line,
                impl_name,
                ..
            } => {
                write!(
                    f,
                    "Adapter outside infrastructure: {}:{} - {} should be in infrastructure/adapters",
                    file.display(),
                    line,
                    impl_name
                )
            }
            Self::ConstantsFileTooLarge {
                file,
                line_count,
                max_allowed,
                ..
            } => {
                write!(
                    f,
                    "Constants file too large: {} has {} lines (max: {}) - consider splitting by domain",
                    file.display(),
                    line_count,
                    max_allowed
                )
            }
            Self::CommonMagicNumber {
                file,
                line,
                value,
                pattern_type,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Common magic number: {}:{} - {} ({}) - {}",
                    file.display(),
                    line,
                    value,
                    pattern_type,
                    suggestion
                )
            }
            Self::LargeFileWithoutModules {
                file,
                line_count,
                max_allowed,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "Large file without modules: {} has {} lines (max: {}) - {}",
                    file.display(),
                    line_count,
                    max_allowed,
                    suggestion
                )
            }
            Self::DualLayerDefinition {
                type_name,
                locations,
                ..
            } => {
                let locs: Vec<String> = locations
                    .iter()
                    .map(|(p, layer)| format!("{}({})", p.display(), layer))
                    .collect();
                write!(
                    f,
                    "CA: Dual layer definition for {}: [{}]",
                    type_name,
                    locs.join(", ")
                )
            }
            Self::ServerCreatingServices {
                file,
                line,
                service_name,
                suggestion,
                ..
            } => {
                write!(
                    f,
                    "CA: Server creating service: {}:{} - {} ({})",
                    file.display(),
                    line,
                    service_name,
                    suggestion
                )
            }
            Self::ApplicationImportsServer {
                file,
                line,
                import_statement,
                ..
            } => {
                write!(
                    f,
                    "CA: Application imports server: {}:{} - {}",
                    file.display(),
                    line,
                    import_statement
                )
            }
            Self::StrictDirectoryViolation {
                file,
                component_type,
                current_directory,
                expected_directory,
                ..
            } => {
                write!(
                    f,
                    "CA: {} in wrong directory: {} is in '{}' but should be in '{}'",
                    component_type,
                    file.display(),
                    current_directory,
                    expected_directory
                )
            }
            Self::DomainLayerImplementation {
                file,
                line,
                impl_type,
                type_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Domain layer has {} for {}: {}:{} (domain should be trait-only)",
                    impl_type,
                    type_name,
                    file.display(),
                    line
                )
            }
            Self::HandlerOutsideHandlers {
                file,
                line,
                handler_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Handler {} outside handlers directory: {}:{}",
                    handler_name,
                    file.display(),
                    line
                )
            }
            Self::PortOutsidePorts {
                file,
                line,
                trait_name,
                ..
            } => {
                write!(
                    f,
                    "CA: Port trait {} outside ports directory: {}:{}",
                    trait_name,
                    file.display(),
                    line
                )
            }
        }
    }
}

impl Violation for OrganizationViolation {
    fn id(&self) -> &str {
        match self {
            Self::MagicNumber { .. } => "ORG001",
            Self::DuplicateStringLiteral { .. } => "ORG002",
            Self::DecentralizedConstant { .. } => "ORG003",
            Self::TypeInWrongLayer { .. } => "ORG004",
            Self::FileInWrongLocation { .. } => "ORG005",
            Self::DeclarationCollision { .. } => "ORG006",
            Self::TraitOutsidePorts { .. } => "ORG007",
            Self::AdapterOutsideInfrastructure { .. } => "ORG008",
            Self::ConstantsFileTooLarge { .. } => "ORG009",
            Self::CommonMagicNumber { .. } => "ORG010",
            Self::LargeFileWithoutModules { .. } => "ORG011",
            Self::DualLayerDefinition { .. } => "ORG012",
            Self::ServerCreatingServices { .. } => "ORG013",
            Self::ApplicationImportsServer { .. } => "ORG014",
            Self::StrictDirectoryViolation { .. } => "ORG015",
            Self::DomainLayerImplementation { .. } => "ORG016",
            Self::HandlerOutsideHandlers { .. } => "ORG017",
            Self::PortOutsidePorts { .. } => "ORG018",
        }
    }

    fn category(&self) -> ViolationCategory {
        ViolationCategory::Organization
    }

    fn severity(&self) -> Severity {
        Self::severity(self)
    }

    fn file(&self) -> Option<&PathBuf> {
        match self {
            Self::DuplicateStringLiteral { .. }
            | Self::DeclarationCollision { .. }
            | Self::DualLayerDefinition { .. } => None,
            Self::MagicNumber { file, .. }
            | Self::DecentralizedConstant { file, .. }
            | Self::TypeInWrongLayer { file, .. }
            | Self::FileInWrongLocation { file, .. }
            | Self::TraitOutsidePorts { file, .. }
            | Self::AdapterOutsideInfrastructure { file, .. }
            | Self::ConstantsFileTooLarge { file, .. }
            | Self::CommonMagicNumber { file, .. }
            | Self::LargeFileWithoutModules { file, .. }
            | Self::ServerCreatingServices { file, .. }
            | Self::ApplicationImportsServer { file, .. }
            | Self::StrictDirectoryViolation { file, .. }
            | Self::DomainLayerImplementation { file, .. }
            | Self::HandlerOutsideHandlers { file, .. }
            | Self::PortOutsidePorts { file, .. } => Some(file),
        }
    }

    fn line(&self) -> Option<usize> {
        match self {
            Self::DuplicateStringLiteral { .. }
            | Self::FileInWrongLocation { .. }
            | Self::DeclarationCollision { .. }
            | Self::ConstantsFileTooLarge { .. }
            | Self::LargeFileWithoutModules { .. }
            | Self::DualLayerDefinition { .. }
            | Self::StrictDirectoryViolation { .. } => None,
            Self::MagicNumber { line, .. }
            | Self::DecentralizedConstant { line, .. }
            | Self::TypeInWrongLayer { line, .. }
            | Self::TraitOutsidePorts { line, .. }
            | Self::AdapterOutsideInfrastructure { line, .. }
            | Self::CommonMagicNumber { line, .. }
            | Self::ServerCreatingServices { line, .. }
            | Self::ApplicationImportsServer { line, .. }
            | Self::DomainLayerImplementation { line, .. }
            | Self::HandlerOutsideHandlers { line, .. }
            | Self::PortOutsidePorts { line, .. } => Some(*line),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self {
            Self::MagicNumber { suggestion, .. }
            | Self::DuplicateStringLiteral { suggestion, .. }
            | Self::DecentralizedConstant { suggestion, .. }
            | Self::CommonMagicNumber { suggestion, .. }
            | Self::LargeFileWithoutModules { suggestion, .. }
            | Self::ServerCreatingServices { suggestion, .. } => Some(suggestion.clone()),
            Self::TypeInWrongLayer { expected_layer, .. } => {
                Some(format!("Move type to {expected_layer} layer"))
            }
            Self::FileInWrongLocation {
                expected_location, ..
            } => Some(format!("Move file to {expected_location}")),
            Self::DeclarationCollision { .. } => {
                Some("Consolidate declarations or use different names".to_string())
            }
            Self::TraitOutsidePorts { .. } => Some("Move trait to domain/ports".to_string()),
            Self::AdapterOutsideInfrastructure { .. } => {
                Some("Move adapter to infrastructure/adapters".to_string())
            }
            Self::ConstantsFileTooLarge { .. } => {
                Some("Split constants file by domain".to_string())
            }
            Self::DualLayerDefinition { .. } => {
                Some("Keep definition in one layer only".to_string())
            }
            Self::ApplicationImportsServer { .. } => {
                Some("Remove server import from application layer".to_string())
            }
            Self::StrictDirectoryViolation {
                expected_directory, ..
            } => Some(format!("Move to {expected_directory}")),
            Self::DomainLayerImplementation { .. } => {
                Some("Move implementation to application or infrastructure layer".to_string())
            }
            Self::HandlerOutsideHandlers { .. } => {
                Some("Move handler to server/handlers".to_string())
            }
            Self::PortOutsidePorts { .. } => Some("Move port trait to domain/ports".to_string()),
        }
    }
}
