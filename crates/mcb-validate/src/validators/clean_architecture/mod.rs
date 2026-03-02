//! Clean Architecture validation module
//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#clean-architecture)
//!

mod validator;
mod violation;

pub use self::validator::CleanArchitectureValidator;
pub use self::violation::CleanArchitectureViolation;

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "clean_architecture",
        description: "Validates Clean Architecture compliance",
        build: |root| {
            Ok(Box::new(CleanArchitectureValidator::new(root))
                as Box<dyn mcb_domain::ports::validation::Validator>)
        },
    };
