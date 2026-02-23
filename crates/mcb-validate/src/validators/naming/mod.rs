//! Naming Convention Validation
//!
//! **Documentation**: [`docs/modules/validate.md#naming`](../../../../../docs/modules/validate.md#naming)
//!
//! Validates naming conventions:
//! - Structs/Enums/Traits: CamelCase
//! - Functions/Methods: `snake_case`
//! - Constants: `SCREAMING_SNAKE_CASE`
//! - Modules/Files: `snake_case`

mod checks;
pub mod constants;
mod validator;
mod violation;

pub use self::validator::NamingValidator;
pub use self::violation::NamingViolation;
