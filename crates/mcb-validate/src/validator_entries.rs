//!
//! Linkme-based validator registration for mcb-domain.
//!
//! Each validator is registered in [`mcb_domain::registry::validation::VALIDATOR_ENTRIES`]
//! and resolved via the domain registry (no manual factory or bootstrap).
//! CLI and handlers always use the domain registry for listing and running.

use std::path::PathBuf;
use std::sync::Arc;

use mcb_domain::error::Error;
use mcb_domain::ports::{RuleValidator, RuleValidatorRequest, ValidationReport, ViolationEntry};
use mcb_domain::registry::validation::ValidatorEntry;

use crate::ValidationConfig;
use crate::generic_reporter::GenericReporter;
use crate::traits::validator::Validator;
use crate::utils::validation_report::from_violations;

/// Adapter that wraps a [`Validator`] and implements the domain [`RuleValidator`] port.
struct RuleValidatorAdapter(Box<dyn Validator>);

impl RuleValidatorAdapter {
    fn new(inner: Box<dyn Validator>) -> Self {
        Self(inner)
    }
}

impl RuleValidator for RuleValidatorAdapter {
    fn name(&self) -> &'static str {
        self.0.name()
    }

    fn run(&self, request: &RuleValidatorRequest) -> std::result::Result<ValidationReport, Error> {
        let mut config = ValidationConfig::new(request.workspace_root.clone());
        if let Some(ref patterns) = request.exclude_patterns {
            for p in patterns {
                config = config.with_exclude_pattern(p.clone());
            }
        }
        let violations = self
            .0
            .validate(&config)
            .map_err(|e| Error::internal(e.to_string()))?;
        let entries: Vec<ViolationEntry> = violations
            .iter()
            .map(|v| GenericReporter::create_entry(v.as_ref()))
            .collect();
        Ok(from_violations(entries))
    }
}

fn build_entry(validator: Box<dyn Validator>) -> Arc<dyn RuleValidator> {
    Arc::new(RuleValidatorAdapter::new(validator))
}

macro_rules! register_validator {
    ($entry_id:ident, $name:literal, $desc:literal, $ty:path) => {
        #[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
        static $entry_id: ValidatorEntry = ValidatorEntry {
            name: $name,
            description: $desc,
            build: |root: PathBuf| {
                let v = <$ty>::new(root);
                Ok(build_entry(Box::new(v)))
            },
        };
    };
}

register_validator!(
    E_CLEAN_ARCHITECTURE,
    "clean_architecture",
    "Validates Clean Architecture dependency direction and layer boundaries",
    crate::validators::CleanArchitectureValidator
);
register_validator!(
    E_LAYER_FLOW,
    "layer_flow",
    "Validates layer flow and dependency direction",
    crate::validators::LayerFlowValidator
);
register_validator!(
    E_PORT_ADAPTER,
    "port_adapter",
    "Validates port/adapter patterns for Clean Architecture compliance",
    crate::validators::PortAdapterValidator
);
register_validator!(
    E_VISIBILITY,
    "visibility",
    "Validates visibility and access modifiers",
    crate::validators::VisibilityValidator
);
register_validator!(
    E_DEPENDENCY,
    "dependency",
    "Validates dependency and crate graph",
    crate::validators::DependencyValidator
);
register_validator!(
    E_QUALITY,
    "quality",
    "Code quality (unwrap, panic, metrics)",
    crate::validators::QualityValidator
);
register_validator!(
    E_SOLID,
    "solid",
    "SOLID principles validation",
    crate::validators::SolidValidator
);
register_validator!(
    E_NAMING,
    "naming",
    "Naming convention validation",
    crate::validators::NamingValidator
);
register_validator!(
    E_PATTERNS,
    "patterns",
    "Pattern-based validation",
    crate::validators::PatternValidator
);
register_validator!(
    E_DOCUMENTATION,
    "documentation",
    "Documentation completeness",
    crate::validators::DocumentationValidator
);
register_validator!(
    E_HYGIENE,
    "hygiene",
    "Code hygiene (TODOs, formatting)",
    crate::validators::HygieneValidator
);
register_validator!(
    E_PERFORMANCE,
    "performance",
    "Performance and loop checks",
    crate::validators::PerformanceValidator
);
register_validator!(
    E_ASYNC_PATTERNS,
    "async_patterns",
    "Async and concurrency patterns",
    crate::validators::AsyncPatternValidator
);
register_validator!(
    E_KISS,
    "kiss",
    "KISS principle validation",
    crate::validators::KissValidator
);
register_validator!(
    E_ORGANIZATION,
    "organization",
    "Code organization and structure",
    crate::validators::OrganizationValidator
);
register_validator!(
    E_IMPLEMENTATION,
    "implementation",
    "Implementation quality",
    crate::validators::ImplementationQualityValidator
);
register_validator!(
    E_REFACTORING,
    "refactoring",
    "Refactoring and duplication",
    crate::validators::RefactoringValidator
);
register_validator!(
    E_ERROR_BOUNDARY,
    "error_boundary",
    "Error boundary and handling",
    crate::validators::ErrorBoundaryValidator
);
register_validator!(
    E_DECLARATIVE,
    "declarative",
    "Declarative YAML-driven rules",
    crate::validators::DeclarativeValidator
);
