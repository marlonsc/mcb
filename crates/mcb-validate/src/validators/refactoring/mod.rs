//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#refactoring)
//!
//! Refactoring Completeness Validation
//!
//! Validates that refactorings are complete and not left halfway:
//! - Orphan imports (use statements pointing to deleted/moved modules)
//! - Duplicate definitions (same type in multiple locations)
//! - Missing test files for new source files
//! - Stale re-exports (pub use of items that don't exist)
//! - Deleted module references
//! - Dead code from refactoring

mod duplicates;
mod modules;
mod tests;
mod validator;
mod violation;

pub use self::validator::RefactoringValidator;
pub use self::violation::RefactoringViolation;

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_REFACTORING,
    "Validates refactoring completeness (duplicate definitions, missing tests, stale references)",
    |root| {
        Ok(Box::new(RefactoringValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
