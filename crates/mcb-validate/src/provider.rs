//!
//! **Documentation**: [docs/modules/validate.md](../../../docs/modules/validate.md)
//!
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use mcb_domain::error::Result;
use mcb_domain::ports::{
    RuleInfo, RuleValidatorRequest, ValidationOptions, ValidationProvider, ValidationReport,
    ValidatorInfo, ViolationEntry,
};
use mcb_domain::registry::validation::{
    VALIDATION_PROVIDERS, ValidationProviderConfig, ValidationProviderEntry, build_validators,
    list_validator_entries, run_validators,
};

/// Validation provider backed by linkme-discovered validators (mcb-domain registry).
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

#[async_trait::async_trait]
impl ValidationProvider for McbValidateProvider {
    fn provider_name(&self) -> &str {
        "mcb-validate"
    }

    fn description(&self) -> &str {
        "Architecture and code quality validation engine"
    }

    fn list_validators(&self) -> Vec<ValidatorInfo> {
        let entries = list_validator_entries();
        let mut rule_counts: HashMap<String, usize> = HashMap::new();
        for rule in self.get_rules(None) {
            *rule_counts.entry(rule.category).or_insert(0) += 1;
        }
        entries
            .into_iter()
            .map(|(id, desc)| ValidatorInfo {
                id: id.to_owned(),
                name: id.replace('_', " "),
                description: desc.to_owned(),
                rule_count: *rule_counts.get(id).unwrap_or(&0),
                categories: vec![id.to_owned()],
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
    let root = workspace_root.to_path_buf();
    let validators_list = build_validators(root.clone())?;
    let request = RuleValidatorRequest {
        workspace_root: root,
        validator_names: options.validators.clone(),
        severity_filter: options.severity_filter.clone(),
        exclude_patterns: options.exclude_patterns.clone(),
    };
    run_validators(&validators_list, &request)
}

fn run_file_validation(file_path: &Path, options: &ValidationOptions) -> Result<ValidationReport> {
    let workspace_root = crate::find_workspace_root_from(file_path).unwrap_or_else(|| {
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
