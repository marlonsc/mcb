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

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "refactoring",
        description: "Validates refactoring completeness (duplicate definitions, missing tests, stale references)",
        build: |root| {
            Ok(Box::new(RefactoringValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
