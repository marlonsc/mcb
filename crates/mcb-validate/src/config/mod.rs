//! Configuration Module
//!
//! Provides file-based configuration for mcb-validate, allowing
//! projects to customize validation rules via `.mcb-validate.toml`.

mod file_config;

pub use file_config::{
    ArchitectureRulesConfig, CleanArchitectureRulesConfig, FileConfig, GeneralConfig,
    ImplementationRulesConfig, KISSRulesConfig, LayerFlowRulesConfig, NamingRulesConfig,
    OrganizationRulesConfig, PatternRulesConfig, PerformanceRulesConfig, PortAdapterRulesConfig,
    QualityRulesConfig, RefactoringRulesConfig, RulesConfig, SolidRulesConfig,
    TestQualityRulesConfig, ValidatorsConfig, VisibilityRulesConfig,
};
