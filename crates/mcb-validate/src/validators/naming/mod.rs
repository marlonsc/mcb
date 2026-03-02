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

#[linkme::distributed_slice(mcb_domain::registry::validation::VALIDATOR_ENTRIES)]
static VALIDATOR_ENTRY: mcb_domain::registry::validation::ValidatorEntry =
    mcb_domain::registry::validation::ValidatorEntry {
        name: "naming",
        description: "Validates naming conventions",
        build: |root| Ok(Box::new(NamingValidator::new(root))),
    };
