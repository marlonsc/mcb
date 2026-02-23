//! Single Source of Truth (SSOT) validation module
//!
//! **Documentation**: [`docs/modules/validate.md#single-source-of-truth-ssot`](../../../../../docs/modules/validate.md#single-source-of-truth-ssot)
//!
/// SSOT validator implementation.
pub mod validator;
/// SSOT violation types.
pub mod violation;

pub use self::validator::SsotValidator;
pub use self::violation::SsotViolation;
