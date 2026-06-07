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

mcb_domain::register_validator!(
    mcb_utils::constants::validate::VALIDATOR_QUALITY,
    "Validates code quality (no unwrap/expect)",
    |root| {
        Ok(Box::new(QualityValidator::new(root))
            as Box<dyn mcb_domain::ports::validation::Validator>)
    }
);
