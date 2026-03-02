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

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "quality",
        description: "Validates code quality (no unwrap/expect)",
        build: |root| Ok(Box::new(QualityValidator::new(root))),
    };
