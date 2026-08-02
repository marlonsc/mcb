//! Validator trait for implementing codebase validation rules.

use super::config::{NamedCheck, ValidationConfig, run_checks};
use super::language_id::LanguageId;
use super::types::{ValidatorResult, Violation};

/// Interface for implementing codebase validation rules.
///
/// Validators are responsible for checking certain aspects of the code (naming, SOLID, cleanliness)
/// and returning a list of violations found.
pub trait Validator: Send + Sync {
    /// Returns the unique name of this validator.
    fn name(&self) -> &'static str;

    /// Returns a list of individual named checks for this validator.
    ///
    /// # Errors
    /// Returns a `ValidatorResult` which may contain an error if check preparation fails.
    fn checks<'a>(&'a self, _config: &'a ValidationConfig) -> ValidatorResult<Vec<NamedCheck<'a>>> {
        Ok(Vec::new())
    }

    /// Performs the full validation scan for this validator.
    ///
    /// # Errors
    /// Returns a `ValidatorResult` error if the validation process fails.
    fn validate(&self, config: &ValidationConfig) -> ValidatorResult<Vec<Box<dyn Violation>>> {
        run_checks(self.name(), self.checks(config)?)
    }

    /// Check if this validator should run by default in a standard scan.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Get a human-readable description of what this validator checks.
    fn description(&self) -> &'static str {
        ""
    }

    /// Get the list of languages supported by this validator.
    fn supported_languages(&self) -> &[LanguageId] {
        &[LanguageId::Rust]
    }
}
