//! Code hygiene validation module
//!
//! **Documentation**: [`docs/modules/validate.md#hygiene`](../../../../../docs/modules/validate.md#hygiene)
//!
/// Test directory structure checks.
mod directory;
/// Test function naming checks.
mod function_naming;
/// Inline test module checks.
mod inline_tests;
/// Test file naming checks.
mod naming;
/// Test assertion and quality checks.
mod quality;
/// Hygiene validator orchestrator.
mod validator;
/// Hygiene violation model.
mod violation;

pub use self::validator::HygieneValidator;
pub use self::violation::HygieneViolation;
