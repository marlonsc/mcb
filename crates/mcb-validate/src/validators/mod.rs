/// Validation macros (`mk_validators!`, `impl_validator!`, `define_violations!`)
pub mod macros;

pub mod async_patterns;
pub mod clean_architecture;
pub mod config_quality;
pub(crate) mod declarative_support;
pub mod declarative_validator;
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
/// Hygiene validation (e.g., TODOs, formatting)
pub mod hygiene;
pub mod implementation;
/// KISS principle validation (Keep It Simple, Stupid).
pub mod kiss;
pub mod layer_flow;
pub mod naming;
/// Organization validation (e.g., directory structure)
pub mod organization;
pub mod pattern_validator;
pub mod performance;
pub mod pmat;
pub(crate) mod pmat_native;
pub mod port_adapter;
/// Quality validation (e.g., unwrap, panic, metrics)
pub mod quality;
pub mod refactoring;
pub mod solid;
/// Single-source-of-truth invariants validator.
pub mod ssot;
pub mod test_quality;
pub mod visibility;

pub use self::async_patterns::{AsyncPatternValidator, AsyncViolation};
pub use self::clean_architecture::{CleanArchitectureValidator, CleanArchitectureViolation};
pub use self::config_quality::{ConfigQualityValidator, ConfigQualityViolation};
pub use self::declarative_validator::DeclarativeValidator;
pub use self::dependency::{DependencyValidator, DependencyViolation};
pub use self::documentation::{DocumentationValidator, DocumentationViolation};
pub use self::error_boundary::{ErrorBoundaryValidator, ErrorBoundaryViolation};
pub use self::hygiene::{HygieneValidator, HygieneViolation};
pub use self::implementation::{ImplementationQualityValidator, ImplementationViolation};
pub use self::kiss::{KissValidator, KissViolation};
pub use self::layer_flow::{LayerFlowValidator, LayerFlowViolation};
pub use self::naming::{NamingValidator, NamingViolation};
pub use self::organization::{OrganizationValidator, OrganizationViolation};
pub use self::pattern_validator::{PatternValidator, PatternViolation};
pub use self::performance::{PerformanceValidator, PerformanceViolation};
pub use self::pmat::{PmatValidator, PmatViolation};
pub use self::port_adapter::{PortAdapterValidator, PortAdapterViolation};
pub use self::quality::{QualityValidator, QualityViolation};
pub use self::refactoring::{RefactoringValidator, RefactoringViolation};
pub use self::solid::{SolidValidator, SolidViolation};
pub use self::ssot::{SsotValidator, SsotViolation};
pub use self::test_quality::{TestQualityValidator, TestQualityViolation};
pub use self::visibility::{VisibilityValidator, VisibilityViolation};
