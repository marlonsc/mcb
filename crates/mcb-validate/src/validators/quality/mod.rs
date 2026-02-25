//! Code quality validation module
//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
mod comments;
mod dead_code;
mod metrics;
mod panic;
mod unwrap;
mod validator;
mod violations;

pub use validator::QualityValidator;
pub use violations::QualityViolation;
