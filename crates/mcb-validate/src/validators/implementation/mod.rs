//! Implementation quality validation module
//!
//! **Documentation**: [`docs/modules/validate.md#implementation`](../../../../../docs/modules/validate.md#implementation)

mod validator;
mod violation;

pub use self::validator::ImplementationQualityValidator;
pub use self::violation::ImplementationViolation;
pub use crate::constants::implementation::{HARDCODED_RETURN_PATTERNS, STUB_SKIP_FILE_KEYWORDS};
