//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::{
    GenericReport, GenericReporter, ValidationConfig, ValidatorRegistry, find_workspace_root_from,
};
use async_trait::async_trait;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    RuleInfo, ValidationOptions, ValidationProvider, ValidationReport, ValidatorInfo,
    ViolationEntry,
};
use mcb_domain::registry::validation::{
    VALIDATION_PROVIDERS, ValidationProviderConfig, ValidationProviderEntry,
};

/// Validation provider backed by the `mcb-validate` registry.
pub struct McbValidateProvider;

impl McbValidateProvider {
    /// Create a new provider instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for McbValidateProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationProvider for McbValidateProvider {
    fn provider_name(&self) -> &str {
        "mcb-validate"
    }

    fn description(&self) -> &str {
        "Architecture and code quality validation engine"
    }

    fn list_validators(&self) -> Vec<ValidatorInfo> {
        let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let registry = ValidatorRegistry::standard_for(workspace_root);

        let descriptions: HashMap<&'static str, &'static str> = registry
            .validators()
            .iter()
            .map(|validator| (validator.name(), validator.description()))
            .collect();

        let mut rule_counts: HashMap<String, usize> = HashMap::new();
        for rule in self.get_rules(None) {
            *rule_counts.entry(rule.category).or_insert(0) += 1;
        }

        ValidatorRegistry::standard_validator_names()
            .iter()
            .map(|name| ValidatorInfo {
                id: (*name).to_owned(),
                name: (*name).replace('_', " "),
                description: descriptions
                    .get(name)
                    .copied()
                    .unwrap_or("No description available")
                    .to_owned(),
                rule_count: *rule_counts.get(*name).unwrap_or(&0),
                categories: vec![(*name).to_owned()],
            })
            .collect()
    }

    fn get_rules(&self, category: Option<&str>) -> Vec<RuleInfo> {
        crate::utils::yaml::get_validation_rules(category)
    }

    async fn validate(
        &self,
        workspace_root: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport> {
        run_validation(workspace_root, &options)
    }

    async fn validate_file(
        &self,
        file_path: &Path,
        options: ValidationOptions,
    ) -> Result<ValidationReport> {
        run_file_validation(file_path, &options)
    }

    fn can_validate(&self, path: &Path) -> bool {
        if path.is_file() {
            return path
                .extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|ext| ext == "rs");
        }

        path.is_dir() && path.join("Cargo.toml").exists()
    }

    fn supported_extensions(&self) -> &[&str] {
        &["rs"]
    }
}

fn run_validation(workspace_root: &Path, options: &ValidationOptions) -> Result<ValidationReport> {
    let mut config = ValidationConfig::new(workspace_root);
    if let Some(patterns) = &options.exclude_patterns {
        for pattern in patterns {
            config = config.with_exclude_pattern(pattern.clone());
        }
    }

    let registry = ValidatorRegistry::standard_for(workspace_root);

    let report = if let Some(names) = options.validators.as_deref() {
        let names_ref: Vec<&str> = names.iter().map(String::as_str).collect();
        let violations = registry
            .validate_named(&config, &names_ref)
            .map_err(|e| Error::internal(e.to_string()))?;
        GenericReporter::create_report(&violations, workspace_root.to_path_buf())
    } else {
        let violations = registry
            .validate_all(&config)
            .map_err(|e| Error::internal(e.to_string()))?;
        GenericReporter::create_report(&violations, workspace_root.to_path_buf())
    };

    Ok(convert_report(
        report,
        options.severity_filter.as_deref(),
        options.include_suggestions,
    ))
}

fn run_file_validation(file_path: &Path, options: &ValidationOptions) -> Result<ValidationReport> {
    let workspace_root = find_workspace_root_from(file_path).unwrap_or_else(|| {
        file_path
            .parent()
            .map_or_else(|| file_path.to_path_buf(), Path::to_path_buf)
    });

    let full_report = run_validation(&workspace_root, options)?;
    let file_str = file_path.to_string_lossy().into_owned();

    let file_violations: Vec<ViolationEntry> = full_report
        .violations
        .into_iter()
        .filter(|violation| {
            violation
                .file
                .as_ref()
                .is_some_and(|file| file.contains(&file_str))
        })
        .collect();

    Ok(crate::utils::validation_report::from_violations(
        file_violations,
    ))
}

fn convert_report(
    report: GenericReport,
    severity_filter: Option<&str>,
    include_suggestions: bool,
) -> ValidationReport {
    crate::utils::validation_report::from_generic_report(
        report,
        severity_filter,
        include_suggestions,
    )
}

fn mcb_validate_factory(
    _config: &ValidationProviderConfig,
) -> std::result::Result<Arc<dyn ValidationProvider>, String> {
    Ok(Arc::new(McbValidateProvider::new()))
}

#[linkme::distributed_slice(VALIDATION_PROVIDERS)]
static MCB_VALIDATE_PROVIDER: ValidationProviderEntry = ValidationProviderEntry {
    name: "mcb-validate",
    description: "Architecture and code quality validation engine",
    build: mcb_validate_factory,
};
