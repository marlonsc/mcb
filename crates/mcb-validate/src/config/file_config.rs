//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! File-based Configuration
//!
//! Loads validation configuration via figment layered providers.
//! Config files are embedded in the binary at compile time.
//!
//! # Provider Chain (later sources override earlier):
//!
//! 1. `config/mcb-validate.toml` (embedded in binary — ALL defaults)
//! 2. `config/mcb-validate-internal.toml` (filesystem — project overrides)
//! 3. Environment variables with `MCB_VALIDATE__` prefix
//!
//! # Example Configuration
//!
//! ```toml
//! [general]
//! workspace_root = "."
//! exclude_patterns = ["target/", "tests/fixtures/"]
//!
//! [rules.architecture]
//! enabled = true
//! severity = "ERROR"
//!
//! [rules.quality]
//! enabled = true
//! max_file_lines = 500
//! allow_unwrap_in_tests = true
//!
//! [validators]
//! dependency = true
//! organization = true
//! quality = true
//! ```

use std::path::PathBuf;

use figment::Figment;
use figment::providers::{Env, Format, Toml};

use crate::config::schema::{GeneralConfig, RulesConfig, ValidatorsConfig};
use mcb_utils::constants::validate::{
    VALIDATOR_ARCHITECTURE, VALIDATOR_ASYNC_PATTERNS, VALIDATOR_CLEAN_ARCHITECTURE,
    VALIDATOR_CONFIG_QUALITY, VALIDATOR_DECLARATIVE, VALIDATOR_DEPENDENCY, VALIDATOR_DOCUMENTATION,
    VALIDATOR_ERROR_BOUNDARY, VALIDATOR_HYGIENE, VALIDATOR_IMPLEMENTATION, VALIDATOR_KISS,
    VALIDATOR_LAYER_FLOW, VALIDATOR_NAMING, VALIDATOR_ORGANIZATION, VALIDATOR_PATTERN,
    VALIDATOR_PERFORMANCE, VALIDATOR_PMAT, VALIDATOR_PORT_ADAPTER, VALIDATOR_QUALITY,
    VALIDATOR_REFACTORING, VALIDATOR_SOLID, VALIDATOR_SSOT, VALIDATOR_TESTS, VALIDATOR_VISIBILITY,
};

/// Embedded default configuration (baked into binary at compile time)
const EMBEDDED_VALIDATE_DEFAULTS: &str = include_str!("../../../../config/mcb-validate.toml");

/// Root configuration loaded via figment provider chain
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileConfig {
    /// General settings
    pub general: GeneralConfig,

    /// Rule-specific configuration
    pub rules: RulesConfig,

    /// Validator enable/disable flags
    pub validators: ValidatorsConfig,
}

impl FileConfig {
    /// Load configuration via figment layered providers.
    ///
    /// Provider chain (later sources override earlier):
    /// 1. `config/mcb-validate.toml` (embedded in binary)
    /// 2. `config/mcb-validate-internal.toml` (filesystem, project overrides)
    /// 3. Environment variables with `MCB_VALIDATE__` prefix
    ///
    pub fn load(workspace_root: impl Into<PathBuf>) -> Self {
        let root = workspace_root.into();

        let figment = Figment::new()
            // Layer 1: Validator defaults (embedded in binary)
            .merge(Toml::string(EMBEDDED_VALIDATE_DEFAULTS))
            // Layer 2: Project-specific overrides (filesystem)
            .merge(Toml::file(root.join("config/mcb-validate-internal.toml")))
            // Layer 3: Runtime env overrides
            .merge(Env::prefixed("MCB_VALIDATE__").split("__").lowercase(true));

        let mut config: Self = match figment.extract() {
            Ok(config) => config,
            Err(err) => {
                mcb_domain::warn!(
                    "validate_config",
                    "failed to load validation config; using embedded defaults",
                    &err
                );
                let mut fallback: Self = Figment::new()
                    .merge(Toml::string(EMBEDDED_VALIDATE_DEFAULTS))
                    .extract()
                    .unwrap_or_else(|e| {
                        mcb_domain::error!(
                            "validate_config",
                            "embedded mcb-validate defaults are invalid"
                        );
                        unreachable!("embedded mcb-validate defaults are invalid: {e}");
                    });
                fallback.general.workspace_root = Some(root.clone());
                fallback
            }
        };
        config.general.workspace_root = Some(root);
        config
    }

    /// Get the workspace root path
    #[must_use]
    pub fn workspace_root(&self) -> PathBuf {
        self.general
            .workspace_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }

    /// Check if a validator is enabled
    #[must_use]
    pub fn is_validator_enabled(&self, name: &str) -> bool {
        #[allow(clippy::type_complexity)] // Why: static array of validator check tuples requires complex type for polymorphic closures.
        const CHECKS: &[(&str, fn(&ValidatorsConfig) -> bool)] = &[
            (VALIDATOR_DEPENDENCY, |c| c.dependency),
            (VALIDATOR_ORGANIZATION, |c| c.organization),
            (VALIDATOR_QUALITY, |c| c.quality),
            (VALIDATOR_SOLID, |c| c.solid),
            (VALIDATOR_ARCHITECTURE, |c| c.architecture),
            (VALIDATOR_REFACTORING, |c| c.refactoring),
            (VALIDATOR_NAMING, |c| c.naming),
            (VALIDATOR_DOCUMENTATION, |c| c.documentation),
            (VALIDATOR_PATTERN, |c| c.patterns),
            (VALIDATOR_KISS, |c| c.kiss),
            (VALIDATOR_TESTS, |c| c.tests),
            (VALIDATOR_ASYNC_PATTERNS, |c| c.async_patterns),
            (VALIDATOR_ERROR_BOUNDARY, |c| c.error_boundary),
            (VALIDATOR_PERFORMANCE, |c| c.performance),
            (VALIDATOR_IMPLEMENTATION, |c| c.implementation),
            (VALIDATOR_PMAT, |c| c.pmat),
            (VALIDATOR_CLEAN_ARCHITECTURE, |c| c.clean_architecture),
            (VALIDATOR_CONFIG_QUALITY, |c| c.config_quality),
            (VALIDATOR_HYGIENE, |c| c.hygiene),
            (VALIDATOR_LAYER_FLOW, |c| c.layer_flow),
            (VALIDATOR_PORT_ADAPTER, |c| c.port_adapter),
            (VALIDATOR_SSOT, |c| c.ssot),
            (VALIDATOR_VISIBILITY, |c| c.visibility),
            (VALIDATOR_DECLARATIVE, |c| c.declarative),
        ];

        for (v_name, check) in CHECKS {
            if name == *v_name {
                return check(&self.validators);
            }
        }
        true // Unknown validators enabled by default
    }
}
