//! Architecture Validators - Clean Architecture, SOLID, Quality, and hygiene
//!
//! **Documentation**: [`docs/modules/validate.md#validators-validators`](../../../../docs/modules/validate.md#validators-validators)
//!
//! Validation macros (`mk_validators!`, `impl_validator!`, `define_violations!`) live in `crate::macros`.

pub mod async_patterns;
pub mod clean_architecture;
pub mod config_quality;
pub(crate) mod declarative_support;
pub mod declarative_validator;
/// Dependency validation module
pub mod dependency;
pub mod documentation;
pub mod error_boundary;
mod helpers;
/// Hygiene validation module (e.g., TODOs, formatting)
pub mod hygiene;
/// Implementation pattern validation module
pub mod implementation;
/// KISS principle validation (Keep It Simple, Stupid).
pub mod kiss;
pub mod layer_flow;
/// Naming convention validation module
pub mod naming;
/// Organization validation (e.g., directory structure)
pub mod organization;
pub mod pattern_validator;
pub mod performance;
pub mod pmat;
pub(crate) mod pmat_native;
pub mod port_adapter;
/// Code quality validation module (unwrap, panic, metrics)
pub mod quality;
pub mod refactoring;
/// SOLID principles validation module
pub mod solid;
/// Single Source of Truth (SSOT) invariants validator
pub mod ssot;
pub mod test_quality;
pub mod visibility;

pub(crate) use helpers::for_each_non_test_non_comment_line;

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
