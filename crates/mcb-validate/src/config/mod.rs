//! Configuration Module
//!
//! Provides file-based configuration for mcb-validate via figment
//! layered providers (embedded TOML + filesystem overrides + env vars).

mod file_config;

pub use file_config::{
    ArchitectureRulesConfig, BypassBoundaryConfig, CleanArchitectureRulesConfig,
    DependencyRulesConfig, FileConfig, GeneralConfig, ImplementationRulesConfig, KISSRulesConfig,
    LayerBoundariesConfig, LayerFlowRulesConfig, NamingRulesConfig, OrganizationRulesConfig,
    PatternRulesConfig, PerformanceRulesConfig, PortAdapterRulesConfig, QualityRulesConfig,
    RefactoringRulesConfig, RulesConfig, SolidRulesConfig, TestQualityRulesConfig,
    ValidatorsConfig, VisibilityRulesConfig,
};
