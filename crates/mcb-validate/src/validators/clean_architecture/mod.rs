//! Clean Architecture validation module
//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#clean-architecture)
//!

mod validator;
mod violation;

pub use self::validator::CleanArchitectureValidator;
pub use self::violation::CleanArchitectureViolation;
