//! Implementation quality validation module
//!
//! **Documentation**: [`docs/modules/validate.md#implementation`](../../../../../docs/modules/validate.md#implementation)

mod validator;
mod violation;

pub use self::validator::ImplementationQualityValidator;
pub use self::violation::ImplementationViolation;

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_IMPLEMENTATION,
    "Validates implementation quality patterns (empty methods, hardcoded returns, stubs)",
    |root| {
        Ok(Box::new(ImplementationQualityValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
