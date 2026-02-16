//! Organization violation types and trait implementations

use std::path::PathBuf;

use crate::Severity;
use crate::define_violations;
use crate::traits::violation::ViolationCategory;

define_violations! {
    ViolationCategory::Organization,
    pub enum OrganizationViolation {
        /// Indicates a magic number usage that should be replaced with a named constant.
        #[violation(
            id = "ORG001",
            severity = Info,
            message = "Magic number: {file}:{line} - {value} ({suggestion})"
        )]
        MagicNumber {
            file: PathBuf,
            line: usize,
            value: String,
            context: String,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates a string literal that is duplicated across multiple files.
        #[violation(
            id = "ORG002",
            severity = Info,
            message = "Duplicate string literal \"{value}\": {suggestion}"
        )]
        DuplicateStringLiteral {
            value: String,
            occurrences: Vec<(PathBuf, usize)>,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates a constant that is defined in an inappropriate module and should be centralized.
        #[violation(
            id = "ORG003",
            severity = Info,
            message = "Decentralized constant: {file}:{line} - {constant_name} ({suggestion})"
        )]
        DecentralizedConstant {
            file: PathBuf,
            line: usize,
            constant_name: String,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates a type definition placed in a layer that violates architectural rules.
        #[violation(
            id = "ORG004",
            severity = Warning,
            message = "Type in wrong layer: {file}:{line} - {type_name} is in {current_layer} but should be in {expected_layer}",
            suggestion = "Move type to {expected_layer} layer"
        )]
        TypeInWrongLayer {
            file: PathBuf,
            line: usize,
            type_name: String,
            current_layer: String,
            expected_layer: String,
            severity: Severity,
        },

        /// Indicates a file located in a directory that does not match its architectural responsibility.
        #[violation(
            id = "ORG005",
            severity = Warning,
            message = "File in wrong location: {file} is in {current_location} but should be in {expected_location} ({reason})",
            suggestion = "Move file to {expected_location}"
        )]
        FileInWrongLocation {
            file: PathBuf,
            current_location: String,
            expected_location: String,
            reason: String,
            severity: Severity,
        },

        /// Indicates multiple declarations with the same name, causing ambiguity or collisions.
        #[violation(
            id = "ORG006",
            severity = Warning,
            message = "Declaration collision: {name}",
            suggestion = "Consolidate declarations or use different names"
        )]
        DeclarationCollision {
            name: String,
            locations: Vec<(PathBuf, usize, String)>, // (file, line, type)
            severity: Severity,
        },

        /// Indicates a trait definition found outside the designated ports directory.
        #[violation(
            id = "ORG007",
            severity = Warning,
            message = "Trait outside ports: {file}:{line} - {trait_name} should be in domain/ports",
            suggestion = "Move trait to domain/ports"
        )]
        TraitOutsidePorts {
            file: PathBuf,
            line: usize,
            trait_name: String,
            severity: Severity,
        },

        /// Indicates an adapter implementation found outside the infrastructure layer.
        #[violation(
            id = "ORG008",
            severity = Warning,
            message = "Adapter outside infrastructure: {file}:{line} - {impl_name} should be in infrastructure/adapters",
            suggestion = "Move adapter to infrastructure/adapters"
        )]
        AdapterOutsideInfrastructure {
            file: PathBuf,
            line: usize,
            impl_name: String,
            severity: Severity,
        },

        /// Indicates a constants file that has exceeded the maximum allowed size.
        #[violation(
            id = "ORG009",
            severity = Warning,
            message = "Constants file too large: {file} has {line_count} lines (max: {max_allowed}) - consider splitting by domain",
            suggestion = "Split constants file by domain"
        )]
        ConstantsFileTooLarge {
            file: PathBuf,
            line_count: usize,
            max_allowed: usize,
            severity: Severity,
        },

        /// Indicates a recurring magic number pattern that suggests a missing shared constant.
        #[violation(
            id = "ORG010",
            severity = Info,
            message = "Common magic number: {file}:{line} - {value} ({pattern_type}) - {suggestion}"
        )]
        CommonMagicNumber {
            file: PathBuf,
            line: usize,
            value: String,
            pattern_type: String,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates a large file that lacks proper module decomposition.
        #[violation(
            id = "ORG011",
            severity = Warning,
            message = "Large file without modules: {file} has {line_count} lines (max: {max_allowed}) - {suggestion}"
        )]
        LargeFileWithoutModules {
            file: PathBuf,
            line_count: usize,
            max_allowed: usize,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates a type or service defined in multiple layers, violating separation of concerns.
        #[violation(
            id = "ORG012",
            severity = Error,
            message = "CA: Dual layer definition for {type_name}",
            suggestion = "Keep definition in one layer only"
        )]
        DualLayerDefinition {
            type_name: String,
            locations: Vec<(PathBuf, String)>, // (file, layer)
            severity: Severity,
            // no file/line mapping in macro for vectors of tuples easily,
            // but we can rely on manual or default implementation if macro supports it.
            // Actually, define_violations! automatically implements `file()` and `line()` via helpers.
            // The helpers `define_violations!(@get_file ...)` check for `file: PathBuf`.
            // DualLayerDefinition has no `file` field, so it will return None. Correct.
        },

        /// Indicates the server layer is instantiating services directly instead of using dependency injection.
        #[violation(
            id = "ORG013",
            severity = Error,
            message = "CA: Server creating service: {file}:{line} - {service_name} ({suggestion})"
        )]
        ServerCreatingServices {
            file: PathBuf,
            line: usize,
            service_name: String,
            suggestion: String,
            severity: Severity,
        },

        /// Indicates the application layer is importing from the server layer, violating dependency rules.
        #[violation(
            id = "ORG014",
            severity = Error,
            message = "CA: Application imports server: {file}:{line} - {import_statement}",
            suggestion = "Remove server import from application layer"
        )]
        ApplicationImportsServer {
            file: PathBuf,
            line: usize,
            import_statement: String,
            severity: Severity,
        },

        /// Indicates a component placed in a directory that strictly contradicts its type.
        #[violation(
            id = "ORG015",
            severity = Error,
            message = "CA: {component_type} in wrong directory: {file} is in '{current_directory}' but should be in '{expected_directory}'",
            suggestion = "Move to {expected_directory}"
        )]
        StrictDirectoryViolation {
            file: PathBuf,
            component_type: crate::ComponentType,
            current_directory: String,
            expected_directory: String,
            severity: Severity,
        },

        /// Indicates an implementation block in the domain layer that contains business logic (should be trait-only).
        #[violation(
            id = "ORG016",
            severity = Error,
            message = "CA: Domain layer has {impl_type} for {type_name}: {file}:{line} (domain should be trait-only)",
            suggestion = "Move implementation to application or infrastructure layer"
        )]
        DomainLayerImplementation {
            file: PathBuf,
            line: usize,
            impl_type: String,
            type_name: String,
            severity: Severity,
        },

        /// Indicates a handler implementation found outside the handlers directory.
        #[violation(
            id = "ORG017",
            severity = Warning,
            message = "CA: Handler {handler_name} outside handlers directory: {file}:{line}",
            suggestion = "Move handler to server/handlers"
        )]
        HandlerOutsideHandlers {
            file: PathBuf,
            line: usize,
            handler_name: String,
            severity: Severity,
        },

        /// Indicates a port trait definition found outside the ports directory.
        #[violation(
            id = "ORG018",
            severity = Warning,
            message = "CA: Port trait {trait_name} outside ports directory: {file}:{line}",
            suggestion = "Move port trait to domain/ports"
        )]
        PortOutsidePorts {
            file: PathBuf,
            line: usize,
            trait_name: String,
            severity: Severity,
        },
    }
}
