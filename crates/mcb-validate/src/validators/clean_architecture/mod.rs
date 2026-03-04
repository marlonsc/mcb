//! Clean Architecture validation module
//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#clean-architecture)
//!

mod validator;
mod violation;

pub use self::validator::CleanArchitectureValidator;
pub use self::violation::CleanArchitectureViolation;

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_CLEAN_ARCHITECTURE,
    "Validates Clean Architecture compliance",
    |root| {
        Ok(Box::new(CleanArchitectureValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
