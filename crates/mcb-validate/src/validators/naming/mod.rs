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
mod validator;
mod violation;

pub use self::validator::NamingValidator;
pub use self::violation::NamingViolation;

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_NAMING,
    "Validates naming conventions",
    |root| {
        Ok(Box::new(NamingValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
