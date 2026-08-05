//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Configuration Module
//!
//! Provides file-based configuration for mcb-validate via figment
//! layered providers (embedded TOML + filesystem overrides + env vars).

mod file_config;
mod schema;

pub use file_config::FileConfig;
pub use schema::{
    ArchitectureRulesConfig, BypassBoundaryConfig, CleanArchitectureRulesConfig,
    DependencyRulesConfig, GeneralConfig, ImplementationRulesConfig, KISSRulesConfig,
    LayerBoundariesConfig, LayerFlowRulesConfig, NamingRulesConfig, OrganizationRulesConfig,
    PatternRulesConfig, PerformanceRulesConfig, PortAdapterRulesConfig, QualityRulesConfig,
    RefactoringRulesConfig, RulesConfig, SolidRulesConfig, TestQualityRulesConfig,
    ValidatorsConfig, VisibilityRulesConfig,
};
