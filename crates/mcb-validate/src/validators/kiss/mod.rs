//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! KISS Principle Validation
//!
//! Validates code simplicity by detecting overly complex structures:
//! - Structs with too many fields
//! - Functions with too many parameters
//! - Overly complex builders
//! - Deep nesting
//! - Long functions

mod checks;
mod counting;
mod validator;
mod violations;

pub use self::validator::KissValidator;
pub use self::violations::KissViolation;
