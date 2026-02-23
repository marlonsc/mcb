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

pub mod constants;
mod duplicates;
mod modules;
mod tests;
mod validator;
mod violation;

pub use self::validator::RefactoringValidator;
pub use self::violation::RefactoringViolation;
